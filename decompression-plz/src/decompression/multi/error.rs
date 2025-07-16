use bytes::BytesMut;
use thiserror::Error;

use crate::decompression::single::error::DecompressError;

pub struct MultiDecompressError {
    reason: MultiDecompressErrorReason,
    error: DecompressError,
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
}
