use std::fmt::Debug;
use thiserror::Error;

use crate::message_head::info_line::one::error::InfoLineError;

#[cfg_attr(any(test, debug_assertions), derive(PartialEq))]
#[derive(Debug, Error)]
pub enum HeaderReadError {
    #[error("infoline| {0}")]
    InfoLine(#[from] InfoLineError),
    #[error("header struct| {0}")]
    HeaderStruct(String),
    #[error("header not enough data")]
    HeaderNotEnoughData,
}
