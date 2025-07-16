use bytes::BytesMut;
use header_plz::body_headers::{content_encoding::ContentEncoding, encoding_info::EncodingInfo};

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
}
