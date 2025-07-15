use bytes::BytesMut;
use header_plz::body_headers::content_encoding::ContentEncoding;

struct DecompressionStruct<'a> {
    main: BytesMut,
    extra: Option<BytesMut>,
    encodings: &'a [ContentEncoding],
    buf: &'a mut BytesMut,
}

impl<'a> DecompressionStruct<'a> {
    fn new(
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
