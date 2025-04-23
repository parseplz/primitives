use crate::info_line::error::InfoLineError;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HeaderReadError {
    #[error("infoline| {0}")]
    InfoLine(#[from] InfoLineError),
    #[error("header struct| {0}")]
    HeaderStruct(String),
    // Not enough data
    #[error("header not enough data")]
    HeaderNotEnoughData,
}
