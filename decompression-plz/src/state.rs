use crate::{
    decompression::{
        magic_bytes::is_compressed,
        multi::{decompress_multi, error::MultiDecompressError},
    },
    dstruct::DecompressionStruct,
    error::DecompressErrorStruct,
};
use bytes::{BufMut, BytesMut, buf::Writer};
use header_plz::body_headers::{
    content_encoding::ContentEncoding, encoding_info::EncodingInfo,
};

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
    EndExtraRawMainDone(DecompressionStruct<'a>),
    EndMainPlusExtra(BytesMut),
    EndExtraMainSeparate(BytesMut, BytesMut),
}

impl std::fmt::Debug for State<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::MainOnly(_) => write!(f, "MainOnly"),
            State::EndMainOnly(_) => write!(f, "EndMainOnly"),
            State::ExtraTry(_) => write!(f, "ExtraTry"),
            State::ExtraDoneMainTry(_, _) => write!(f, "ExtraDoneMainTry"),
            State::ExtraPlusMainTry(_) => write!(f, "ExtraPlusMainTry"),
            State::ExtraRawMainTry(_) => write!(f, "ExtraRawMainTry"),
            State::EndExtraRawMainDone(_) => write!(f, "EndExtraRawMainDone"),
            State::EndMainPlusExtra(_) => write!(f, "EndMainPlusExtra"),
            State::EndExtraMainSeparate(_, _) => {
                write!(f, "EndExtraMainSeparate")
            }
        }
    }
}

impl<'a> State<'a> {
    pub fn start(
        main: &'a [u8],
        extra: Option<&'a [u8]>,
        encodings: &'a [EncodingInfo],
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
                    Ok(main_decompressed) => {
                        State::EndExtraRawMainDone(decompression_struct)
                    }
                    Err(e) => return Err(e),
                }
            }
            State::EndExtraRawMainDone(decompression_struct) => todo!(),
            State::EndMainOnly(_)
            | State::EndMainPlusExtra(_)
            | State::EndExtraMainSeparate(..) => {
                panic!("already ended")
            }
        };
        Ok(next_state)
    }

    fn ended(&self) -> bool {
        matches!(self, Self::EndMainOnly(_))
            || matches!(self, Self::EndExtraMainSeparate(..))
            || matches!(self, Self::EndMainPlusExtra(_))
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
    encodings: &'a [EncodingInfo],
    buf: &'a mut BytesMut,
) -> Result<State<'a>, MultiDecompressError> {
    let mut state = State::start(main, extra, encodings, buf.writer());
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
        let result = runner(&input, None, &einfo, &mut buf).unwrap();
        if let State::EndMainOnly(main) = result {
            assert_eq!(main, "hello world");
        } else {
            panic!()
        }
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
        let main = &compressed[..compressed.len() / 2];
        let extra = &compressed[compressed.len() / 2..];
        let mut buf = BytesMut::new();
        let result = runner(&main, Some(&extra), &einfo, &mut buf).unwrap();
        if let State::EndMainPlusExtra(main) = result {
            assert_eq!(main, "hello world");
        } else {
            panic!()
        }
    }

    #[test]
    fn test_state_main_extra_compressed_together_multi_header() {
        let einfo = all_encoding_info_multi_header();
        let compressed = all_compressed_data();
        let main = &compressed[..compressed.len() / 2];
        let extra = &compressed[compressed.len() / 2..];
        let mut buf = BytesMut::new();
        let result = runner(&main, Some(&extra), &einfo, &mut buf).unwrap();
        if let State::EndMainPlusExtra(main) = result {
            assert_eq!(main, "hello world");
        } else {
            panic!()
        }
    }
}
