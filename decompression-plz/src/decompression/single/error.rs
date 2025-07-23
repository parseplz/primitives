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
}

impl DecompressError {
    pub fn deflate() -> Self {
        let err = std::io::Error::from(std::io::ErrorKind::InvalidData);
        DecompressError::Deflate(err)
    }

    pub fn corrupt(encoding: &ContentEncoding) -> Self {
        let err = std::io::Error::from(std::io::ErrorKind::InvalidData);
        match encoding {
            ContentEncoding::Brotli => Self::Brotli(err),
            ContentEncoding::Deflate => Self::Deflate(err),
            ContentEncoding::Gzip => Self::Gzip(err),
            ContentEncoding::Compress | ContentEncoding::Zstd => {
                Self::Zstd(err)
            }
            ContentEncoding::Unknown(enc) => Self::Unknown(enc.to_string()),
            ContentEncoding::Identity | ContentEncoding::Chunked => {
                Self::Identity(err)
            }
        }
    }
}
