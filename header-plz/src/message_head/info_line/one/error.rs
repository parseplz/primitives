use thiserror::Error;

#[cfg_attr(any(test, debug_assertions), derive(PartialEq))]
#[derive(Debug, Error)]
pub enum InfoLineError {
    #[error("first ows| {0}")]
    FirstOWS(String),
    #[error("second ows| {0}")]
    SecondOWS(String),
}
