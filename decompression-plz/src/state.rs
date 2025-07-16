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

#[cfg_attr(test, derive(PartialEq))]
pub enum State<'a> {
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

impl std::fmt::Debug for State<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::MainOnly(_) => write!(f, "MainOnly"),
            State::EndMainOnly(_) => write!(f, "EndMainOnly"),
            State::Extra(_) => write!(f, "Extra"),
            State::ExtraDecompressedMain(_) => write!(f, "ExtraDecompressedMain"),
            State::MainPlusExtra(_) => write!(f, "MainPlusExtra"),
            State::EndMainOnyDecompressed(_) => write!(f, "EndMainOnyDecompressed"),
            State::EndMainPlusExtra(_) => write!(f, "EndMainPlusExtra"),
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

    fn ended(&self) -> bool {
        matches!(self, Self::EndMainOnly(_)) || matches!(self, Self::EndMainOnyDecompressed(_))
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

    #[test]
    fn test_state_main_only_single() {
        let compressed = all_compressed_data();
        let input = BytesMut::from(&compressed[..]);
        let einfo = all_encoding_info();
        let mut buf = BytesMut::new();
        let state = State::start(input, None, &einfo, &mut buf);
        let result = runner(state).unwrap();
        assert_eq!(result, State::EndMainOnly("hello world".into()));
    }
}
