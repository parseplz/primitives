use std::io::Write;

use flate2::Compression;
use header_plz::body_headers::{
    content_encoding::ContentEncoding, encoding_info::EncodingInfo,
};

pub const INPUT: &[u8] = b"hello world";

pub const ALL_COMPRESSIONS: &str = "br, deflate, gzip, zstd";

pub fn all_compressed_data() -> Vec<u8> {
    let brotli_compressed = compress_brotli(INPUT);
    let deflate_compressed = compress_deflate(&brotli_compressed);
    let gzip_compressed = compress_gzip(&deflate_compressed);
    compress_zstd(&gzip_compressed)
}

pub fn single_compression(encoding: &ContentEncoding) -> Vec<u8> {
    match encoding {
        ContentEncoding::Brotli => compress_brotli(INPUT),
        ContentEncoding::Deflate => compress_deflate(INPUT),
        ContentEncoding::Gzip => compress_gzip(INPUT),
        ContentEncoding::Zstd | ContentEncoding::Compress => {
            compress_zstd(INPUT)
        }
        ContentEncoding::Identity => INPUT.to_vec(),
        _ => panic!(),
    }
}

pub fn all_encoding_info_multi_header() -> Vec<EncodingInfo> {
    vec![
        EncodingInfo::new(1, vec![ContentEncoding::Brotli]),
        EncodingInfo::new(3, vec![ContentEncoding::Deflate]),
        EncodingInfo::new(5, vec![ContentEncoding::Identity]),
        EncodingInfo::new(7, vec![ContentEncoding::Gzip]),
        EncodingInfo::new(9, vec![ContentEncoding::Zstd]),
    ]
}

pub fn all_encoding_info_single_header() -> Vec<EncodingInfo> {
    vec![EncodingInfo::new(
        0,
        vec![
            ContentEncoding::Brotli,
            ContentEncoding::Deflate,
            ContentEncoding::Identity,
            ContentEncoding::Gzip,
            ContentEncoding::Zstd,
        ],
    )]
}

pub fn compress_brotli(data: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    {
        let mut writer =
            brotli::CompressorWriter::new(&mut compressed, 4096, 0, 22);
        writer.write_all(data).unwrap();
        writer.flush().unwrap();
    }
    compressed
}

pub fn compress_deflate(data: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    let mut encoder =
        flate2::write::ZlibEncoder::new(&mut compressed, Compression::fast());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap();
    compressed
}

pub fn compress_gzip(data: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    let mut encoder =
        flate2::write::GzEncoder::new(&mut compressed, Compression::fast());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap();
    compressed
}

pub fn compress_zstd(data: &[u8]) -> Vec<u8> {
    zstd::encode_all(data, 1).unwrap()
}
