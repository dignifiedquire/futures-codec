use futures::{executor, TryStreamExt};
use futures_codec::{BytesCodec, Framed};
use std::io::Cursor;

#[test]
fn decodes() {
    let mut buf = [0u8; 32];
    let expected = buf.clone();
    let cur = Cursor::new(&mut buf);
    let pool = byte_pool::BytePool::new();
    let mut framed = Framed::new(cur, BytesCodec {}, &pool);

    let read = executor::block_on(framed.try_next()).unwrap().unwrap();
    assert_eq!(&read[..], &expected[..]);

    assert!(executor::block_on(framed.try_next()).unwrap().is_none());
}
