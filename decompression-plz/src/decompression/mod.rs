use brotli::Decompressor;
use header_plz::body_headers::{content_encoding::ContentEncoding, encoding_info::EncodingInfo};

mod magic_bytes;
use thiserror::Error;

pub mod multi;
pub mod single;
