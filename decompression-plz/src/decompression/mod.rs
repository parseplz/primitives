use brotli::Decompressor;
use header_plz::body_headers::{
    content_encoding::ContentEncoding, encoding_info::EncodingInfo,
};

pub mod magic_bytes;

pub mod dstruct;
pub mod multi;
pub mod single;
pub mod state;
