use bytes::{BytesMut, buf::Writer};
use header_plz::body_headers::encoding_info::EncodingInfo;

use crate::decompression::single::decompress_single;

pub mod error;
use error::*;

pub fn decompress_multi<'a, T>(
    compressed: &[u8],
    mut writer: &mut Writer<&mut BytesMut>,
    encoding_info: T,
) -> Result<BytesMut, MultiDecompressError>
where
    T: Iterator<Item = &'a EncodingInfo> + std::iter::DoubleEndedIterator,
{
    let mut input: &[u8] = compressed;
    let mut output: BytesMut = writer.get_mut().split();

    for (header_index, encoding_info) in encoding_info.rev().enumerate() {
        for (compression_index, encoding) in
            encoding_info.encodings().iter().rev().enumerate()
        {
            let curs = std::io::Cursor::new(&mut input);
            let result = decompress_single(curs, &mut writer, encoding);
            match result {
                Ok(_) => {
                    output = writer.get_mut().split();
                    input = &output[..];
                }
                Err(e) => {
                    let reason = if header_index == 0 && compression_index == 0
                    {
                        MultiDecompressErrorReason::Corrupt
                    } else {
                        writer.get_mut().clear();
                        std::io::copy(&mut input, writer)?;
                        output = writer.get_mut().split();
                        MultiDecompressErrorReason::Partial {
                            partial_body: output,
                            header_index,
                            compression_index,
                        }
                    };
                    return Err(MultiDecompressError::new(reason, e));
                }
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tests::*;
    use bytes::BufMut;
    use header_plz::body_headers::content_encoding::ContentEncoding;

    #[test]
    fn test_decompress_multi_single_header() {
        let input = all_compressed_data();
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = [EncodingInfo::new(
            0,
            vec![
                ContentEncoding::Brotli,
                ContentEncoding::Deflate,
                ContentEncoding::Gzip,
                ContentEncoding::Zstd,
                ContentEncoding::Identity,
            ],
        )];
        let result =
            decompress_multi(&input, &mut writer, &mut einfo_list.iter())
                .unwrap();
        assert_eq!(result, INPUT);
    }

    #[test]
    fn test_decompress_multi_multi_header() {
        let input = all_compressed_data();
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = [
            EncodingInfo::new(0, vec![ContentEncoding::Brotli]),
            EncodingInfo::new(1, vec![ContentEncoding::Deflate]),
            EncodingInfo::new(2, vec![ContentEncoding::Gzip]),
            EncodingInfo::new(3, vec![ContentEncoding::Zstd]),
            EncodingInfo::new(4, vec![ContentEncoding::Identity]),
        ];

        let result =
            decompress_multi(&input, &mut writer, &mut einfo_list.iter())
                .unwrap();
        assert_eq!(result, INPUT);
    }

    #[test]
    fn test_decompress_multi_multi_header_split() {
        let input = all_compressed_data();
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = [
            EncodingInfo::new(
                0,
                vec![ContentEncoding::Brotli, ContentEncoding::Deflate],
            ),
            EncodingInfo::new(
                2,
                vec![ContentEncoding::Gzip, ContentEncoding::Identity],
            ),
            EncodingInfo::new(
                3,
                vec![ContentEncoding::Zstd, ContentEncoding::Identity],
            ),
        ];

        let result =
            decompress_multi(&input, &mut writer, &mut einfo_list.iter())
                .unwrap();
        assert_eq!(result, INPUT);
    }

    #[test]
    fn test_decompress_multi_error_partial_single_header() {
        let input = compress_brotli(INPUT);
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = [EncodingInfo::new(
            0,
            vec![ContentEncoding::Deflate, ContentEncoding::Brotli],
        )];
        let result =
            decompress_multi(&input, &mut writer, &mut einfo_list.iter())
                .unwrap_err();
        if let MultiDecompressErrorReason::Partial {
            partial_body,
            header_index,
            compression_index,
        } = result.reason
        {
            assert_eq!(header_index, 0);
            assert_eq!(compression_index, 1);
            assert_eq!(partial_body.as_ref(), INPUT);
        }
    }

    #[test]
    fn test_decompress_multi_error_partial_single_header_all_compression() {
        let input = all_compressed_data();
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = [EncodingInfo::new(
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
        let result =
            decompress_multi(&input, &mut writer, &mut einfo_list.iter())
                .unwrap_err();
        if let MultiDecompressErrorReason::Partial {
            partial_body,
            header_index,
            compression_index,
        } = result.reason
        {
            assert_eq!(header_index, 0);
            assert_eq!(compression_index, 5);
            assert_eq!(partial_body.as_ref(), INPUT);
        }
    }

    #[test]
    fn test_decompress_multi_error_partial_multi_header() {
        let input = compress_brotli(INPUT);
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = [
            EncodingInfo::new(0, vec![ContentEncoding::Zstd]),
            EncodingInfo::new(4, vec![ContentEncoding::Brotli]),
        ];
        let result =
            decompress_multi(&input, &mut writer, &mut einfo_list.iter())
                .unwrap_err();
        if let MultiDecompressErrorReason::Partial {
            partial_body,
            header_index,
            compression_index,
        } = result.reason
        {
            assert_eq!(header_index, 1);
            assert_eq!(compression_index, 0);
            assert_eq!(partial_body.as_ref(), INPUT);
        }
    }

    #[test]
    fn test_decompress_multi_error_partial_multi_header_all_compression() {
        let input = all_compressed_data();
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = [
            EncodingInfo::new(0, vec![ContentEncoding::Brotli]),
            EncodingInfo::new(1, vec![ContentEncoding::Brotli]),
            EncodingInfo::new(2, vec![ContentEncoding::Deflate]),
            EncodingInfo::new(3, vec![ContentEncoding::Gzip]),
            EncodingInfo::new(4, vec![ContentEncoding::Zstd]),
            EncodingInfo::new(5, vec![ContentEncoding::Identity]),
        ];
        let result =
            decompress_multi(&input, &mut writer, &mut einfo_list.iter())
                .unwrap_err();
        if let MultiDecompressErrorReason::Partial {
            partial_body,
            header_index,
            compression_index,
        } = result.reason
        {
            assert_eq!(header_index, 5);
            assert_eq!(compression_index, 0);
            assert_eq!(partial_body.as_ref(), INPUT);
        }
    }

    #[test]
    fn test_decompress_multi_error_corrupt_single_header() {
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = [EncodingInfo::new(0, vec![ContentEncoding::Zstd])];
        let result =
            decompress_multi(INPUT, &mut writer, &mut einfo_list.iter())
                .unwrap_err();
        assert!(matches!(result.reason, MultiDecompressErrorReason::Corrupt));
    }

    #[test]
    fn test_decompress_multi_error_corrupt_multi_header() {
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = [
            EncodingInfo::new(0, vec![ContentEncoding::Zstd]),
            EncodingInfo::new(4, vec![ContentEncoding::Brotli]),
        ];

        let result =
            decompress_multi(INPUT, &mut writer, &mut einfo_list.iter())
                .unwrap_err();
        assert!(matches!(result.reason, MultiDecompressErrorReason::Corrupt));
    }
}
