use header_plz::const_headers::{CONTENT_ENCODING, TRANSFER_ENCODING};

use super::*;

const VERIFY: &str = "Host: example.com\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        random: random\r\n\
        another-random: random\r\n\
        test-header: test-header\r\n\
        Content-Length: 11\r\n\r\n\
        hello world";

fn build_test_message_all_encodings_multi_header(
    header_name: &str,
) -> TestMessage {
    let body: Vec<u8> = all_compressed_data();
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

    TestMessage::build(
        headers.as_bytes().into(),
        Body::Raw(body.as_slice().into()),
        None,
    )
}

#[test]
fn assert_decode_state_ce_all_multi_header() {
    let tm = build_test_message_all_encodings_multi_header(CONTENT_ENCODING);

    assert_case_multi_compression(tm, CONTENT_ENCODING, VERIFY);
}

#[test]
fn assert_decode_state_te_all_multi_header() {
    let tm = build_test_message_all_encodings_multi_header(TRANSFER_ENCODING);
    assert_case_multi_compression(tm, TRANSFER_ENCODING, VERIFY);
}
