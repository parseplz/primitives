use brotli::Decompressor;
use header_plz::body_headers::{content_encoding::ContentEncoding, encoding_info::EncodingInfo};

mod magic_bytes;
use thiserror::Error;

pub mod multi;
pub mod single;

/*
#[cfg(test)]
pub mod tests {
    use crate::{
        decompression::{
            decompress, decompress_multi,
            decompressors::{decompress_brotli, decompress_deflate},
            error::DecompressError,
        },
        error::Reason,
    };
    use bytes::{BufMut, BytesMut};
    use flate2::Compression;
    use header_plz::body_headers::{
        content_encoding::ContentEncoding, encoding_info::EncodingInfo,
    };
    use std::io::{Read, Write};

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

    // Combined tests
    #[test]
    fn test_decompress_all_single_header() {
        let input = all_compressed_data();
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = vec![EncodingInfo::new(
            0,
            vec![
                ContentEncoding::Brotli,
                ContentEncoding::Deflate,
                ContentEncoding::Gzip,
                ContentEncoding::Zstd,
                ContentEncoding::Identity,
            ],
        )];
        let result = decompress_multi(&input, &mut writer, &einfo_list).unwrap();
        assert_eq!(result, INPUT);
    }

    #[test]
    fn test_decompress_all_multi_header() {
        let input = all_compressed_data();
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = vec![
            EncodingInfo::new(0, vec![ContentEncoding::Brotli]),
            EncodingInfo::new(1, vec![ContentEncoding::Deflate]),
            EncodingInfo::new(2, vec![ContentEncoding::Gzip]),
            EncodingInfo::new(3, vec![ContentEncoding::Zstd]),
            EncodingInfo::new(4, vec![ContentEncoding::Identity]),
        ];

        let result = decompress_multi(&input, &mut writer, &einfo_list).unwrap();
        assert_eq!(result, INPUT);
    }

    #[test]
    fn test_decompress_all_multi_header_split() {
        let input = all_compressed_data();
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = vec![
            EncodingInfo::new(0, vec![ContentEncoding::Brotli, ContentEncoding::Deflate]),
            EncodingInfo::new(2, vec![ContentEncoding::Gzip, ContentEncoding::Identity]),
            EncodingInfo::new(3, vec![ContentEncoding::Zstd, ContentEncoding::Identity]),
        ];

        let result = decompress_multi(&input, &mut writer, &einfo_list).unwrap();
        assert_eq!(result, INPUT);
    }

    #[test]
    fn test_decompress_all_error_single_header() {
        let input = compress_brotli(INPUT);
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = vec![EncodingInfo::new(
            0,
            vec![ContentEncoding::Deflate, ContentEncoding::Brotli],
        )];
        let result = decompress_multi(&input, &mut writer, &einfo_list).unwrap_err();
        assert_eq!(*result.reason(), Reason::PartialCorrupt(0, 1));
        let (body, extra_body) = result.into_body_and_extra();
        assert_eq!(body.as_ref(), INPUT);
        assert!(extra_body.is_none());
    }

    #[test]
    fn test_decompress_all_error_single_header_all_compression() {
        let input = all_compressed_data();
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = vec![EncodingInfo::new(
            0,
            vec![
                ContentEncoding::Compress,
                ContentEncoding::Brotli,
                ContentEncoding::Deflate,
                ContentEncoding::Gzip,
                ContentEncoding::Zstd,
                ContentEncoding::Identity,
            ],
        )];
        let result = decompress_multi(&input, &mut writer, &einfo_list).unwrap_err();
        assert_eq!(*result.reason(), Reason::PartialCorrupt(0, 5));
        let (body, extra_body) = result.into_body_and_extra();
        assert_eq!(body.as_ref(), INPUT);
        assert!(extra_body.is_none());
    }

    #[test]
    fn test_decompress_all_error_multi_header() {
        let input = compress_brotli(INPUT);
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = vec![
            EncodingInfo::new(0, vec![ContentEncoding::Zstd]),
            EncodingInfo::new(4, vec![ContentEncoding::Brotli]),
        ];
        let result = decompress_multi(&input, &mut writer, &einfo_list).unwrap_err();
        assert_eq!(*result.reason(), Reason::PartialCorrupt(1, 0));
        let (body, extra_body) = result.into_body_and_extra();
        assert_eq!(body.as_ref(), INPUT);
        assert!(extra_body.is_none());
    }

    #[test]
    fn test_decompress_all_error_multi_header_all_compression() {
        let input = all_compressed_data();
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = vec![
            EncodingInfo::new(0, vec![ContentEncoding::Brotli]),
            EncodingInfo::new(1, vec![ContentEncoding::Brotli]),
            EncodingInfo::new(2, vec![ContentEncoding::Deflate]),
            EncodingInfo::new(3, vec![ContentEncoding::Gzip]),
            EncodingInfo::new(4, vec![ContentEncoding::Zstd]),
            EncodingInfo::new(5, vec![ContentEncoding::Identity]),
        ];
        let result = decompress_multi(&input, &mut writer, &einfo_list).unwrap_err();
        assert_eq!(*result.reason(), Reason::PartialCorrupt(5, 0));
        let (body, extra_body) = result.into_body_and_extra();
        assert_eq!(body.as_ref(), INPUT);
        assert!(extra_body.is_none());
    }

    #[test]
    fn test_decompress_all_error_corrupt() {
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = vec![
            EncodingInfo::new(0, vec![ContentEncoding::Zstd]),
            EncodingInfo::new(4, vec![ContentEncoding::Brotli]),
        ];
        let result = decompress_multi(INPUT, &mut writer, &einfo_list).unwrap_err();
        assert_eq!(*result.reason(), Reason::Corrupt);
        let (body, extra_body) = result.into_body_and_extra();
        assert_eq!(body.as_ref(), INPUT);
        assert!(extra_body.is_none());
    }
}
*/
