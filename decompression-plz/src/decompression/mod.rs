use std::io::{Read, copy};

use brotli::Decompressor;
use bytes::{BufMut, BytesMut, buf::Writer};
use header_plz::body_headers::content_encoding::ContentEncoding;

mod decompressors;
use decompressors::*;

use crate::decompression::error::DecompressError;
mod error;

pub fn decompress(
    mut input: &[u8],
    writer: &mut Writer<BytesMut>,
    content_encoding: ContentEncoding,
) {
    match content_encoding {
        ContentEncoding::Brotli => decompress_brotli(input, writer),
        ContentEncoding::Compress | ContentEncoding::Zstd => decompress_zstd(input, writer),
        ContentEncoding::Deflate => decompress_deflate(input, writer),
        ContentEncoding::Gzip => decompress_gzip(input, writer),
        ContentEncoding::Identity => {
            std::io::copy(&mut input, writer).map_err(DecompressError::Identity)
        }
        ContentEncoding::Chunked => todo!(),
        ContentEncoding::Unknown(_) => todo!(),
    };
}
