use std::io::{Cursor, Read};

use bytes::{BufMut, BytesMut, buf::Writer};
use header_plz::body_headers::{
    content_encoding::ContentEncoding, encoding_info::EncodingInfo,
};

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
        self.encoding_info
            .last()
            .unwrap()
            .encodings()
            .last()
            .unwrap()
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

    /*
    pub fn is_encodings_empty(&self) -> bool {
        let mut iter = self.encoding_info.iter().rev();
        if iter
            .next()
            .unwrap()
            .encodings()
            .is_empty()
        {
            return iter.next().is_none();
        }
        false
    }*/

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
            &self.encoding_info,
        )
    }

    pub fn try_decompress_main(
        &mut self,
    ) -> Result<BytesMut, MultiDecompressError> {
        decompress_multi(
            self.main.as_ref(),
            &mut self.writer,
            &self.encoding_info,
        )
    }

    pub fn try_decompress_main_plus_extra(
        &mut self,
    ) -> Result<BytesMut, MultiDecompressError> {
        let last_encoding = self.pop_last_encoding();
        let mut chained = Cursor::new(self.main.as_ref())
            .chain(Cursor::new(self.extra.as_ref().unwrap().as_ref()));
        dbg!(&chained);
        let _ =
            decompress_single(&mut chained, &mut self.writer, &last_encoding)
                .map_err(|e| {
                    MultiDecompressError::new(
                        MultiDecompressErrorReason::Corrupt,
                        e,
                    )
                })?;
        let (main, extra) = chained.get_ref();
        dbg!(main.position());
        dbg!(extra.position());

        let output = self.writer.get_mut().split();
        //dbg!(&output);
        //dbg!(&self.encoding_info);
        //dbg!(&self.writer);
        if self.is_encodings_empty() {
            dbg!("empty");
            Ok(output)
        } else {
            let result = decompress_multi(
                &output,
                &mut self.writer,
                &self.encoding_info,
            )?;
            self.push_last_encoding(last_encoding);
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{
        all_encoding_info_multi_header, all_encoding_info_single_header,
    };

    use super::*;

    #[test]
    fn test_decompression_struct_last_encoding_single_value() {
        let mut encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Gzip])];
        let mut buf = BytesMut::new();
        let decompression_struct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(
            decompression_struct.last_encoding(),
            &ContentEncoding::Gzip
        );
    }

    #[test]
    fn test_decompression_struct_last_encoding_single_header() {
        let mut encoding_info = all_encoding_info_single_header();
        let mut buf = BytesMut::new();
        let decompression_struct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(
            decompression_struct.last_encoding(),
            &ContentEncoding::Zstd
        );
    }

    #[test]
    fn test_decompression_struct_last_encoding_multi_header() {
        let mut encoding_info = all_encoding_info_multi_header();
        let mut buf = BytesMut::new();
        let decompression_struct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert_eq!(
            decompression_struct.last_encoding(),
            &ContentEncoding::Zstd
        );
    }

    #[test]
    fn test_push_last_encoding_single_value() {
        let mut encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Gzip])];

        let mut buf = BytesMut::new();
        let mut decompression_struct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        decompression_struct.push_last_encoding(ContentEncoding::Brotli);
        let to_verify = vec![EncodingInfo::new(
            0,
            vec![ContentEncoding::Gzip, ContentEncoding::Brotli],
        )];

        assert_eq!(decompression_struct.encoding_info, to_verify);
    }

    #[test]
    fn test_push_last_encoding_single_header() {
        let mut encoding_info = all_encoding_info_single_header();
        let mut buf = BytesMut::new();
        let mut decompression_struct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        decompression_struct.push_last_encoding(ContentEncoding::Brotli);
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

        assert_eq!(decompression_struct.encoding_info, to_verify);
    }

    #[test]
    fn test_push_last_encoding_multi_header() {
        let mut encoding_info = all_encoding_info_multi_header();
        let mut buf = BytesMut::new();
        let mut decompression_struct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        decompression_struct.push_last_encoding(ContentEncoding::Brotli);
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

        assert_eq!(decompression_struct.encoding_info, to_verify);
    }

    #[test]
    fn test_dstruct_is_encodings_empty_true_single_value() {
        let mut encoding_info = vec![EncodingInfo::new(0, vec![])];
        let mut buf = BytesMut::new();
        let decompression_struct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert!(decompression_struct.is_encodings_empty());
    }

    #[test]
    fn test_dstruct_is_encodings_empty_false_single_value() {
        let mut encoding_info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Gzip])];
        let mut buf = BytesMut::new();
        let decompression_struct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert!(!decompression_struct.is_encodings_empty());
    }

    #[test]
    fn test_dstruct_is_encodings_empty_false_last_encoding_empty() {
        let mut encoding_info = vec![
            EncodingInfo::new(0, vec![ContentEncoding::Gzip]),
            EncodingInfo::new(1, vec![]),
        ];
        let mut buf = BytesMut::new();
        let decompression_struct = DecompressionStruct::new(
            &[],
            None,
            &mut encoding_info,
            (&mut buf).writer(),
        );
        assert!(!decompression_struct.is_encodings_empty());
    }
}
