use bytes::BytesMut;
use header_plz::body_headers::BodyHeader;
use header_plz::body_headers::encoding_info::EncodingInfo;

use crate::decompression::multi::error::MultiDecompressError;
use crate::decompression::state::runner;
use crate::dtraits::DecompressTrait;

pub struct DecodeStruct<'a, T> {
    pub message: T,
    pub body: BytesMut,
    pub extra_body: Option<BytesMut>,
    body_headers: Option<BodyHeader>,
    pub buf: &'a mut BytesMut,
}

impl<'a, T> DecodeStruct<'a, T>
where
    T: DecompressTrait,
{
    pub fn new(
        mut message: T,
        mut extra_body: Option<BytesMut>,
        buf: &'a mut BytesMut,
    ) -> Self {
        let mut body = message.get_body().into_bytes().unwrap();
        let mut body_headers = message.body_headers_as_mut().take();
        Self {
            message,
            body_headers,
            body,
            extra_body,
            buf,
        }
    }

    pub fn transfer_encoding_is_some(&self) -> bool {
        self.body_headers
            .as_ref()
            .and_then(|bh| bh.transfer_encoding.as_ref())
            .is_some()
    }

    pub fn content_encoding_is_some(&self) -> bool {
        self.body_headers
            .as_ref()
            .and_then(|bh| bh.content_encoding.as_ref())
            .is_some()
    }

    pub fn transfer_encoding(&mut self) -> Vec<EncodingInfo> {
        self.body_headers.as_mut().unwrap().transfer_encoding.take().unwrap()
    }

    pub fn content_encoding(&mut self) -> Vec<EncodingInfo> {
        self.body_headers.as_mut().unwrap().content_encoding.take().unwrap()
    }

    pub fn extra_body_is_some(&self) -> bool {
        self.extra_body.is_some()
    }

    pub fn take_main_body(&mut self) -> BytesMut {
        self.body.split()
    }

    pub fn take_extra_body(&mut self) -> Option<BytesMut> {
        self.extra_body.take()
    }
}
