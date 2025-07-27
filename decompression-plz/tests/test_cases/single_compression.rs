use header_plz::{
    body_headers::content_encoding::ContentEncoding,
    const_headers::{CONTENT_ENCODING, TRANSFER_ENCODING},
};
use tests_utils::single_compression;

use super::*;

fn assert_case_single_compression(
    header_name: &str,
    content_encoding: ContentEncoding,
) {
    let body: Vec<u8> = single_compression(&content_encoding);
    let headers = format!(
        "Host: example.com\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         {}: {}\r\n\
         Content-Length: {}\r\n\r\n",
        header_name,
        content_encoding.as_ref(),
        body.len()
    );

    let expected_state = encoding_state(header_name);

    let mut tm = TestMessage::build(
        headers.as_bytes().into(),
        Body::Raw(body.as_slice().into()),
        None,
    );
    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!((expected_state)(&state));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(_)));
    state = state.try_next().unwrap();
    assert!(state.is_ended());

    let result = tm.into_bytes();
    let verify = "Host: example.com\r\n\
                  Content-Type: text/html; charset=utf-8\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(result, verify);
}

// Transfer-Encoding

#[test]
fn assert_decode_state_single_te_brotli() {
    assert_case_single_compression(TRANSFER_ENCODING, ContentEncoding::Brotli);
}

#[test]
fn assert_decode_state_single_te_compress() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Compress,
    );
}

#[test]
fn assert_decode_state_single_te_deflate() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Deflate,
    );
}

#[test]
fn assert_decode_state_single_te_gzip() {
    assert_case_single_compression(TRANSFER_ENCODING, ContentEncoding::Gzip);
}

#[test]
fn assert_decode_state_single_te_identity() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Identity,
    );
}

#[test]
fn assert_decode_state_single_te_zstd() {
    assert_case_single_compression(TRANSFER_ENCODING, ContentEncoding::Zstd);
}

// CE only
#[test]
fn assert_decode_state_single_ce_brotli() {
    assert_case_single_compression(CONTENT_ENCODING, ContentEncoding::Brotli);
}

#[test]
fn assert_decode_state_single_ce_compress() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Compress,
    );
}

#[test]
fn assert_decode_state_single_ce_deflate() {
    assert_case_single_compression(CONTENT_ENCODING, ContentEncoding::Deflate);
}

#[test]
fn assert_decode_state_single_ce_gzip() {
    assert_case_single_compression(CONTENT_ENCODING, ContentEncoding::Gzip);
}

#[test]
fn assert_decode_state_single_ce_identity() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Identity,
    );
}

#[test]
fn assert_decode_state_single_ce_zstd() {
    assert_case_single_compression(CONTENT_ENCODING, ContentEncoding::Zstd);
}
