use bytes::{BufMut, BytesMut};
use header_plz::body_headers::{content_encoding::ContentEncoding, encoding_info::EncodingInfo};

use crate::decompression::{
    magic_bytes::is_compressed,
    multi::{decompress_multi, error::MultiDecompressError},
};

#[cfg_attr(test, derive(PartialEq))]
pub struct DecompressionStruct<'a> {
    pub main: BytesMut,
    pub extra: Option<BytesMut>,
    pub encoding_info: &'a [EncodingInfo],
    pub buf: &'a mut BytesMut,
}

impl<'a> DecompressionStruct<'a> {
    pub fn new(
        main: BytesMut,
        extra: Option<BytesMut>,
        encoding_info: &'a [EncodingInfo],
        buf: &'a mut BytesMut,
    ) -> Self {
        Self {
            main,
            extra,
            encoding_info,
            buf,
        }
    }

    pub fn last_encoding(&self) -> &ContentEncoding {
        self.encoding_info
            .last()
            .unwrap()
            .encodings()
            .last()
            .unwrap()
    }

    pub fn extra(&self) -> &[u8] {
        self.extra.as_ref().unwrap().as_ref()
    }

    pub fn is_extra_compressed(&self) -> bool {
        let last_encoding = self.last_encoding();
        is_compressed(self.extra(), last_encoding)
    }

    pub fn try_decompress_extra(&mut self) -> Result<BytesMut, MultiDecompressError> {
        let mut writer = self.buf.writer();
        decompress_multi(
            self.extra.as_ref().unwrap().as_ref(),
            &mut writer,
            &self.encoding_info,
        )
    }

    pub fn try_decompress_main(&mut self) -> Result<BytesMut, MultiDecompressError> {
        let mut writer = self.buf.writer();
        decompress_multi(self.main.as_ref(), &mut writer, &self.encoding_info)
    }
}
