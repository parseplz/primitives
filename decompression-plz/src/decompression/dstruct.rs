use std::io::{Cursor, Read};

use bytes::{BytesMut, buf::Writer};
use header_plz::body_headers::{
    content_encoding::ContentEncoding, encoding_info::EncodingInfo,
};

use crate::decompression::{
    magic_bytes::is_compressed,
    multi::{decompress_multi, error::MultiDecompressError},
    single::{decompress_single, error::DecompressError},
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

    pub fn last_encoding(&self) -> Option<&ContentEncoding> {
        self.encoding_info.last().and_then(|einfo| einfo.encodings().last())
    }

    pub fn pop_last_encoding(&mut self) -> Option<ContentEncoding> {
        self.encoding_info
            .last_mut()
            .and_then(|einfo| einfo.encodings_as_mut().pop())
    }

    pub fn push_last_encoding(&mut self, encoding: ContentEncoding) {
        if let Some(einfo) = self.encoding_info.last_mut() {
            einfo.encodings_as_mut().push(encoding)
        }
    }

    pub fn is_encodings_empty(&self) -> bool {
        match self.encoding_info.split_last() {
            Some((last, [])) => last.encodings().is_empty(),
            _ => false,
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.main.len() + self.extra.as_ref().map(|e| e.len()).unwrap_or(0)
    }

    /* pre - last encoding in last_encoding_info is popped
     *
     * last_encoding_info is empty
     *          - vec![ vec![..], vec![] ]
     *          - (1, 0)
     *
     * last_encoding_info not empty
     *          - vec![ vec![..], vec![..] ]
     *          - (0, 1)
     */
    pub fn last_header_compression_index(&self) -> (usize, usize) {
        if let Some((last, _)) = self.encoding_info.split_last()
            && last.encodings().is_empty()
        {
            return (1, 0);
        }
        (0, 1)
    }

    pub fn clear_buf(&mut self) {
        self.writer.get_mut().clear();
    }

    pub fn extra(&self) -> Option<&[u8]> {
        self.extra
    }

    pub fn is_extra_compressed(&self) -> bool {
        if let Some(encoding) = self.last_encoding()
            && let Some(extra) = self.extra()
        {
            is_compressed(extra, encoding)
        } else {
            false
        }
    }

    pub fn try_decompress_extra(
        &mut self,
    ) -> Result<BytesMut, MultiDecompressError> {
        decompress_multi(
            self.extra.as_ref().expect("no extra"),
            &mut self.writer,
            &mut self.encoding_info.iter(),
        )
    }

    pub fn try_decompress_main(
        &mut self,
    ) -> Result<BytesMut, MultiDecompressError> {
        decompress_multi(
            self.main,
            &mut self.writer,
            &mut self.encoding_info.iter(),
        )
    }

    pub fn try_decompress_main_plus_extra(
        &mut self,
    ) -> Result<BytesMut, MultiDecompressError> {
        let last_encoding =
            self.pop_last_encoding().expect("no last encoding");
        let chained = Cursor::new(self.main)
            .chain(Cursor::new(self.extra.expect("no extra")));
        let len = self.len() as u64;
        let result = Self::try_decompress_chain_first(
            chained,
            &mut self.writer,
            &last_encoding,
            len,
        );
        if let Err(e) = result {
            self.push_last_encoding(last_encoding);
            return Err(e.into());
        }
        let output = self.writer.get_mut().split();
        if !self.is_encodings_empty() {
            self.try_decompress_chain_remaining(output, last_encoding)
        } else {
            self.push_last_encoding(last_encoding);
            Ok(output)
        }
    }

    fn try_decompress_chain_first(
        mut input: std::io::Chain<Cursor<&[u8]>, Cursor<&[u8]>>,
        mut writer: &mut Writer<&mut BytesMut>,
        content_encoding: &ContentEncoding,
        len: u64,
    ) -> Result<(), DecompressError> {
        if let ContentEncoding::Deflate = content_encoding {
            let mut reader = flate2::read::ZlibDecoder::new(input);
            std::io::copy(&mut reader, &mut writer)
                .map_err(DecompressError::Deflate)?;
            if reader.total_in() != len {
                return Err(DecompressError::deflate());
            }
            return Ok(());
        }
        // others
        decompress_single(&mut input, &mut writer, content_encoding)?;
        let (_, extra_curs) = input.get_ref();
        // brotli
        if extra_curs.position() == 0 {
            return Err(DecompressError::corrupt(content_encoding));
        }
        Ok(())
    }

    fn try_decompress_chain_remaining(
        &mut self,
        input: BytesMut,
        last_encoding: ContentEncoding,
    ) -> Result<BytesMut, MultiDecompressError> {
        let iter = &mut self.encoding_info.iter();
        let result =
            decompress_multi(&input, &mut self.writer, iter).map_err(|e| {
                if e.is_corrupt() {
                    let (header_index, compression_index) =
                        self.last_header_compression_index();
                    e.corrupt_to_partial(
                        input,
                        header_index,
                        compression_index,
                    )
                } else {
                    e
                }
            });
        self.push_last_encoding(last_encoding);
        result
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;
    use tests_utils::*;

    use crate::decompression::multi::error::MultiDecompressErrorReason;

    use super::*;

    // last_encoding() + pop_last_encoding()
    #[track_caller]
    fn assert_last_encoding(
        initial: Vec<EncodingInfo>,
        expected_last: ContentEncoding,
    ) {
        let mut encoding_info = initial.clone();
        let mut buf = BytesMut::new();
        let mut d = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(d.last_encoding().unwrap(), &expected_last,);
        assert_eq!(d.pop_last_encoding().unwrap(), expected_last,);
    }

    #[test]
    fn test_dstruct_last_encoding_single_value() {
        let encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Gzip])];
        assert_last_encoding(encoding_info, ContentEncoding::Gzip);
    }

    #[test]
    fn test_dstruct_last_encoding_single_header() {
        let encoding_info = all_encoding_info_single_header();
        assert_last_encoding(encoding_info, ContentEncoding::Zstd);
    }

    #[test]
    fn test_dstruct_last_encoding_multi_header() {
        let encoding_info = all_encoding_info_multi_header();
        assert_last_encoding(encoding_info, ContentEncoding::Zstd);
    }

    // push_last_encoding
    #[track_caller]
    fn assert_push_last_encoding(
        initial: Vec<EncodingInfo>,
        push: ContentEncoding,
        expected: Vec<EncodingInfo>,
    ) {
        let mut encoding_info = initial.clone();
        let mut buf = BytesMut::new();
        let mut dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        dstruct.push_last_encoding(push);
        assert_eq!(dstruct.encoding_info, expected);
    }

    #[test]
    fn test_dstruct_push_last_encoding_single_value() {
        let encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Gzip])];
        let to_verify = vec![EncodingInfo::new(
            0,
            vec![ContentEncoding::Gzip, ContentEncoding::Brotli],
        )];

        assert_push_last_encoding(
            encoding_info,
            ContentEncoding::Brotli,
            to_verify,
        );
    }

    #[test]
    fn test_push_last_encoding_single_header() {
        let encoding_info = all_encoding_info_single_header();
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

        assert_push_last_encoding(
            encoding_info,
            ContentEncoding::Brotli,
            to_verify,
        );
    }

    #[test]
    fn test_push_last_encoding_multi_header() {
        let encoding_info = all_encoding_info_multi_header();
        let to_verify = vec![
            EncodingInfo::new(1, vec![ContentEncoding::Brotli]),
            EncodingInfo::new(3, vec![ContentEncoding::Deflate]),
            EncodingInfo::new(5, vec![ContentEncoding::Identity]),
            EncodingInfo::new(7, vec![ContentEncoding::Gzip]),
            EncodingInfo::new(
                9,
                vec![ContentEncoding::Zstd, ContentEncoding::Brotli],
            ),
        ];

        assert_push_last_encoding(
            encoding_info,
            ContentEncoding::Brotli,
            to_verify,
        );
    }

    // is_encodings_empty
    #[track_caller]
    fn assert_encodings_empty(
        mut encoding_info: Vec<EncodingInfo>,
        expect: bool,
    ) {
        let mut buf = BytesMut::new();
        let d = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(d.is_encodings_empty(), expect,);
    }

    #[test]
    fn test_dstruct_is_encodings_empty_true_single_value() {
        let encoding_info = vec![EncodingInfo::new(0, vec![])];
        assert_encodings_empty(encoding_info, true);
    }

    #[test]
    fn test_dstruct_is_encodings_empty_false_single_value() {
        let encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Gzip])];
        assert_encodings_empty(encoding_info, false);
    }

    #[test]
    fn test_dstruct_is_encodings_empty_false_last_encoding_empty() {
        let encoding_info = vec![
            EncodingInfo::new(0, vec![ContentEncoding::Gzip]),
            EncodingInfo::new(1, vec![]),
        ];
        assert_encodings_empty(encoding_info, false);
    }

    // len()
    #[track_caller]
    fn assert_len(main: &[u8], extra: Option<&[u8]>, expected: usize) {
        let mut encoding_info = Vec::new();
        let mut buf = BytesMut::new();
        let dstruct = DecompressionStruct::new(
            main,
            extra,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(dstruct.len(), expected);
    }

    #[test]
    fn test_dstruct_len_empty() {
        assert_len(&[], None, 0);
    }

    #[test]
    fn test_dstruct_len_only_main() {
        assert_len(&[1, 2, 3, 4, 5], None, 5);
    }

    #[test]
    fn test_dstruct_len_main_and_extra() {
        assert_len(&[1, 2, 3, 4, 5], Some(&[6, 7, 8, 9, 10]), 10);
    }

    // last_header_compression_index
    #[track_caller]
    fn assert_last_header_compression_index(
        mut encoding_info: Vec<EncodingInfo>,
        expected: (usize, usize),
    ) {
        let mut buf = BytesMut::new();
        let dstruct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(
            dstruct.last_header_compression_index(),
            expected,
            "unexpected result for encoding_info = {encoding_info:?}"
        );
    }

    #[test]
    fn test_dstruct_last_header_compression_index_sigle_value() {
        let encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Gzip])];
        assert_last_header_compression_index(encoding_info, (0, 1));
    }

    #[test]
    fn test_dstruct_last_header_compression_index_multi_value_single_header() {
        let encoding_info = all_encoding_info_single_header();
        assert_last_header_compression_index(encoding_info, (0, 1));
    }

    #[test]
    fn test_dstruct_last_header_compression_index_multi_value_multi_header() {
        let encoding_info = all_encoding_info_multi_header();
        assert_last_header_compression_index(encoding_info, (0, 1));
    }

    #[test]
    fn test_dstruct_last_header_compression_index_last_empty() {
        let encoding_info = vec![
            EncodingInfo::new(
                0,
                vec![ContentEncoding::Gzip, ContentEncoding::Brotli],
            ),
            EncodingInfo::new(1, vec![]),
        ];
        assert_last_header_compression_index(encoding_info, (1, 0));
    }

    // try_decompress_chain_errors
    #[test]
    fn test_dstruct_d_main_extra_err_extra_raw_all() {
        for &(compress_fn, ref encoding) in &[
            (
                compress_deflate as fn(&[u8]) -> Vec<u8>,
                ContentEncoding::Deflate,
            ),
            (compress_brotli, ContentEncoding::Brotli),
            (compress_gzip, ContentEncoding::Gzip),
            (compress_zstd, ContentEncoding::Zstd),
        ] {
            let compressed = compress_fn(INPUT);
            let mut encoding_info =
                vec![EncodingInfo::new(0, vec![encoding.clone()])];
            let original_info = encoding_info.clone();
            let mut buf = BytesMut::new();
            let mut ds = DecompressionStruct::new(
                compressed.as_ref(),
                Some(INPUT),
                &mut encoding_info,
                (&mut buf).writer(),
            );

            let err = ds.try_decompress_main_plus_extra().unwrap_err();
            assert_eq!(err.reason, MultiDecompressErrorReason::Corrupt);
            assert!(!ds.writer.get_ref().is_empty());
            assert_eq!(ds.encoding_info, original_info);
        }
    }

    // try_decompress_remaining
    #[track_caller]
    fn assert_dstruct_d_main_extra_partial_err(
        main: &[u8],
        extra: Option<&[u8]>,
        mut encoding_info: Vec<EncodingInfo>,
        expected_header_index: usize,
        expected_compression_index: usize,
    ) {
        let mut buf = BytesMut::new();
        let verify_encodings = encoding_info.clone();
        let mut ds = DecompressionStruct::new(
            main,
            extra,
            &mut encoding_info,
            (&mut buf).writer(),
        );

        let e = ds.try_decompress_main_plus_extra().unwrap_err();
        if let MultiDecompressErrorReason::Partial {
            partial_body,
            header_index,
            compression_index,
            ..
        } = e.reason
        {
            assert_eq!(partial_body, INPUT);
            assert_eq!(header_index, expected_header_index);
            assert_eq!(compression_index, expected_compression_index);
        } else {
            panic!("Expected Partial error, got: {:?}", e.reason);
        }

        assert!(ds.writer.get_ref().is_empty());
        assert_eq!(ds.encoding_info, verify_encodings);
    }

    // Corrupt error to Partial
    #[test]
    fn test_dstruct_d_main_extra_partial_err_corrupt_to_partial_two_values() {
        let compressed = compress_brotli(INPUT);
        let (first, second) = compressed.split_at(compressed.len() / 2);
        let encoding_info = vec![EncodingInfo::new(
            0,
            vec![ContentEncoding::Deflate, ContentEncoding::Brotli],
        )];
        assert_dstruct_d_main_extra_partial_err(
            first,
            Some(second),
            encoding_info,
            0,
            1,
        );
    }

    #[test]
    fn test_dstruct_d_main_extra_partial_err_corrupt_to_partial_three_values()
    {
        let compressed = compress_brotli(INPUT);
        let (first, second) = compressed.split_at(compressed.len() / 2);
        let encoding_info = vec![EncodingInfo::new(
            0,
            vec![
                ContentEncoding::Deflate,
                ContentEncoding::Gzip,
                ContentEncoding::Brotli,
            ],
        )];
        assert_dstruct_d_main_extra_partial_err(
            first,
            Some(second),
            encoding_info,
            0,
            1,
        );
    }

    #[test]
    fn test_dstruct_d_main_extra_partial_err_corrupt_to_partial_single_header()
    {
        let compressed = compress_zstd(INPUT);
        let (first, second) = compressed.split_at(compressed.len() / 2);
        let encoding_info = all_encoding_info_single_header();
        assert_dstruct_d_main_extra_partial_err(
            first,
            Some(second),
            encoding_info,
            0,
            1,
        );
    }

    #[test]
    fn test_dstruct_d_main_extra_partial_err_corrupt_to_partial_multi_header()
    {
        let compressed = compress_zstd(INPUT);
        let (first, second) = compressed.split_at(compressed.len() / 2);
        let encoding_info = all_encoding_info_multi_header();
        assert_dstruct_d_main_extra_partial_err(
            first,
            Some(second),
            encoding_info,
            1,
            0,
        );
    }

    // Partial Error
    fn compress_gzip_zstd() -> Vec<u8> {
        let br = compress_gzip(INPUT);
        compress_zstd(&br)
    }

    #[test]
    fn test_dstruct_d_main_extra_partial_err_partial_three_values() {
        let compressed = compress_gzip_zstd();
        let (first, second) = compressed.split_at(compressed.len() / 2);
        let encoding_info = vec![EncodingInfo::new(
            0,
            vec![
                ContentEncoding::Deflate,
                ContentEncoding::Gzip,
                ContentEncoding::Zstd,
            ],
        )];
        assert_dstruct_d_main_extra_partial_err(
            first,
            Some(second),
            encoding_info,
            0,
            1,
        );
    }

    #[test]
    fn test_dstruct_d_main_extra_partial_err_partial_single_header() {
        let compressed = compress_gzip_zstd();
        let (first, second) = compressed.split_at(compressed.len() / 2);
        let encoding_info = all_encoding_info_single_header();
        assert_dstruct_d_main_extra_partial_err(
            first,
            Some(second),
            encoding_info,
            0,
            2,
        );
    }

    #[test]
    fn test_dstruct_d_main_extra_partial_err_partial_multi_header() {
        let compressed = compress_gzip_zstd();
        let (first, second) = compressed.split_at(compressed.len() / 2);
        let encoding_info = all_encoding_info_multi_header();
        assert_dstruct_d_main_extra_partial_err(
            first,
            Some(second),
            encoding_info,
            3,
            0,
        );
    }

    #[test]
    fn test_dstruct_d_main_extra_partial_err_partial_multi_header_mixed() {
        let compressed = compress_gzip_zstd();
        let (first, second) = compressed.split_at(compressed.len() / 2);
        let encoding_info = vec![
            EncodingInfo::new(0, vec![ContentEncoding::Brotli]),
            EncodingInfo::new(1, vec![ContentEncoding::Deflate]),
            EncodingInfo::new(
                2,
                vec![ContentEncoding::Identity, ContentEncoding::Gzip],
            ),
            EncodingInfo::new(3, vec![ContentEncoding::Zstd]),
        ];
        assert_dstruct_d_main_extra_partial_err(
            first,
            Some(second),
            encoding_info,
            2,
            0,
        );
    }

    #[test]
    fn test_dstruct_d_main_extra_partial_err_partial_multi_header_mixed_two() {
        let compressed = compress_gzip_zstd();
        let (first, second) = compressed.split_at(compressed.len() / 2);
        let encoding_info = vec![
            EncodingInfo::new(0, vec![ContentEncoding::Brotli]),
            EncodingInfo::new(
                1,
                vec![
                    ContentEncoding::Deflate,
                    ContentEncoding::Identity,
                    ContentEncoding::Gzip,
                ],
            ),
            EncodingInfo::new(3, vec![ContentEncoding::Zstd]),
        ];
        assert_dstruct_d_main_extra_partial_err(
            first,
            Some(second),
            encoding_info,
            1,
            2,
        );
    }
}
