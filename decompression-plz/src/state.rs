use crate::{dstruct::DecompressionStruct, error::DecompressErrorStruct};
use bytes::BytesMut;
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

enum State<'a> {
    // Main
    MainOnly(DecompressionStruct<'a>),
    EndMainOnly(DecompressionStruct<'a>),
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

    fn try_next(self) -> Result<Self, DecompressErrorStruct> {
        todo!()
    }

    fn ended(self) -> bool {
        matches!(self, Self::EndMainOnly(_)) || matches!(self, Self::EndMainOnyDecompressed(_))
    }
}
