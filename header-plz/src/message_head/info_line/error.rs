use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfoLineError {
    #[error("first ows| {0}")]
    FirstOWS(String),
    #[error("second ows| {0}")]
    SecondOWS(String),
}
