use bytes::BytesMut;
use header_plz::body_headers::content_encoding::ContentEncoding;

pub struct DecompressionStruct<'a> {
    main: BytesMut,
    pub extra: Option<BytesMut>,
    encodings: &'a [ContentEncoding],
    buf: &'a mut BytesMut,
}

impl<'a> DecompressionStruct<'a> {
    pub fn new(
        main: BytesMut,
        extra: Option<BytesMut>,
        encodings: &'a [ContentEncoding],
        buf: &'a mut BytesMut,
    ) -> Self {
        Self {
            main,
            extra,
            encodings,
            buf,
        }
    }
}
