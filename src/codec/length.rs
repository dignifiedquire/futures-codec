use crate::{DecodeResult, Decoder, Encoder};
use byte_pool::Block;
use std::io::Error;

const U64_LENGTH: usize = std::mem::size_of::<u64>();

/// A simple `Codec` implementation sending your data by prefixing it by its length.
///
/// # Example
///
/// This codec will most likely be used wrapped in another codec like so.
///
/// ```rust
/// use futures_codec::{Decoder, Encoder, LengthCodec};
/// use bytes::{Bytes, BytesMut};
/// use std::io::{Error, ErrorKind};
///
/// pub struct MyStringCodec(LengthCodec);
///
/// impl Encoder for MyStringCodec {
///     type Item = String;
///     type Error = Error;
///
///     fn encode(&mut self, src: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
///         let bytes = Bytes::from(src);
///         self.0.encode(bytes, dst)
///     }
/// }
///
/// impl Decoder for MyStringCodec {
///     type Item = String;
///     type Error = Error;
///
///     fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
///         match self.0.decode(src)? {
///             Some(bytes) => {
///                 match String::from_utf8(bytes.to_vec()) {
///                     Ok(string) => Ok(Some(string)),
///                     Err(e) => Err(Error::new(ErrorKind::InvalidData, e))
///                 }
///             },
///             None => Ok(None),
///         }
///     }
/// }
/// ```
pub struct LengthCodec;

impl Encoder for LengthCodec {
    type Item = Vec<u8>;
    type Error = Error;

    fn encode(&mut self, src: Self::Item, dst: &mut Vec<u8>) -> Result<(), Self::Error> {
        dst.extend_from_slice(&(src.len() as u64).to_be_bytes());
        dst.extend_from_slice(&src);
        Ok(())
    }
}

impl<'a> Decoder<'a> for LengthCodec {
    type Item = (Block<'a>, usize);
    type Error = Error;

    fn decode(
        &mut self,
        src: byte_pool::Block<'a>,
        size: usize,
    ) -> Result<DecodeResult<'a, Self::Item>, Self::Error> {
        if size < U64_LENGTH {
            return Ok(DecodeResult::None(src));
        }
        let mut arr = [0u8; U64_LENGTH];
        arr.copy_from_slice(&src[..U64_LENGTH]);
        let len = u64::from_be_bytes(arr);

        if size - U64_LENGTH >= len as usize {
            Ok(DecodeResult::Some((src, size)))
        } else {
            Ok(DecodeResult::None(src))
        }
    }
}
