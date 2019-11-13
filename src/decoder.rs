use super::framed::Fuse;
use super::framed_write::FramedWrite2;
use byte_pool::Block;
use std::io::Error;

/// The result of attempting a decoding.
pub enum DecodeResult<'a, I> {
    /// Returned when a value was successfully decoded.
    Some(I),
    /// Returned when parsing is not complete yet. Returns the used buffer for further
    /// filling.
    None(Block<'a>),
}

/// Decoding of frames via buffers, for use with `FramedRead`.
pub trait Decoder<'a> {
    /// The type of items returned by `decode`
    type Item;
    /// The type of decoding errors.
    type Error: From<Error>;

    /// Decode an item from the src `Block` into an item. Must return
    fn decode(&mut self, src: Block<'a>) -> Result<DecodeResult<'a, Self::Item>, Self::Error>;
}

impl<'a, T, U: Decoder<'a>> Decoder<'a> for Fuse<T, U> {
    type Item = U::Item;
    type Error = U::Error;

    fn decode(&mut self, src: Block<'a>) -> Result<DecodeResult<'a, Self::Item>, Self::Error> {
        self.1.decode(src)
    }
}

impl<'a, T: Decoder<'a>> Decoder<'a> for FramedWrite2<T> {
    type Item = T::Item;
    type Error = T::Error;

    fn decode(&mut self, src: Block<'a>) -> Result<DecodeResult<'a, Self::Item>, Self::Error> {
        self.inner.decode(src)
    }
}
