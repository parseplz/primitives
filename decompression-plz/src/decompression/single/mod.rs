use std::io::Read;
use std::io::Write;
use std::io::copy;

use header_plz::body_headers::content_encoding::ContentEncoding;
pub mod error;
use error::DecompressError;

pub fn decompress<R, W>(
    mut input: R,
    mut writer: W,
    content_encoding: ContentEncoding,
) -> Result<u64, DecompressError>
where
    R: Read + AsRef<[u8]>,
    W: Write,
{
    let mut input = std::io::Cursor::new(input);
    match content_encoding {
        ContentEncoding::Brotli => decompress_brotli(input, writer),
        ContentEncoding::Compress | ContentEncoding::Zstd => decompress_zstd(input, writer),
        ContentEncoding::Deflate => decompress_deflate(input, writer),
        ContentEncoding::Gzip => decompress_gzip(input, writer),
        ContentEncoding::Identity => {
            copy(&mut input, &mut writer).map_err(DecompressError::Identity)
        }
        ContentEncoding::Chunked => Ok(0),
        ContentEncoding::Unknown(e) => Err(DecompressError::Unknown(e.to_string())),
    }
}

pub fn decompress_brotli<R, W>(input: R, mut buf: W) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    copy(&mut brotli::Decompressor::new(input, 4096), &mut buf).map_err(DecompressError::Brotli)
}

pub fn decompress_deflate<R, W>(input: R, mut buf: W) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    copy(&mut flate2::read::ZlibDecoder::new(input), &mut buf).map_err(DecompressError::Deflate)
}

pub fn decompress_gzip<R, W>(input: R, mut buf: W) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    copy(&mut flate2::read::GzDecoder::new(input), &mut buf).map_err(DecompressError::Gzip)
}

pub fn decompress_zstd<R, W>(input: R, mut buf: W) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    //zstd::stream::copy_decode(input, &mut buf)?;
    //Ok(0)
    // -----
    copy(
        &mut zstd::stream::read::Decoder::new(input).map_err(DecompressError::Zstd)?,
        &mut buf,
    )
    .map_err(DecompressError::Zstd)
}

#[cfg(test)]
pub mod tests {
    use bytes::{BufMut, BytesMut};
    use flate2::Compression;
    use header_plz::body_headers::{
        content_encoding::ContentEncoding, encoding_info::EncodingInfo,
    };
    use std::io::{Read, Write};

    use crate::decompression::single::{decompress, error::DecompressError};

    pub const INPUT: &[u8] = b"hello world";

    pub fn all_compressed_data() -> Vec<u8> {
        let brotli_compressed = compress_brotli(INPUT);
        let deflate_compressed = compress_deflate(&brotli_compressed);
        let gzip_compressed = compress_gzip(&deflate_compressed);
        compress_zstd(&gzip_compressed)
    }

    pub fn compressed_data() -> Vec<u8> {
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
        let mut encoder = flate2::write::ZlibEncoder::new(&mut compressed, Compression::fast());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap();
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
            ContentEncoding::Identity | ContentEncoding::Unknown(_) | ContentEncoding::Chunked => {
                data.to_vec()
            }
            _ => panic!(),
        };
        let mut buf = BytesMut::new();
        let mut writer = buf.writer();
        decompress(compressed.as_slice(), &mut writer, content_encoding).unwrap();
        writer.into_inner()
    }

    // Individual tests
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

    #[test]
    fn test_basic_chunked() {
        let result = test_decompress(INPUT, ContentEncoding::Chunked);
        assert_eq!(result.as_ref(), b"");
    }

    #[test]
    fn test_basic_unknown() {
        let mut buf = BytesMut::new();
        let mut writer = buf.writer();
        let result = decompress(
            INPUT,
            &mut writer,
            ContentEncoding::Unknown("unknown".to_string()),
        )
        .unwrap_err();
        if let DecompressError::Unknown(e) = result {
            assert_eq!(e, "unknown");
        } else {
            panic!();
        }
    }
}
