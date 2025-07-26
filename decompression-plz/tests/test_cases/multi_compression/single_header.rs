use header_plz::const_headers::{CONTENT_ENCODING, TRANSFER_ENCODING};

use super::*;

fn build_test_message_all_encodings_single_header(
    header_name: &str,
) -> TestMessage {
    let body: Vec<u8> = all_compressed_data();
    let headers = format!(
        "Host: example.com\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         {}: {}\r\n\
         Content-Length: {}\r\n\r\n",
        header_name,
        ALL_COMPRESSIONS,
        body.len()
    );

    let mut tm = TestMessage::build(
        headers.as_bytes().into(),
        Body::Raw(body.as_slice().into()),
        None,
    );
    tm
}

const VERIFY: &str = "Host: example.com\r\n\
                Content-Type: text/html; charset=utf-8\r\n\
                Content-Length: 11\r\n\r\n\
                hello world";

#[test]
fn assert_decode_state_ce_all_single_header() {
    let tm = build_test_message_all_encodings_single_header(CONTENT_ENCODING);

    let f = move |s: &DecodeState<TestMessage>| {
        matches!(s, DecodeState::ContentEncoding(_, _))
    };
    assert_case_multi_compression(f, tm, VERIFY);
}

#[test]
fn assert_decode_state_te_all_single_header() {
    let tm = build_test_message_all_encodings_single_header(TRANSFER_ENCODING);
    let f = move |s: &DecodeState<TestMessage>| {
        matches!(s, DecodeState::TransferEncoding(_, _))
    };
    assert_case_multi_compression(f, tm, VERIFY);
}
