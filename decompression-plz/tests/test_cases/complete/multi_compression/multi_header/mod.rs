use header_plz::const_headers::{CONTENT_ENCODING, TRANSFER_ENCODING};

use super::*;
mod body_only;
mod with_extra;

const VERIFY_MULTI_HEADER: &str = "Host: example.com\r\n\
                                   Content-Type: text/html; charset=utf-8\r\n\
                                   random: random\r\n\
                                   another-random: random\r\n\
                                   test-header: test-header\r\n\
                                   Content-Length: 11\r\n\r\n\
                                   hello world";

const VERIFY_MULTI_HEADER_EXTRA: &str = "Host: example.com\r\n\
                                   Content-Type: text/html; charset=utf-8\r\n\
                                   random: random\r\n\
                                   another-random: random\r\n\
                                   test-header: test-header\r\n\
                                   Content-Length: 22\r\n\r\n\
                                   hello worldhello world";

fn build_test_message_all_encodings_multi_header(
    header_name: &str,
    body: &[u8],
    extra: Option<&[u8]>,
) -> TestMessage {
    let headers = format!(
        "Host: example.com\r\n\
        {}: br\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        {}: deflate\r\n\
        random: random\r\n\
        {}: identity\r\n\
        another-random: random\r\n\
        {}: gzip\r\n\
        test-header: test-header\r\n\
        {}: zstd\r\n\
        Content-Length: {}\r\n\r\n",
        header_name,
        header_name,
        header_name,
        header_name,
        header_name,
        body.len()
    );

    TestMessage::new(
        headers.as_bytes().into(),
        Body::Raw(body.into()),
        extra.map(BytesMut::from),
    )
}
