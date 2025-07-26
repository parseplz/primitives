use std::io::Read;
use std::io::Write;
use std::io::copy;

use header_plz::body_headers::content_encoding::ContentEncoding;
pub mod error;
use error::DecompressError;

pub fn decompress_single<R, W>(
    mut input: R,
    mut writer: W,
    content_encoding: &ContentEncoding,
) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    match content_encoding {
        ContentEncoding::Brotli => decompress_brotli(input, writer),
        ContentEncoding::Compress | ContentEncoding::Zstd => {
            decompress_zstd(input, writer)
        }
        ContentEncoding::Deflate => decompress_deflate(input, writer),
        ContentEncoding::Gzip => decompress_gzip(input, writer),
        ContentEncoding::Identity | ContentEncoding::Chunked => {
            copy(&mut input, &mut writer).map_err(DecompressError::Identity)
        }
        ContentEncoding::Unknown(e) => {
            Err(DecompressError::Unknown(e.to_string()))
        }
    }
}

#[inline]
pub fn decompress_brotli<R, W>(
    input: R,
    mut buf: W,
) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    copy(&mut brotli::Decompressor::new(input, 4096), &mut buf)
        .map_err(DecompressError::Brotli)
}

#[inline]
pub fn decompress_deflate<R, W>(
    input: R,
    mut buf: W,
) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    copy(&mut flate2::read::ZlibDecoder::new(input), &mut buf)
        .map_err(DecompressError::Deflate)
}

#[inline]
pub fn decompress_gzip<R, W>(
    input: R,
    mut buf: W,
) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    copy(&mut flate2::read::GzDecoder::new(input), &mut buf)
        .map_err(DecompressError::Gzip)
}

#[inline]
pub fn decompress_zstd<R, W>(
    input: R,
    mut buf: W,
) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    copy(
        &mut zstd::stream::read::Decoder::new(input)
            .map_err(DecompressError::Zstd)?,
        &mut buf,
    )
    .map_err(DecompressError::Zstd)
}

#[cfg(test)]
pub mod tests {
    use bytes::{BufMut, BytesMut};

    use header_plz::body_headers::content_encoding::ContentEncoding;
    use tests_utils::*;

    use crate::decompression::single::{
        decompress_single, error::DecompressError,
    };

    fn test_decompress(
        data: &[u8],
        content_encoding: ContentEncoding,
    ) -> BytesMut {
        let compressed = match content_encoding {
            ContentEncoding::Brotli => compress_brotli(data),
            ContentEncoding::Deflate => compress_deflate(data),
            ContentEncoding::Gzip => compress_gzip(data),
            ContentEncoding::Zstd | ContentEncoding::Compress => {
                compress_zstd(data)
            }
            ContentEncoding::Identity
            | ContentEncoding::Unknown(_)
            | ContentEncoding::Chunked => data.to_vec(),
        };
        let buf = BytesMut::new();
        let mut writer = buf.writer();
        decompress_single(
            compressed.as_slice(),
            &mut writer,
            &content_encoding,
        )
        .unwrap();
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
        assert_eq!(result.as_ref(), INPUT);
    }

    #[test]
    fn test_basic_unknown() {
        let buf = BytesMut::new();
        let mut writer = buf.writer();
        let result = decompress_single(
            INPUT,
            &mut writer,
            &ContentEncoding::Unknown("unknown".to_string()),
        )
        .unwrap_err();
        if let DecompressError::Unknown(e) = result {
            assert_eq!(e, "unknown");
        } else {
            panic!();
        }
    }
}
