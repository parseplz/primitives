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
        self.encoding_info
            .last()
            .unwrap()
            .encodings()
            .is_empty()
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
        let to_copy = self.main.len() + self.extra().len();
        let mut buf = self.writer.get_mut();
        buf.reserve(to_copy);
        // copy main and extra to buf
        buf.put(self.main.as_ref());
        buf.put(self.extra.as_ref().unwrap().as_ref());
        let combined = buf.split();
        decompress_multi(&combined, &mut self.writer, &self.encoding_info)
    }

    pub fn try_decompress_main_plus_extra_new(
        &mut self,
    ) -> Result<BytesMut, MultiDecompressError> {
        let last_encoding = self.pop_last_encoding();
        let mut chained = Cursor::new(self.main.as_ref())
            .chain(Cursor::new(self.extra.as_ref().unwrap().as_ref()));
        let _ = decompress_single(chained, &mut self.writer, &last_encoding)
            .map_err(|e| {
            MultiDecompressError::new(MultiDecompressErrorReason::Corrupt, e)
        })?;

        let output = self.writer.get_mut().split();
        if self.is_encodings_empty() {
            Ok(output)
        } else {
            let _ = decompress_multi(
                &output,
                &mut self.writer,
                &self.encoding_info,
            )?;
            self.push_last_encoding(last_encoding);
            Ok(self.writer.get_mut().split())
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
            &ContentEncoding::Identity
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
            &ContentEncoding::Identity
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
                ContentEncoding::Gzip,
                ContentEncoding::Zstd,
                ContentEncoding::Identity,
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
            EncodingInfo::new(2, vec![ContentEncoding::Gzip]),
            EncodingInfo::new(3, vec![ContentEncoding::Zstd]),
            EncodingInfo::new(
                4,
                vec![ContentEncoding::Identity, ContentEncoding::Brotli],
            ),
        ];

        assert_eq!(decompression_struct.encoding_info, to_verify);
    }

    #[test]
    fn test_dstruct_is_encodings_empty_true() {
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
    fn test_dstruct_is_encodings_empty_false() {
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
}
