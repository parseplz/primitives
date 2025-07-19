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

    pub fn extra(&self) -> &[u8] {
        self.extra.as_ref().unwrap().as_ref()
    }

    pub fn is_extra_compressed(&self) -> bool {
        is_compressed(self.extra(), self.last_encoding())
    }

    pub fn len(&self) -> usize {
        self.main.len()
            + self
                .extra
                .as_ref()
                .map(|e| e.len())
                .unwrap_or(0)
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
        let len = self.len() as u64;
        let result = Self::decompress_chain(
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
            output = decompress_multi(
                &output,
                &mut self.writer,
                &self.encoding_info,
            )?;
        }
        self.push_last_encoding(last_encoding);
        Ok(output)
    }

    fn decompress_chain(
        mut input: std::io::Chain<Cursor<&[u8]>, Cursor<&[u8]>>,
        mut writer: &mut Writer<&mut BytesMut>,
        content_encoding: &ContentEncoding,
        len: u64,
    ) -> Result<(), MultiDecompressError> {
        if let ContentEncoding::Deflate = content_encoding {
            let mut reader = flate2::read::ZlibDecoder::new(input);
            std::io::copy(&mut reader, &mut writer)?;
            if reader.total_in() != len {
                return Err(MultiDecompressError::deflate_corrupt());
            }
            return Ok(());
        }
        decompress_single(&mut input, &mut writer, &content_encoding)
            .map_err(MultiDecompressError::corrupt)?;
        if let (_, extra_curs) = input.get_ref()
            && extra_curs.position() == 0
        {
            return Err(MultiDecompressError::extra_raw());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{
        all_encoding_info_multi_header, all_encoding_info_single_header,
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
}
