use bytes::BytesMut;
use header_plz::body_headers::content_encoding::ContentEncoding;
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

    pub fn reason(&self) -> &MultiDecompressErrorReason {
        &self.reason
    }

    pub fn is_corrupt(&self) -> bool {
        matches!(self.reason, MultiDecompressErrorReason::Corrupt)
    }

    pub fn is_unknown_encoding(&self) -> bool {
        matches!(self.error, DecompressError::Unknown(_))
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

impl From<DecompressError> for MultiDecompressError {
    fn from(e: DecompressError) -> Self {
        MultiDecompressError::new(MultiDecompressErrorReason::Corrupt, e)
    }
}

#[cfg_attr(test, derive(PartialEq, Eq))]
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

impl MultiDecompressErrorReason {
    pub fn is_partial(&self) -> bool {
        matches!(self, MultiDecompressErrorReason::Partial { .. })
    }
}
