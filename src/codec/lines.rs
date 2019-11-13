use crate::{DecodeResult, Decoder, Encoder};
use buf_redux::Buffer;
use memchr::memchr;
use std::io::{Error, ErrorKind};

use byte_pool::Block;

/// A simple `Codec` implementation that splits up data into lines.
pub struct LinesCodec {}

impl Encoder for LinesCodec {
    type Item = String;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut Buffer) -> Result<(), Self::Error> {
        dst.reserve(item.as_bytes().len());
        dst.copy_from_slice(item.as_bytes());
        Ok(())
    }
}

impl<'a> Decoder<'a> for LinesCodec {
    type Item = String;
    type Error = Error;

    fn decode(
        &mut self,
        src: Block<'a>,
        size: usize,
    ) -> Result<DecodeResult<'a, Self::Item>, Self::Error> {
        match memchr(b'\n', &src[..size]) {
            Some(pos) => std::str::from_utf8(&src[..pos + 1])
                .map(|s| DecodeResult::Some(s.to_string()))
                .map_err(|e| Error::new(ErrorKind::InvalidData, e)),
            _ => Ok(DecodeResult::None(src)),
        }
    }
}
