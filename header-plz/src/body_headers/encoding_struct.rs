use crate::body_headers::content_encoding::ContentEncoding;

struct EncodingInfo {
    header_index: usize,
    encodings: ContentEncoding,
}
