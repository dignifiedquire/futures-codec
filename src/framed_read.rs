use super::framed::Fuse;
use super::{DecodeResult, Decoder};

use futures::io::AsyncRead;
use futures::{ready, Sink, Stream, TryStreamExt};
use std::io;
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A `Stream` of messages decoded from an `AsyncRead`.
///
/// # Example
/// ```
/// #![feature(async_await)]
/// use futures_codec::{BytesCodec, FramedRead};
/// use futures::{executor, TryStreamExt};
/// use bytes::Bytes;
///
/// let buf = b"Hello World!";
/// let mut framed = FramedRead::new(&buf[..], BytesCodec {});
///
/// executor::block_on(async move {
///     let msg = framed.try_next().await.unwrap().unwrap();
///     assert_eq!(msg, Bytes::from(&buf[..]));
/// })
/// ```
pub struct FramedRead<'a, T, D> {
    inner: FramedRead2<'a, Fuse<T, D>>,
}

impl<'a, T, D> FramedRead<'a, T, D>
where
    T: AsyncRead,
    D: Decoder<'a>,
{
    /// Creates a new `FramedRead` transport with the given `Decoder`.
    pub fn new(inner: T, decoder: D, pool: &'a byte_pool::BytePool) -> Self {
        Self {
            inner: framed_read_2(Fuse(inner, decoder), pool),
        }
    }

    /// Release the I/O and Decoder
    pub fn release(self: Self) -> (T, D) {
        let fuse = self.inner.release();
        (fuse.0, fuse.1)
    }
}

impl<'a, T, D> Stream for FramedRead<'a, T, D>
where
    T: AsyncRead + Unpin,
    D: Decoder<'a>,
{
    type Item = Result<D::Item, D::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.try_poll_next_unpin(cx)
    }
}

pub struct FramedRead2<'a, T> {
    inner: T,
    pool: &'a byte_pool::BytePool,
    buffer: byte_pool::Block<'a>,
    pos: usize,
}

const INITIAL_CAPACITY: usize = 8 * 1024;

pub fn framed_read_2<'a, T>(inner: T, pool: &'a byte_pool::BytePool) -> FramedRead2<'a, T> {
    FramedRead2 {
        inner,
        pool,
        buffer: pool.alloc(INITIAL_CAPACITY),
        pos: 0,
    }
}

impl<'a, T> Stream for FramedRead2<'a, T>
where
    T: AsyncRead + Decoder<'a> + Unpin,
{
    type Item = Result<T::Item, T::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;

        let buf = std::mem::replace(&mut this.buffer, this.pool.alloc(INITIAL_CAPACITY));
        match this.inner.decode(buf)? {
            DecodeResult::Some(item) => {
                return Poll::Ready(Some(Ok(item)));
            }
            DecodeResult::None(buf) => {
                std::mem::replace(&mut this.buffer, buf);
            }
        }

        loop {
            let n = ready!(Pin::new(&mut this.inner).poll_read(cx, &mut this.buffer[this.pos..]))?;

            // TODO: pass in valid size
            this.pos += n;

            let buf = std::mem::replace(&mut this.buffer, this.pool.alloc(INITIAL_CAPACITY));
            match this.inner.decode(buf)? {
                DecodeResult::Some(item) => return Poll::Ready(Some(Ok(item))),
                DecodeResult::None(buf) => {
                    std::mem::replace(&mut this.buffer, buf);

                    if this.pos == 0 {
                        return Poll::Ready(None);
                    } else if n == 0 {
                        return Poll::Ready(Some(Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "bytes remaining in stream",
                        )
                        .into())));
                    }
                }
            }
        }
    }
}

impl<'a, T, I> Sink<I> for FramedRead2<'a, T>
where
    T: Sink<I> + Unpin,
{
    type Error = T::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_ready(cx)
    }
    fn start_send(mut self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        Pin::new(&mut self.inner).start_send(item)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

impl<'a, T> FramedRead2<'a, T> {
    pub fn release(self: Self) -> T {
        self.inner
    }
}
