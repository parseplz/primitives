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
}

impl From<std::io::Error> for MultiDecompressError {
    fn from(e: std::io::Error) -> Self {
        MultiDecompressError::new(
            MultiDecompressErrorReason::Copy,
            DecompressError::Copy(e),
        )
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
}
