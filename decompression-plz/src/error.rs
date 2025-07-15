use bytes::BytesMut;

use crate::decompression::error::DecompressError;

#[derive(Debug)]
pub struct DecompressErrorStruct {
    pub body: BytesMut,
    pub extra_body: Option<BytesMut>,
    pub error: DecompressError,
}

impl DecompressErrorStruct {
    pub fn new(body: BytesMut, extra_body: Option<BytesMut>, error: DecompressError) -> Self {
        DecompressErrorStruct {
            body,
            extra_body,
            error,
        }
    }

    pub fn is_unknown_encoding(&self) -> bool {
        matches!(self.error, DecompressError::Unknown(_))
    }

    pub fn into_body_and_error(mut self) -> (BytesMut, DecompressError) {
        if let Some(ebody) = self.extra_body {
            self.body.unsplit(ebody);
        }
        (self.body, self.error)
    }
}
