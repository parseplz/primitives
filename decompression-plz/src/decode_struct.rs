use body_plz::variants::Body;
use bytes::BytesMut;
use header_plz::body_headers::BodyHeader;
use header_plz::body_headers::encoding_info::EncodingInfo;
use header_plz::body_headers::transfer_types::TransferType;

use crate::chunked::chunked_to_raw;
use crate::decompress_trait::DecompressTrait;

pub struct DecodeStruct<'a, T> {
    pub message: &'a mut T,
    pub body: BytesMut,
    pub extra_body: Option<BytesMut>,
    body_headers: Option<BodyHeader>,
    pub buf: &'a mut BytesMut,
}

impl<'a, T> DecodeStruct<'a, T>
where
    T: DecompressTrait,
{
    pub fn new(message: &'a mut T, buf: &'a mut BytesMut) -> Self {
        let body = message.get_body().into_bytes().unwrap();
        let extra_body = message.get_extra_body();
        let body_headers = message.body_headers_as_mut().take();
        Self {
            message,
            body_headers,
            body,
            extra_body,
            buf,
        }
    }

    // TODO: implement new method in BodyHeader
    pub fn is_chunked_te(&self) -> bool {
        self.body_headers
            .as_ref()
            .and_then(|bh| bh.transfer_type.as_ref())
            .and_then(|tt| Some(tt == &TransferType::Chunked))
            .unwrap_or(false)
    }

    pub fn chunked_to_raw(&mut self) {
        if let Body::Chunked(_) = self.message.get_body() {
            chunked_to_raw(self.message, &mut self.buf);
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

    pub fn is_chunked_te_only(&self) -> bool {
        self.body_headers
            .as_ref()
            .map(|bh| bh.is_chunked_te_only())
            .unwrap_or(false)
    }

    pub fn is_identity_te_only(&self) -> bool {
        self.body_headers
            .as_ref()
            .map(|bh| bh.is_identity_te_only())
            .unwrap_or(false)
    }

    pub fn is_identity_ce_only(&self) -> bool {
        self.body_headers
            .as_ref()
            .map(|bh| bh.is_identity_ce_only())
            .unwrap_or(false)
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

    pub fn extra_body_is_none(&self) -> bool {
        self.extra_body.is_none()
    }

    pub fn take_main_body(&mut self) -> BytesMut {
        self.body.split()
    }

    pub fn take_extra_body(&mut self) -> Option<BytesMut> {
        self.extra_body.take()
    }
}
