use std::io::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecompressError {
    #[error("brotli| {0}")]
    Brotli(Error),
    #[error("deflate| {0}")]
    Deflate(Error),
    #[error("gzip| {0}")]
    Gzip(Error),
    #[error("zstd| {0}")]
    Zstd(Error),
    #[error("identity| {0}")]
    Identity(Error),
    #[error("unknown| {0}")]
    Unknown(String),
}
