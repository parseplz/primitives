use header_plz::message_head::header_map::HMap;

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

pub fn build_test_message_all_encodings_multi_header<T>(
    header_name: &str,
    body: &[u8],
    extra: Option<&[u8]>,
) -> TestMessage<T>
where
    T: From<OneHeader>,
    HMap<T>: From<HMap<header_plz::OneHeader>>,
{
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
