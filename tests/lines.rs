use futures::{executor, TryStreamExt};
use futures_codec::{FramedRead, LinesCodec};
use std::io::Cursor;

#[test]
fn it_works() {
    let buf = "Hello\nWorld\nError".to_owned();
    let cur = Cursor::new(buf);

    let pool = byte_pool::BytePool::new();
    let mut framed = FramedRead::new(cur, LinesCodec {}, &pool);
    let next = executor::block_on(framed.try_next()).unwrap().unwrap();
    assert_eq!(next, "Hello\n");
    let next = executor::block_on(framed.try_next()).unwrap().unwrap();
    assert_eq!(next, "World\n");

    assert!(executor::block_on(framed.try_next()).is_err());
}
