use std::io::{Cursor, Read, Write};

use bytes::{BufMut, BytesMut, buf::Writer};
use header_plz::body_headers::{
    content_encoding::ContentEncoding, encoding_info::EncodingInfo,
};
use tracing::{instrument, trace};

use crate::decompression::{
    magic_bytes::is_compressed,
    multi::{
        decompress_multi,
        error::{MultiDecompressError, MultiDecompressErrorReason},
    },
    single::decompress_single,
};

pub struct DecompressionStruct<'a> {
    pub main: &'a [u8],
    pub extra: Option<&'a [u8]>,
    pub encoding_info: &'a mut [EncodingInfo],
    pub writer: Writer<&'a mut BytesMut>,
}

impl<'a> DecompressionStruct<'a> {
    pub fn new(
        main: &'a [u8],
        extra: Option<&'a [u8]>,
        encoding_info: &'a mut [EncodingInfo],
        writer: Writer<&'a mut BytesMut>,
    ) -> Self {
        Self {
            main,
            extra,
            encoding_info,
            writer,
        }
    }

    pub fn last_encoding(&self) -> &ContentEncoding {
        self.encoding_info.last().unwrap().encodings().last().unwrap()
    }

    pub fn pop_last_encoding(&mut self) -> ContentEncoding {
        self.encoding_info
            .last_mut()
            .unwrap()
            .encodings_as_mut()
            .pop()
            .unwrap()
    }

    pub fn push_last_encoding(&mut self, encoding: ContentEncoding) {
        self.encoding_info
            .last_mut()
            .unwrap()
            .encodings_as_mut()
            .push(encoding);
    }

    pub fn is_encodings_empty(&self) -> bool {
        match self.encoding_info.split_last() {
            Some((last, rest)) if rest.is_empty() => {
                last.encodings().is_empty()
            }
            _ => false,
        }
    }

    pub fn len(&self) -> usize {
        self.main.len() + self.extra.as_ref().map(|e| e.len()).unwrap_or(0)
    }

    pub fn last_compression_index(&self) -> usize {
        let (last, rest) = self.encoding_info.split_last().unwrap();
        let target_encs = if !last.encodings().is_empty() {
            last.encodings()
        } else if let Some(last_before) = rest.last() {
            last_before.encodings()
        } else {
            &[]
        };
        target_encs.len()
    }

    pub fn extra(&self) -> &[u8] {
        self.extra.as_ref().unwrap().as_ref()
    }

    pub fn is_extra_compressed(&self) -> bool {
        is_compressed(self.extra(), self.last_encoding())
    }

    pub fn try_decompress_extra(
        &mut self,
    ) -> Result<BytesMut, MultiDecompressError> {
        decompress_multi(
            self.extra.as_ref().unwrap().as_ref(),
            &mut self.writer,
            &mut self.encoding_info.iter(),
        )
    }

    pub fn try_decompress_main(
        &mut self,
    ) -> Result<BytesMut, MultiDecompressError> {
        decompress_multi(
            self.main.as_ref(),
            &mut self.writer,
            &mut self.encoding_info.iter(),
        )
    }

    pub fn try_decompress_main_plus_extra(
        &mut self,
    ) -> Result<BytesMut, MultiDecompressError> {
        let last_encoding = self.pop_last_encoding();
        let mut chained = Cursor::new(self.main.as_ref())
            .chain(Cursor::new(self.extra.as_ref().unwrap().as_ref()));
        let len = self.len() as u64;
        let result = Self::try_decompress_chain(
            chained,
            &mut self.writer,
            &last_encoding,
            len,
        );
        if result.is_err() {
            self.push_last_encoding(last_encoding);
            return Err(result.unwrap_err());
        }
        let mut output = self.writer.get_mut().split();
        if !self.is_encodings_empty() {
            self.try_decompress_chain_remaining(output, last_encoding)
        } else {
            self.push_last_encoding(last_encoding);
            Ok(output)
        }
    }

    // Errors:
    //      1. Copy
    //      2. Corrupt - Deflate
    //      3. Corrupt - Others
    //      4. ExtraRaw
    fn try_decompress_chain(
        mut input: std::io::Chain<Cursor<&[u8]>, Cursor<&[u8]>>,
        mut writer: &mut Writer<&mut BytesMut>,
        content_encoding: &ContentEncoding,
        len: u64,
    ) -> Result<(), MultiDecompressError> {
        if let ContentEncoding::Deflate = content_encoding {
            let mut reader = flate2::read::ZlibDecoder::new(input);
            std::io::copy(&mut reader, &mut writer)?;
            if reader.total_in() != len {
                return Err(MultiDecompressError::extra_raw());
            }
            return Ok(());
        }
        decompress_single(&mut input, &mut writer, &content_encoding)
            .map_err(|_| MultiDecompressError::extra_raw())?;
        if let (_, extra_curs) = input.get_ref()
            && extra_curs.position() == 0
        {
            return Err(MultiDecompressError::extra_raw());
        }
        Ok(())
    }

    fn try_decompress_chain_remaining(
        &mut self,
        mut input: BytesMut,
        last_encoding: ContentEncoding,
    ) -> Result<BytesMut, MultiDecompressError> {
        let iter = &mut self
            .encoding_info
            .iter()
            .filter(|einfo| !einfo.encodings().is_empty());
        decompress_multi(&input, &mut self.writer, iter).map_err(|e| {
            self.push_last_encoding(last_encoding);
            if e.is_corrupt() {
                let header_index = self.encoding_info.len() - 1;
                let compression_index = self.last_compression_index() - 1;
                let partial_error = e.from_corrupt_to_partial(
                    input,
                    header_index,
                    compression_index,
                );
                return partial_error;
            }
            e
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        decompression::single::error::DecompressError,
        tests::{
            INPUT, all_encoding_info_multi_header,
            all_encoding_info_single_header, compress_brotli,
            compress_deflate, compress_gzip, compress_zstd,
        },
    };

    use super::*;

    // last_encoding() + pop_last_encoding()
    #[test]
    fn test_dstruct_last_encoding_single_value() {
        let mut encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Gzip])];
        let mut buf = BytesMut::new();
        let mut dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(dstruct.last_encoding(), &ContentEncoding::Gzip);
        assert_eq!(dstruct.pop_last_encoding(), ContentEncoding::Gzip);
    }

    #[test]
    fn test_dstruct_last_encoding_single_header() {
        let mut encoding_info = all_encoding_info_single_header();
        let mut buf = BytesMut::new();
        let mut dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(dstruct.last_encoding(), &ContentEncoding::Zstd);
        assert_eq!(dstruct.pop_last_encoding(), ContentEncoding::Zstd);
    }

    #[test]
    fn test_dstruct_last_encoding_multi_header() {
        let mut encoding_info = all_encoding_info_multi_header();
        let mut buf = BytesMut::new();
        let mut dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(dstruct.last_encoding(), &ContentEncoding::Zstd);
        assert_eq!(dstruct.pop_last_encoding(), ContentEncoding::Zstd);
    }

    // push_last_encoding
    #[test]
    fn test_dstruct_push_last_encoding_single_value() {
        let mut encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Gzip])];

        let mut buf = BytesMut::new();
        let mut dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        dstruct.push_last_encoding(ContentEncoding::Brotli);
        let to_verify = vec![EncodingInfo::new(
            0,
            vec![ContentEncoding::Gzip, ContentEncoding::Brotli],
        )];

        assert_eq!(dstruct.encoding_info, to_verify);
    }

    #[test]
    fn test_push_last_encoding_single_header() {
        let mut encoding_info = all_encoding_info_single_header();
        let mut buf = BytesMut::new();
        let mut dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        dstruct.push_last_encoding(ContentEncoding::Brotli);
        let to_verify = vec![EncodingInfo::new(
            0,
            vec![
                ContentEncoding::Brotli,
                ContentEncoding::Deflate,
                ContentEncoding::Identity,
                ContentEncoding::Gzip,
                ContentEncoding::Zstd,
                ContentEncoding::Brotli,
            ],
        )];

        assert_eq!(dstruct.encoding_info, to_verify);
    }

    #[test]
    fn test_push_last_encoding_multi_header() {
        let mut encoding_info = all_encoding_info_multi_header();
        let mut buf = BytesMut::new();
        let mut dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        dstruct.push_last_encoding(ContentEncoding::Brotli);
        let to_verify = vec![
            EncodingInfo::new(0, vec![ContentEncoding::Brotli]),
            EncodingInfo::new(1, vec![ContentEncoding::Deflate]),
            EncodingInfo::new(2, vec![ContentEncoding::Identity]),
            EncodingInfo::new(3, vec![ContentEncoding::Gzip]),
            EncodingInfo::new(
                4,
                vec![ContentEncoding::Zstd, ContentEncoding::Brotli],
            ),
        ];

        assert_eq!(dstruct.encoding_info, to_verify);
    }

    // is_encodings_empty
    #[test]
    fn test_dstruct_is_encodings_empty_true_single_value() {
        let mut encoding_info = vec![EncodingInfo::new(0, vec![])];
        let mut buf = BytesMut::new();
        let dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert!(dstruct.is_encodings_empty());
    }

    #[test]
    fn test_dstruct_is_encodings_empty_false_single_value() {
        let mut encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Gzip])];
        let mut buf = BytesMut::new();
        let dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert!(!dstruct.is_encodings_empty());
    }

    #[test]
    fn test_dstruct_is_encodings_empty_false_last_encoding_empty() {
        let mut encoding_info = vec![
            EncodingInfo::new(0, vec![ContentEncoding::Gzip]),
            EncodingInfo::new(1, vec![]),
        ];
        let mut buf = BytesMut::new();
        let dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert!(!dstruct.is_encodings_empty());
    }

    // len()
    #[test]
    fn test_dstruct_len_empty() {
        let mut encoding_info = vec![];
        let mut buf = BytesMut::new();
        let dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(dstruct.len(), 0);
    }

    #[test]
    fn test_dstruct_len_only_main() {
        let mut encoding_info = vec![];
        let mut buf = BytesMut::new();
        let dstruct = DecompressionStruct::new(
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(dstruct.len(), 10);
    }

    #[test]
    fn test_dstruct_len_main_and_extra() {
        let mut encoding_info = vec![];
        let mut buf = BytesMut::new();
        let dstruct = DecompressionStruct::new(
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            Some(&[11, 12, 13, 14, 15, 16, 17, 18, 19, 20]),
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(dstruct.len(), 20);
    }

    // last_compression_index
    #[test]
    fn test_dstruct_last_compression_index_no_value() {
        let mut encoding_info = vec![EncodingInfo::new(0, vec![])];
        let mut buf = BytesMut::new();
        let dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(dstruct.last_compression_index(), 0);
    }

    #[test]
    fn test_dstruct_last_compression_index_sigle_value() {
        let mut encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Gzip])];
        let mut buf = BytesMut::new();
        let dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(dstruct.last_compression_index(), 1);
    }

    #[test]
    fn test_dstruct_last_compression_index_multi_value_single_header() {
        let mut encoding_info = all_encoding_info_single_header();
        let mut buf = BytesMut::new();
        let dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(dstruct.last_compression_index(), 5);
    }

    #[test]
    fn test_dstruct_last_compression_index_multi_value_multi_header() {
        let mut encoding_info = all_encoding_info_multi_header();
        let mut buf = BytesMut::new();
        let dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(dstruct.last_compression_index(), 1);
    }

    #[test]
    fn test_dstruct_last_compression_index_last_empty() {
        let mut encoding_info = vec![
            EncodingInfo::new(
                0,
                vec![ContentEncoding::Gzip, ContentEncoding::Brotli],
            ),
            EncodingInfo::new(1, vec![]),
        ];
        let mut buf = BytesMut::new();
        let dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(dstruct.last_compression_index(), 2);
    }

    // Main + Extra - errors
    // try_decompress_chain
    #[test]
    fn test_dstruct_decompress_main_plus_extra_error_extra_raw_deflate() {
        let mut compressed = compress_deflate(INPUT);
        let mut encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Deflate])];
        let mut to_check_encoding_info = encoding_info.clone();
        let mut buf = BytesMut::new();
        let mut dstruct = DecompressionStruct::new(
            compressed.as_ref(),
            Some(&INPUT),
            &mut encoding_info,
            (&mut buf).writer(),
        );

        let result = dstruct.try_decompress_main_plus_extra();
        if let Err(e) = result {
            assert_eq!(e.reason, MultiDecompressErrorReason::ExtraRaw);
            assert!(matches!(e.error, DecompressError::Copy(_)));
            assert_eq!(dstruct.encoding_info, to_check_encoding_info);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_dstruct_decompress_main_plus_extra_error_extra_raw_brotli() {
        let mut compressed = compress_brotli(INPUT);
        let mut encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Brotli])];
        let to_check_encoding_info = encoding_info.clone();
        let mut buf = BytesMut::new();
        let mut dstruct = DecompressionStruct::new(
            compressed.as_ref(),
            Some(&INPUT),
            &mut encoding_info,
            (&mut buf).writer(),
        );
        let result = dstruct.try_decompress_main_plus_extra();
        if let Err(e) = result {
            assert_eq!(e.reason, MultiDecompressErrorReason::ExtraRaw);
            assert!(matches!(e.error, DecompressError::Copy(_)));
            assert_eq!(dstruct.encoding_info, to_check_encoding_info);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_dstruct_decompress_main_plus_extra_error_extra_raw_gzip() {
        let mut compressed = compress_gzip(INPUT);
        let mut encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Gzip])];
        let to_check_encoding_info = encoding_info.clone();
        let mut buf = BytesMut::new();
        let mut dstruct = DecompressionStruct::new(
            compressed.as_ref(),
            Some(&INPUT),
            &mut encoding_info,
            (&mut buf).writer(),
        );
        let result = dstruct.try_decompress_main_plus_extra();
        if let Err(e) = result {
            assert_eq!(e.reason, MultiDecompressErrorReason::ExtraRaw);
            assert!(matches!(e.error, DecompressError::Copy(_)));
            assert_eq!(dstruct.encoding_info, to_check_encoding_info);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_dstruct_decompress_main_plus_extra_error_extra_raw_zstd() {
        let mut compressed = compress_zstd(INPUT);
        let mut encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Zstd])];
        let to_check_encoding_info = encoding_info.clone();
        let mut buf = BytesMut::new();
        let mut dstruct = DecompressionStruct::new(
            compressed.as_ref(),
            Some(&INPUT),
            &mut encoding_info,
            (&mut buf).writer(),
        );
        let result = dstruct.try_decompress_main_plus_extra();
        if let Err(e) = result {
            assert_eq!(e.reason, MultiDecompressErrorReason::ExtraRaw);
            assert!(matches!(e.error, DecompressError::Copy(_)));
            assert_eq!(dstruct.encoding_info, to_check_encoding_info);
        } else {
            panic!();
        }
    }

    // try_decompress_chain
    #[test]
    fn test_dstruct_decompress_main_plus_extra_partial_error_two_values() {
        let mut compressed = compress_brotli(INPUT);
        let (first, second) = compressed.split_at(compressed.len() / 2);
        let mut encoding_info = vec![EncodingInfo::new(
            0,
            vec![ContentEncoding::Deflate, ContentEncoding::Brotli],
        )];
        let to_check_encoding_info = encoding_info.clone();
        let mut buf = BytesMut::new();
        let mut dstruct = DecompressionStruct::new(
            first.as_ref(),
            Some(&second),
            &mut encoding_info,
            (&mut buf).writer(),
        );
        let result = dstruct.try_decompress_main_plus_extra();
        if let Err(MultiDecompressError {
            reason,
            error,
        }) = result
        {
            dbg!(reason);
            dbg!(error);
            //assert_eq!(e.reason, MultiDecompressErrorReason::ExtraRaw);
            //assert!(matches!(e.error, DecompressError::Copy(_)));
            //assert_eq!(dstruct.encoding_info, to_check_encoding_info);
        } else {
            panic!();
        }
    }
}
