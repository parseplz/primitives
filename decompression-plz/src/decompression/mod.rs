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
) -> Result<u64, DecompressError> {
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
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use bytes::{BufMut, BytesMut};
    use flate2::Compression;
    use header_plz::body_headers::content_encoding::ContentEncoding;

    use crate::decompression::decompress;

    const INPUT: &[u8] = b"hello world";

    pub fn all_compressed_data() -> Vec<u8> {
        let level = 0;
        let data = b"hello world";
        let brotli_compressed = compress_brotli(data);
        let deflate_compressed = compress_deflate(&brotli_compressed);
        let gzip_compressed = compress_gzip(&deflate_compressed);
        compress_zstd(&gzip_compressed)
    }

    pub fn compress_brotli(data: &[u8]) -> Vec<u8> {
        let mut compressed = Vec::new();
        {
            let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, 0, 22);
            writer.write_all(data).unwrap();
            writer.flush().unwrap();
        }
        compressed
    }

    pub fn compress_deflate(data: &[u8]) -> Vec<u8> {
        let mut compressed = Vec::new();
        let mut encoder = flate2::read::DeflateEncoder::new(&data[..], flate2::Compression::fast());
        //let mut encoder = flate2::write::ZlibEncoder::new(&mut compressed, Compression::fast());
        encoder.read_to_end(&mut compressed).unwrap();
        compressed
    }

    pub fn compress_gzip(data: &[u8]) -> Vec<u8> {
        let mut compressed = Vec::new();
        let mut encoder = flate2::write::GzEncoder::new(&mut compressed, Compression::fast());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap();
        compressed
    }

    pub fn compress_zstd(data: &[u8]) -> Vec<u8> {
        zstd::encode_all(data, 1).unwrap()
    }

    fn test_decompress(data: &[u8], content_encoding: ContentEncoding) -> BytesMut {
        let compressed = match content_encoding {
            ContentEncoding::Brotli => compress_brotli(data),
            ContentEncoding::Deflate => compress_deflate(data),
            ContentEncoding::Gzip => compress_gzip(data),
            ContentEncoding::Zstd | ContentEncoding::Compress => compress_zstd(data),
            ContentEncoding::Identity => data.to_vec(),
            _ => panic!(),
        };
        let mut buf = BytesMut::new();
        let mut writer = buf.writer();
        decompress(compressed.as_slice(), &mut writer, content_encoding).unwrap();
        writer.into_inner()
    }

    #[test]
    fn test_basic_brotli() {
        let result = test_decompress(INPUT, ContentEncoding::Brotli);
        assert_eq!(result.as_ref(), INPUT);
    }

    #[test]
    fn test_basic_deflate() {
        let result = test_decompress(INPUT, ContentEncoding::Deflate);
        assert_eq!(result.as_ref(), INPUT);
    }

    #[test]
    fn test_basic_gzip() {
        let result = test_decompress(INPUT, ContentEncoding::Gzip);
        assert_eq!(result.as_ref(), INPUT);
    }

    #[test]
    fn test_basic_zstd() {
        let result = test_decompress(INPUT, ContentEncoding::Zstd);
        assert_eq!(result.as_ref(), INPUT);
    }

    #[test]
    fn test_basic_compress() {
        let result = test_decompress(INPUT, ContentEncoding::Compress);
        assert_eq!(result.as_ref(), INPUT);
    }

    #[test]
    fn test_basic_identity() {
        let result = test_decompress(INPUT, ContentEncoding::Identity);
        assert_eq!(result.as_ref(), INPUT);
    }
}
