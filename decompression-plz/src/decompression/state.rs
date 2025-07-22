use crate::decompression::{
    dstruct::DecompressionStruct,
    magic_bytes::is_compressed,
    multi::{decompress_multi, error::MultiDecompressError},
};
use bytes::{BufMut, BytesMut, buf::Writer};
use header_plz::body_headers::{
    content_encoding::ContentEncoding, encoding_info::EncodingInfo,
};

/*
1. (Main + Extra) - compressed ie. compresssed together

    Compression | Result
    ------------|----------
    all         | success

2. Main - compressed + Extra - raw

    Compression | Result
    ------------|----------
    brotli      | no error + main decompressed + extra no read
    deflate     | no error + main decompressed + extra read
    gzip        | no error + main decompressed + extra no read
    zstd        | error + main read + extra read

3. (Main - compressed) + (Extra - compressed) = compressed separately

    Compression | Result
    ------------|----------
    brotli      | main decompressed + extra no read
    deflate     | main decompressed + extra read
    gzip        | main decompressed + extra no read
    zstd        | success
*/

pub enum State<'a> {
    // Main
    MainOnly(DecompressionStruct<'a>),
    EndMainOnly(BytesMut),
    // Main + Extra
    ExtraTry(DecompressionStruct<'a>),
    ExtraDoneMainTry(DecompressionStruct<'a>, BytesMut),
    ExtraPlusMainTry(DecompressionStruct<'a>),
    ExtraRawMainTry(DecompressionStruct<'a>),
    // End
    EndExtraRawMainDone(DecompressionStruct<'a>, BytesMut),
    EndMainPlusExtra(BytesMut),
    EndExtraMainSeparate(BytesMut, BytesMut),
}

impl std::fmt::Debug for State<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::MainOnly(_) => write!(f, "MainOnly"),
            State::EndMainOnly(_) => write!(f, "EndMainOnly"),
            State::ExtraTry(_) => write!(f, "ExtraTry"),
            State::ExtraDoneMainTry(..) => write!(f, "ExtraDoneMainTry"),
            State::ExtraPlusMainTry(_) => write!(f, "ExtraPlusMainTry"),
            State::ExtraRawMainTry(_) => write!(f, "ExtraRawMainTry"),
            State::EndExtraRawMainDone(..) => write!(f, "EndExtraRawMainDone"),
            State::EndMainPlusExtra(_) => write!(f, "EndMainPlusExtra"),
            State::EndExtraMainSeparate(..) => {
                write!(f, "EndExtraMainSeparate")
            }
        }
    }
}

impl<'a> State<'a> {
    pub fn start(
        main: &'a [u8],
        extra: Option<&'a [u8]>,
        encodings: &'a mut [EncodingInfo],
        writer: Writer<&'a mut BytesMut>,
    ) -> Self {
        let dstruct = DecompressionStruct::new(main, extra, encodings, writer);
        if dstruct.extra.is_some() {
            Self::ExtraTry(dstruct)
        } else {
            Self::MainOnly(dstruct)
        }
    }

    fn try_next(self) -> Result<Self, MultiDecompressError> {
        let next_state = match self {
            State::MainOnly(mut dstruct) => {
                let result = dstruct.try_decompress_main()?;
                State::EndMainOnly(result)
            }
            /* Extra - is compressed
             *         true => try decompress
             *                   Ok  => ExtraDoneMainTry
             *                   Err => ExtraPlusMainTry
             *                          [ Maybe main + extra can decompress ]
             *         false => ExtraPlusMainTry
             */
            State::ExtraTry(mut dstruct) => {
                match dstruct.is_extra_compressed() {
                    true => match dstruct.try_decompress_extra() {
                        Ok(extra_decompressed) => State::ExtraDoneMainTry(
                            dstruct,
                            extra_decompressed,
                        ),
                        Err(_) => State::ExtraPlusMainTry(dstruct),
                    },
                    false => State::ExtraPlusMainTry(dstruct),
                }
            }
            /* Main - try decompress
             *        Ok  => EndExtraMainSeparate
             *        Err => ExtraPlusMainTry
             *               [ Maybe main + extra can decompress ]
             */
            State::ExtraDoneMainTry(mut dstruct, extra) => {
                match dstruct.try_decompress_main() {
                    Ok(main_decompressed) => {
                        State::EndExtraMainSeparate(main_decompressed, extra)
                    }
                    Err(_) => State::ExtraPlusMainTry(dstruct),
                }
            }
            /* Main + Extra - try decompress
             *      Ok  => EndMainPlusExtraDecompressed
             *      Err => ExtraRawMainTry
             */
            State::ExtraPlusMainTry(mut decompression_struct) => {
                match decompression_struct.try_decompress_main_plus_extra() {
                    Ok(main_plus_extra_decompressed) => {
                        State::EndMainPlusExtra(main_plus_extra_decompressed)
                    }
                    Err(_) => State::ExtraRawMainTry(decompression_struct),
                }
            }
            /* Main - try decompress
             *      Ok  => EndExtraRawMainDone
             *      Err => Err
             */
            State::ExtraRawMainTry(mut decompression_struct) => {
                match decompression_struct.try_decompress_main() {
                    Ok(main_decompressed) => State::EndExtraRawMainDone(
                        decompression_struct,
                        main_decompressed,
                    ),
                    Err(e) => return Err(e),
                }
            }
            State::EndMainOnly(_)
            | State::EndExtraRawMainDone(..)
            | State::EndMainPlusExtra(_)
            | State::EndExtraMainSeparate(..) => {
                panic!("already ended")
            }
        };
        Ok(next_state)
    }

    fn is_ended(&self) -> bool {
        matches!(self, Self::EndMainOnly(_))
            || matches!(self, Self::EndMainPlusExtra(_))
            || matches!(self, Self::EndExtraMainSeparate(..))
            || matches!(self, Self::EndExtraRawMainDone(..))
    }
}

impl<'a> From<State<'a>> for (BytesMut, Option<BytesMut>) {
    fn from(state: State) -> Self {
        match state {
            State::EndMainOnly(main) | State::EndMainPlusExtra(main) => {
                (main, None)
            }
            State::EndExtraMainSeparate(main, extra) => (main, Some(extra)),
            _ => unreachable!(),
        }
    }
}

pub fn runner<'a>(
    main: &'a [u8],
    extra: Option<&'a [u8]>,
    encodings: &'a mut [EncodingInfo],
    buf: &'a mut BytesMut,
) -> Result<State<'a>, MultiDecompressError> {
    let mut state = State::start(main, extra, encodings, buf.writer());
    loop {
        state = state.try_next()?;
        if state.is_ended() {
            return Ok(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    use crate::tests::*;

    // ----- Main
    fn assert_main_only_finish_flow(
        main: &[u8],
        extra: Option<&[u8]>,
        encoding_info: &mut Vec<EncodingInfo>,
    ) {
        let mut buf = BytesMut::new();
        let mut state =
            State::start(main, extra, encoding_info, (&mut buf).writer());
        assert!(matches!(state, State::MainOnly(_)));

        state = state.try_next().unwrap();
        assert!(state.is_ended());
        match state {
            State::EndMainOnly(out) => {
                assert_eq!(out, "hello world");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_state_main_only_single_compression_brotli() {
        let mut info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Brotli])];
        let compressed = compress_brotli(INPUT);
        assert_main_only_finish_flow(&compressed, None, &mut info);
    }

    #[test]
    fn test_state_main_only_single_compression_chunked() {
        let mut info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Chunked])];
        assert_main_only_finish_flow(INPUT, None, &mut info);
    }

    #[test]
    fn test_state_main_only_single_compression_identity() {
        let mut info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Identity])];
        assert_main_only_finish_flow(INPUT, None, &mut info);
    }

    #[test]
    #[test]
    fn test_state_main_only_multi_compression_single_header() {
        let mut info = all_encoding_info_single_header();
        let compressed = all_compressed_data();
        assert_main_only_finish_flow(&compressed, None, &mut info);
    }

    #[test]
    fn test_state_main_only_multi_compression_multi_header() {
        let mut info = all_encoding_info_multi_header();
        let compressed = all_compressed_data();
        assert_main_only_finish_flow(&compressed, None, &mut info);
    }

    // ----- Extra

    // Main + Extra - compressed together
    fn assert_main_plus_extra_flow(
        enc_info: &mut Vec<EncodingInfo>,
        compressed: &[u8],
    ) {
        let mid = compressed.len() / 2;
        let main_slice = &compressed[..mid];
        let extra_slice = &compressed[mid..];

        let mut buf = BytesMut::new();
        let mut state = State::start(
            main_slice,
            Some(extra_slice),
            enc_info,
            (&mut buf).writer(),
        );

        state = state.try_next().unwrap();
        assert!(matches!(state, State::ExtraPlusMainTry(_)));
        state = state.try_next().unwrap();
        assert!(state.is_ended());
        match state {
            State::EndMainPlusExtra(val) => assert_eq!(val, "hello world"),
            _ => panic!(),
        }
    }

    #[test]
    fn test_state_main_plus_extra_single_compression() {
        let mut info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Brotli])];
        let compressed = compress_brotli(INPUT);
        assert_main_plus_extra_flow(&mut info, &compressed);
    }

    #[test]
    fn test_state_main_plus_extra_compressed_together_single_header() {
        let mut info = all_encoding_info_single_header();
        let compressed = all_compressed_data();
        assert_main_plus_extra_flow(&mut info, &compressed);
    }

    #[test]
    fn test_state_main_plus_extra_compressed_together_multi_header() {
        let mut info = all_encoding_info_multi_header();
        let compressed = all_compressed_data();
        assert_main_plus_extra_flow(&mut info, &compressed);
    }

    // Main - separate + Extra - separate
    fn assert_main_separate_extra_separate_flow(
        enc_info: &mut Vec<EncodingInfo>,
        main: &[u8],
        extra: &[u8],
    ) {
        let mut buf = BytesMut::new();
        let mut state =
            State::start(main, Some(extra), enc_info, (&mut buf).writer());
        state = state.try_next().unwrap();
        assert!(
            matches!(state, State::ExtraDoneMainTry(_, ref result) if result == INPUT)
        );

        state = state.try_next().unwrap();
        assert!(state.is_ended());

        match state {
            State::EndExtraMainSeparate(main, extra) => {
                assert_eq!(main, INPUT);
                assert_eq!(extra, INPUT);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_state_main_separate_extra_separate_single_compression() {
        let mut info =
            vec![EncodingInfo::new(0, vec![ContentEncoding::Brotli])];
        let main = compress_brotli(INPUT);
        let extra = main.clone();
        assert_main_separate_extra_separate_flow(&mut info, &main, &extra);
    }

    #[test]
    fn test_state_main_separate_extra_separate_single_header() {
        let mut info = all_encoding_info_single_header();
        let main = all_compressed_data();
        let extra = main.clone();
        assert_main_separate_extra_separate_flow(&mut info, &main, &extra);
    }

    #[test]
    fn test_state_main_separate_extra_separate_multi_header() {
        let mut info = all_encoding_info_multi_header();
        let main = all_compressed_data();
        let extra = main.clone();
        assert_main_separate_extra_separate_flow(&mut info, &main, &extra);
    }

    // Main - compressed + Extra - raw
    fn assert_main_compressed_extra_raw_flow(
        enc_info: &mut Vec<EncodingInfo>,
        main: &[u8],
    ) {
        let mut buf = BytesMut::new();
        let mut state =
            State::start(main, Some(b"extra"), enc_info, (&mut buf).writer());
        state = state.try_next().unwrap();
        assert!(matches!(state, State::ExtraPlusMainTry(_)));
        state = state.try_next().unwrap();
        assert!(matches!(state, State::ExtraRawMainTry(_)));
        state = state.try_next().unwrap();
        assert!(state.is_ended());
        if let State::EndExtraRawMainDone(dstruct, result) = state {
            assert_eq!(result, INPUT);
            assert_eq!(dstruct.extra.unwrap(), b"extra");
        }
    }

    fn build_single_compression(encoding: ContentEncoding) {
        let main = single_compression(&encoding);
        let mut info = vec![EncodingInfo::new(0, vec![encoding])];
        assert_main_compressed_extra_raw_flow(&mut info, &main);
    }

    #[test]
    fn test_state_main_compressed_exra_raw_single_compression_gzip() {
        build_single_compression(ContentEncoding::Gzip);
    }

    #[test]
    fn test_state_main_compressed_exra_raw_single_compression_brotli() {
        build_single_compression(ContentEncoding::Brotli);
    }

    #[test]
    fn test_state_main_compressed_exra_raw_single_compression_deflate() {
        build_single_compression(ContentEncoding::Deflate);
    }

    #[test]
    fn test_state_main_compressed_exra_raw_single_compression_zstd() {
        build_single_compression(ContentEncoding::Zstd);
    }

    #[test]
    fn test_state_main_compressed_exra_raw_multi_compression_single_header() {
        let mut info = all_encoding_info_single_header();
        let main = all_compressed_data();
        assert_main_compressed_extra_raw_flow(&mut info, &main);
    }

    #[test]
    fn test_state_main_compressed_exra_raw_multi_compression_multi_header() {
        let mut info = all_encoding_info_multi_header();
        let main = all_compressed_data();
        assert_main_compressed_extra_raw_flow(&mut info, &main);
    }
}
