#![allow(warnings)]
use std::io::Read;

use brotli::Decompressor;
use bytes::{BufMut, BytesMut};
use header_plz::body_headers::content_encoding::ContentEncoding;
mod decompressors;
use decompressors::*;

fn decompress(mut input: &[u8]) {
    let buf = BytesMut::new();
    let mut writer = buf.writer();
    let content_encoding = ContentEncoding::Gzip;

    match content_encoding {
        ContentEncoding::Brotli => decompress_brotli(input, writer),
        ContentEncoding::Compress | ContentEncoding::Zstd => decompress_zstd(input, writer),
        ContentEncoding::Deflate => decompress_deflate(input, writer),
        ContentEncoding::Gzip => decompress_gzip(input, writer),
        ContentEncoding::Identity => std::io::copy(&mut input, &mut writer), //.map_err(DecompressError::Identity)
        ContentEncoding::Chunked => todo!(),
        ContentEncoding::Unknown(_) => todo!(),
    };
}
