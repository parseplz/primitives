use bytes::BytesMut;

use crate::decompression::single::error::DecompressError;

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug)]
pub enum Reason {
    Corrupt,
    PartialCorrupt(usize, usize), // header index , compression index
}

#[derive(Debug)]
pub struct DecompressErrorStruct {
    body: BytesMut,
    extra_body: Option<BytesMut>,
    error: DecompressError,
    reason: Reason,
}

impl DecompressErrorStruct {
    pub fn new(
        body: BytesMut,
        extra_body: Option<BytesMut>,
        error: DecompressError,
        reason: Reason,
    ) -> Self {
        DecompressErrorStruct {
            body,
            extra_body,
            error,
            reason,
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

    pub fn reason(&self) -> &Reason {
        &self.reason
    }

    #[cfg(test)]
    pub fn into_body_and_extra(mut self) -> (BytesMut, Option<BytesMut>) {
        (self.body, self.extra_body)
    }
}
