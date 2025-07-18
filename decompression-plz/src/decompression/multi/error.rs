use bytes::BytesMut;
use thiserror::Error;

use crate::decompression::single::error::DecompressError;

#[derive(Debug)]
pub struct MultiDecompressError {
    pub(crate) reason: MultiDecompressErrorReason,
    pub(crate) error: DecompressError,
}

impl MultiDecompressError {
    pub fn new(
        reason: MultiDecompressErrorReason,
        error: DecompressError,
    ) -> Self {
        MultiDecompressError {
            reason,
            error,
        }
    }

    pub fn corrupt(e: DecompressError) -> Self {
        MultiDecompressError::new(MultiDecompressErrorReason::Corrupt, e)
    }

    pub fn deflate_corrupt() -> Self {
        let e = std::io::Error::from(std::io::ErrorKind::InvalidData);
        Self::corrupt(DecompressError::Deflate(e))
    }

    pub fn extra_raw() -> Self {
        let e = std::io::Error::from(std::io::ErrorKind::InvalidData);
        MultiDecompressError::new(
            MultiDecompressErrorReason::ExtraRaw,
            DecompressError::Copy(e),
        )
    }

    pub fn reason(&self) -> &MultiDecompressErrorReason {
        &self.reason
    }

    pub fn is_corrupt(&self) -> bool {
        matches!(self.reason, MultiDecompressErrorReason::Corrupt)
    }

    pub fn from_corrupt_to_partial(
        mut self,
        partial_body: BytesMut,
        header_index: usize,
        compression_index: usize,
    ) -> Self {
        let reason = MultiDecompressErrorReason::Partial {
            partial_body,
            header_index,
            compression_index,
        };
        self.reason = reason;
        self
    }
}

impl From<std::io::Error> for MultiDecompressError {
    fn from(e: std::io::Error) -> Self {
        MultiDecompressError::new(
            MultiDecompressErrorReason::Copy,
            DecompressError::Copy(e),
        )
    }
}

impl From<MultiDecompressError> for DecompressError {
    fn from(e: MultiDecompressError) -> Self {
        e.error
    }
}

#[derive(Debug, Error)]
pub enum MultiDecompressErrorReason {
    #[error("Corrupt")]
    Corrupt,
    #[error("Partial")]
    Partial {
        partial_body: BytesMut,
        header_index: usize,
        compression_index: usize,
    },
    #[error("Copy")]
    Copy,
    #[error("ExtraRaw")]
    ExtraRaw,
}
