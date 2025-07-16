use brotli::Decompressor;
use header_plz::body_headers::{content_encoding::ContentEncoding, encoding_info::EncodingInfo};

mod magic_bytes;
use thiserror::Error;

pub mod multi;
pub mod single;

/*

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
