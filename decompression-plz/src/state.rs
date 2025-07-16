use crate::{
    decompression::{
        magic_bytes::is_compressed,
        multi::{decompress_multi, error::MultiDecompressError},
    },
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

#[cfg_attr(test, derive(PartialEq))]
pub enum State<'a> {
    // Main
    MainOnly(DecompressionStruct<'a>),
    EndMainOnlyDecompressed(BytesMut),
    // Main + Extra
    ExtraTryDecompress(DecompressionStruct<'a>),
    MainPlusExtraTryDecompress(DecompressionStruct<'a>),
    ExtraDecompressedMainTryDecompress(DecompressionStruct<'a>, BytesMut),
    ExtraNoDecompressMainTryDecompress(DecompressionStruct<'a>),
    EndMainOnyDecompressed(DecompressionStruct<'a>),
    EndMainPlusExtraDecompressed(BytesMut),
    EndExtraMainDecompressedSeparate(BytesMut, BytesMut),
}

impl std::fmt::Debug for State<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::MainOnly(_) => write!(f, "MainOnly"),
            State::EndMainOnlyDecompressed(_) => write!(f, "EndMainOnly"),
            State::ExtraTryDecompress(_) => write!(f, "Extra"),
            State::ExtraDecompressedMainTryDecompress(..) => {
                write!(f, "ExtraDecompressedMainTryDecompress")
            }
            State::ExtraNoDecompressMainTryDecompress(_) => write!(f, "MainPlusExtra"),
            State::EndMainOnyDecompressed(_) => write!(f, "EndMainOnyDecompressed"),
            State::EndMainPlusExtraDecompressed(_) => write!(f, "EndMainPlusExtra"),
            State::MainPlusExtraTryDecompress(_) => write!(f, "ExtraPlainMainTryDecompress"),
            State::EndExtraMainDecompressedSeparate(..) => {
                write!(f, "EndExtraMainDecompressedSeparate")
            }
        }
    }
}

impl<'a> State<'a> {
    pub fn start(
        main: BytesMut,
        extra: Option<BytesMut>,
        encodings: &'a [EncodingInfo],
        buf: &'a mut BytesMut,
    ) -> Self {
        let dstruct = DecompressionStruct::new(main, extra, encodings, buf);
        if dstruct.extra.is_some() {
            Self::ExtraTryDecompress(dstruct)
        } else {
            Self::MainOnly(dstruct)
        }
    }

    fn try_next(self) -> Result<Self, MultiDecompressError> {
        match self {
            State::MainOnly(mut dstruct) => {
                let result = dstruct.try_decompress_main()?;
                Ok(State::EndMainOnlyDecompressed(result))
            }
            State::ExtraTryDecompress(mut dstruct) => match dstruct.is_extra_compressed() {
                true => match dstruct.try_decompress_extra() {
                    Ok(extra_decompressed) => Ok(State::ExtraDecompressedMainTryDecompress(
                        dstruct,
                        extra_decompressed,
                    )),
                    Err(_) => Ok(State::ExtraNoDecompressMainTryDecompress(dstruct)),
                },
                false => Ok(State::MainPlusExtraTryDecompress(dstruct)),
            },
            State::ExtraDecompressedMainTryDecompress(mut dstruct, extra) => {
                let result = dstruct.try_decompress_main()?;
                Ok(State::EndMainPlusExtraDecompressed(result))
            }
            State::ExtraNoDecompressMainTryDecompress(decompression_struct) => todo!(),
            State::MainPlusExtraTryDecompress(decompression_struct) => todo!(),
            State::EndMainOnyDecompressed(decompression_struct) => todo!(),
            State::EndMainOnlyDecompressed(_)
            | State::EndMainPlusExtraDecompressed(_)
            | State::EndExtraMainDecompressedSeparate(..) => {
                panic!("already ended")
            }
        }
    }

    fn ended(&self) -> bool {
        matches!(self, Self::EndMainOnlyDecompressed(_))
            || matches!(self, Self::EndExtraMainDecompressedSeparate(..))
            || matches!(self, Self::EndMainPlusExtraDecompressed(_))
    }
}

pub fn runner(mut state: State) -> Result<State, MultiDecompressError> {
    loop {
        state = state.try_next()?;
        if state.ended() {
            return Ok(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    use crate::tests::*;

    fn test_all_compression(einfo: Vec<EncodingInfo>) {
        let compressed = all_compressed_data();
        let input = BytesMut::from(&compressed[..]);
        let mut buf = BytesMut::new();
        let state = State::start(input, None, &einfo, &mut buf);
        let result = runner(state).unwrap();
        assert_eq!(result, State::EndMainOnlyDecompressed("hello world".into()));
    }

    // ----- Main
    #[test]
    fn test_state_main_only_single_header() {
        let einfo = all_encoding_info_single_header();
        test_all_compression(einfo);
    }

    #[test]
    fn test_state_main_only_multi_header() {
        let einfo = all_encoding_info_multi_header();
        test_all_compression(einfo);
    }

    // ----- Main + Extra
    #[test]
    fn test_state_main_extra_compressed_together_single_header() {
        let einfo = all_encoding_info_single_header();
        let compressed = all_compressed_data();
        let main = BytesMut::from(&compressed[..compressed.len() / 2]);
        let extra = BytesMut::from(&compressed[compressed.len() / 2..]);
        let mut buf = BytesMut::new();
        let state = State::start(main, Some(extra), &einfo, &mut buf);
        let result = runner(state).unwrap();
        assert_eq!(
            result,
            State::EndMainPlusExtraDecompressed("hello world".into())
        );
    }
}
