use bytes::{BytesMut, buf::Writer};
use header_plz::body_headers::encoding_info::EncodingInfo;

use crate::{
    decompression::single::decompress,
    error::{DecompressErrorStruct, Reason},
};

mod error;

pub fn decompress_multi(
    mut compressed: &[u8],
    mut writer: &mut Writer<&mut BytesMut>,
    encoding_info: &[EncodingInfo],
) -> Result<BytesMut, DecompressErrorStruct> {
    let mut input: &[u8] = compressed;
    let mut output: BytesMut = writer.get_mut().split();

    for (header_index, encoding_info) in encoding_info.iter().rev().enumerate() {
        for (compression_index, encoding) in encoding_info.encodings().iter().rev().enumerate() {
            let result = decompress(&mut input, &mut writer, encoding.clone());
            match result {
                Ok(_) => {
                    output = writer.get_mut().split();
                    input = &output[..];
                }
                Err(e) => {
                    let reason = if header_index == 0 && compression_index == 0 {
                        Reason::Corrupt
                    } else {
                        Reason::PartialCorrupt(header_index, compression_index)
                    };

                    let body = match reason {
                        Reason::Corrupt => None,
                        Reason::PartialCorrupt(_, _) => {
                            writer.get_mut().clear();
                            std::io::copy(&mut input, writer).unwrap();
                            output = writer.get_mut().split();
                            Some(output)
                        }
                    };

                    todo!()
                    //return Err(DecompressErrorStruct::new(output, None, e, reason));
                }
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {

    use bytes::BufMut;
    use header_plz::body_headers::content_encoding::ContentEncoding;

    use crate::decompression::single::tests::*;

    use super::*;

    pub fn all_compressed_data() -> Vec<u8> {
        let brotli_compressed = compress_brotli(INPUT);
        let deflate_compressed = compress_deflate(&brotli_compressed);
        let gzip_compressed = compress_gzip(&deflate_compressed);
        compress_zstd(&gzip_compressed)
    }

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

    /*
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
    */
}
