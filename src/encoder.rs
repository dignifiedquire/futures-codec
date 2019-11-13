use super::framed::Fuse;
use std::io::Error;

/// Encoding of messages as bytes, for use with `FramedWrite`.
pub trait Encoder {
    /// The type of items consumed by `encode`
    type Item;
    /// The type of encoding errors.
    type Error: From<Error>;

    /// Encodes an item into the `BytesMut` provided by dst.
    fn encode(&mut self, item: Self::Item, dst: &mut Vec<u8>) -> Result<(), Self::Error>;
}

impl<T, U: Encoder> Encoder for Fuse<T, U> {
    type Item = U::Item;
    type Error = U::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut Vec<u8>) -> Result<(), Self::Error> {
        self.1.encode(item, dst)
    }
}
