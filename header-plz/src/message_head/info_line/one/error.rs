use bytes::BytesMut;
use thiserror::Error;

#[cfg_attr(any(test, debug_assertions), derive(PartialEq))]
#[derive(Debug, Error)]
#[error("info line err| {}", self.error)]
pub struct InfoLineError {
    pub(crate) bytes: BytesMut,
    pub(crate) error: InfoLineErrorKind,
}

impl InfoLineError {
    #[inline(always)]
    pub(crate) fn first_ows(bytes: BytesMut) -> Self {
        Self {
            bytes,
            error: InfoLineErrorKind::FirstOws,
        }
    }

    #[inline(always)]
    pub(crate) fn second_ows(bytes: BytesMut) -> Self {
        Self {
            bytes,
            error: InfoLineErrorKind::SecondOws,
        }
    }

    pub fn into_bytes(self) -> BytesMut {
        self.bytes
    }

    pub fn bytes_mut(&mut self) -> &mut BytesMut {
        &mut self.bytes
    }
}

#[cfg_attr(any(test, debug_assertions), derive(PartialEq))]
#[derive(Debug, Error)]
pub enum InfoLineErrorKind {
    #[error("first ows")]
    FirstOws,
    #[error("second ows")]
    SecondOws,
}
