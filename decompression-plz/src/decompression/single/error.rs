use header_plz::body_headers::content_encoding::ContentEncoding;
use std::io::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecompressError {
    // Error when copying partial decompressed data
    #[error("copy| {0}")]
    Copy(Error),
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

    #[error("extra raw")]
    ExtraRaw(ContentEncoding),
}

impl DecompressError {
    pub fn deflate() -> Self {
        DecompressError::ExtraRaw(ContentEncoding::Deflate)
    }

    pub fn extra_raw(encoding: ContentEncoding) -> Self {
        DecompressError::ExtraRaw(encoding)
    }
}
