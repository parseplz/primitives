use body_plz::variants::Body;
use bytes::BytesMut;
use header_plz::body_headers::BodyHeader;
use header_plz::body_headers::encoding_info::EncodingInfo;
use header_plz::body_headers::transfer_types::TransferType;

use crate::chunked::chunked_to_raw;
use crate::content_length::add_body_and_update_cl;
use crate::decompress_trait::DecompressTrait;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct DecodeStruct<'a, T> {
    pub message: &'a mut T,
    pub body: BytesMut,
    pub extra_body: Option<BytesMut>,
    pub body_headers: Option<BodyHeader>,
    pub buf: &'a mut BytesMut,
}

impl<'a, T> DecodeStruct<'a, T>
where
    T: DecompressTrait + std::fmt::Debug,
{
    pub fn new(message: &'a mut T, buf: &'a mut BytesMut) -> Self {
        let body = match message.get_body() {
            Body::Raw(data) => data,
            Body::Chunked(chunks) => {
                message.set_body(Body::Chunked(chunks));
                buf.split()
            }
        };
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
            .map(|tt| tt == &TransferType::Chunked)
            .unwrap_or(false)
    }

    pub fn chunked_to_raw(&mut self) {
        chunked_to_raw(self.message, self.buf);
        self.body = self.message.get_body().into_bytes().unwrap();
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

    pub fn get_transfer_encoding(&mut self) -> Vec<EncodingInfo> {
        self.body_headers.as_mut().unwrap().transfer_encoding.take().unwrap()
    }

    pub fn get_content_encoding(&mut self) -> Vec<EncodingInfo> {
        self.body_headers.as_mut().unwrap().content_encoding.take().unwrap()
    }

    pub fn set_transfer_encoding(&mut self, te: Vec<EncodingInfo>) {
        self.body_headers.as_mut().unwrap().transfer_encoding = Some(te);
    }

    pub fn set_content_encoding(&mut self, ce: Vec<EncodingInfo>) {
        self.body_headers.as_mut().unwrap().content_encoding = Some(ce);
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

    pub fn add_body_and_update_cl(&mut self) {
        let mut body = self.take_main_body();
        if let Some(extra) = self.take_extra_body() {
            body.unsplit(extra);
        }
        add_body_and_update_cl(self.message, body);
    }
}
