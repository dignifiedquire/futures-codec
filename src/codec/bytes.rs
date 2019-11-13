use crate::{DecodeResult, Decoder, Encoder};
use byte_pool::Block;
use std::io::Error;

/// A simple codec that ships bytes around
///
/// # Example
///
///  ```
/// #![feature(async_await)]
/// use bytes::Bytes;
/// use futures::{SinkExt, TryStreamExt};
/// use std::io::Cursor;
/// use futures_codec::{BytesCodec, Framed};
///
/// async move {
///     let mut buf = vec![];
///     // Cursor implements AsyncRead and AsyncWrite
///     let cur = Cursor::new(&mut buf);
///     let mut framed = Framed::new(cur, BytesCodec {});
///
///     let msg = b"Hello World!";
///     framed.send(msg.to_vec()).await.unwrap();
///
///     while let Some(msg) = framed.try_next().await.unwrap() {
///         println!("{:?}", msg);
///     }
/// };
/// ```
pub struct BytesCodec {}

impl Encoder for BytesCodec {
    type Item = Vec<u8>;
    type Error = Error;

    fn encode(&mut self, src: Self::Item, dst: &mut Vec<u8>) -> Result<(), Self::Error> {
        dst.extend_from_slice(&src);
        Ok(())
    }
}

impl<'a> Decoder<'a> for BytesCodec {
    type Item = (Block<'a>, usize);
    type Error = Error;

    fn decode(
        &mut self,
        src: Block<'a>,
        size: usize,
    ) -> Result<DecodeResult<'a, Self::Item>, Self::Error> {
        if size > 0 {
            Ok(DecodeResult::Some((src, size)))
        } else {
            Ok(DecodeResult::None(src))
        }
    }
}
