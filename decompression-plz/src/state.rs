use bytes::BytesMut;
use header_plz::body_headers::content_encoding::ContentEncoding;

use crate::dstruct::DecompressionStruct;

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
    fn new(
        main: BytesMut,
        extra: Option<BytesMut>,
        encodings: &'a [ContentEncoding],
        buf: &'a mut BytesMut,
    ) -> Self {
        let d = DecompressionStruct::new(main, extra, encodings, buf);
        if d.extra.is_some() {
            Self::Extra(d)
        } else {
            Self::MainOnly(d)
        }
    }
}

/*
1. Extra
    true    => Main
    false   => Main + Extra

2. Main
    true    => Main_decompressed + Extra_decompressed
    false   => Err()

3. Main + Extra
    true    => Main_and_Extra_decompressed
    false   => Err()
*/

// 1. Try decompressing extra
// 2. If success, try decompressing main
// 2. If failed, try decompressing main + extra
// 3. If failed, try decompressing main
