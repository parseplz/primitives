use bytes::BytesMut;
use std::fmt::Debug;
use thiserror::Error;

use crate::message_head::info_line::one::error::InfoLineError;

#[cfg_attr(any(test, debug_assertions), derive(PartialEq))]
#[derive(Debug, Error)]
pub enum MessageHeadError {
    #[error("unable to find first line")]
    NoInfoLine(BytesMut),
    #[error("infoline| {0}")]
    ParseInfoLine(#[from] InfoLineError),
}

impl MessageHeadError {
    pub fn into_bytes(self) -> BytesMut {
        match self {
            MessageHeadError::NoInfoLine(buf) => buf,
            MessageHeadError::ParseInfoLine(e) => e.into_bytes(),
        }
    }
}
