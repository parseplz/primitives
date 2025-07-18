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
    pub encoding_info: &'a [EncodingInfo],
    pub writer: Writer<&'a mut BytesMut>,
}

impl<'a> DecompressionStruct<'a> {
    pub fn new(
        main: &'a [u8],
        extra: Option<&'a [u8]>,
        encoding_info: &'a [EncodingInfo],
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
        /*
        &mut self
            .encoding_info
            .last()
            .unwrap()
            .encodings_as_mut()
            .pop()
            .unwrap()
        */
        todo!()
    }

    pub fn push_last_encoding(&mut self, encoding: ContentEncoding) {
        todo!()
        //self.encoding_info
        //    .last_mut()
        //    .unwrap()
        //    .encodings_as_mut()
        //    .push(encoding);
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
        let last_encoding = self.last_encoding();
        is_compressed(self.extra(), last_encoding)
    }

    pub fn try_decompress_extra(
        &mut self,
    ) -> Result<BytesMut, MultiDecompressError> {
        //let mut writer = self.writer.writer();
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
