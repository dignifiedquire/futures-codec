use futures::executor;
use futures::stream::StreamExt;
use futures::task::Context;
use futures::{AsyncRead, Poll};
use futures_codec::{FramedRead, LinesCodec};
use std::io;
use std::pin::Pin;

// Sends two lines at once, then nothing else forever
struct MockBurstySender {
    sent: bool,
}
impl AsyncRead for MockBurstySender {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        const MESSAGES: &'static [u8] = b"one\ntwo\n";
        if !self.sent && buf.len() >= MESSAGES.len() {
            self.sent = true;
            buf[0..MESSAGES.len()].clone_from_slice(MESSAGES);
            Poll::Ready(Ok(MESSAGES.len()))
        } else {
            Poll::Pending
        }
    }
}

#[test]
fn line_read_multi() {
    let io = MockBurstySender { sent: false };
    let pool = byte_pool::BytePool::new();

    let mut framed = FramedRead::new(io, LinesCodec {}, &pool);
    let one = executor::block_on(framed.next()).unwrap().unwrap();
    assert_eq!(one, "one\n");
    let two = executor::block_on(framed.next()).unwrap().unwrap();
    assert_eq!(two, "two\n");
}
