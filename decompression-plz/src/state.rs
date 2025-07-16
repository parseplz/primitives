use crate::{
    decompression::multi::{decompress_multi, error::MultiDecompressError},
    dstruct::DecompressionStruct,
    error::DecompressErrorStruct,
};
use bytes::{BufMut, BytesMut};
use header_plz::body_headers::{content_encoding::ContentEncoding, encoding_info::EncodingInfo};

/*
1. Main
    true    => Main_decompressed + Extra_decompressed
    false   => Err()

2. Extra
    true    => Main
    false   => Main + Extra

3. Main + Extra
    true    => Main_and_Extra_decompressed
    false   => Err()
*/

// 1. Try decompressing extra
// 2. If success, try decompressing main
// 2. If failed, try decompressing main + extra
// 3. If failed, try decompressing main

/*
1. Extra part of Main. ie. compresssed together

    Compression | Result
    ------------|----------
    all         | success

2. Main - compressed + Extra - raw

    Compression | Result
    ------------|----------
    brotli      | error + main decompressed + extra no read
    deflate     | error + main decompressed + extra read
    gzip        | error + main decompressed + extra no read
    zstd        | error + main decompressed + extra read

3. Main - compressed + Extra - compressed = separately compressed

    Compression | Result
    ------------|----------
    brotli      | error + main decompressed + extra no read
    deflate     | error + main decompressed + extra read
    gzip        | error + main decompressed + extra no read
    zstd        | success
*/

enum State<'a> {
    // Main
    MainOnly(DecompressionStruct<'a>),
    EndMainOnly(BytesMut),
    // Main + Extra
    Extra(DecompressionStruct<'a>),
    ExtraDecompressedMain(DecompressionStruct<'a>),
    MainPlusExtra(DecompressionStruct<'a>),
    EndMainOnyDecompressed(DecompressionStruct<'a>),
    EndMainPlusExtra(DecompressionStruct<'a>),
}

impl<'a> State<'a> {
    fn start(
        main: BytesMut,
        extra: Option<BytesMut>,
        encodings: &'a [EncodingInfo],
        buf: &'a mut BytesMut,
    ) -> Self {
        let dstruct = DecompressionStruct::new(main, extra, encodings, buf);
        if dstruct.extra.is_some() {
            Self::Extra(dstruct)
        } else {
            Self::MainOnly(dstruct)
        }
    }

    fn try_next(self) -> Result<Self, MultiDecompressError> {
        match self {
            // Main only
            State::MainOnly(dstruct) => {
                let mut writer = dstruct.buf.writer();
                let result = decompress_multi(&dstruct.main, &mut writer, &dstruct.encoding_info)?;
                Ok(State::EndMainOnly(result))
            }
            State::EndMainOnly(_) | State::EndMainPlusExtra(_) => {
                panic!("already ended")
            }
            //
            State::Extra(decompression_struct) => todo!(),
            State::ExtraDecompressedMain(decompression_struct) => todo!(),
            State::MainPlusExtra(decompression_struct) => todo!(),
            State::EndMainOnyDecompressed(decompression_struct) => todo!(),
        }
    }

    fn ended(self) -> bool {
        matches!(self, Self::EndMainOnly(_)) || matches!(self, Self::EndMainOnyDecompressed(_))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_state_main_only_single() {}
}
