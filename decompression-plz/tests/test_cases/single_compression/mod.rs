use header_plz::{
    body_headers::content_encoding::ContentEncoding,
    const_headers::{CONTENT_ENCODING, TRANSFER_ENCODING},
};
use tests_utils::single_compression;

use super::*;
mod body_only;
mod with_extra;

fn assert_case_single_compression(
    header_name: &str,
    content_encoding: ContentEncoding,
    extra: Option<BytesMut>,
) {
    let body = single_compression(&content_encoding);
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
    let with_extra = extra.is_some();
    let mut tm =
        TestMessage::build(headers.as_bytes().into(), Body::Raw(body), extra);
    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!((expected_state)(&state));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(_)));
    state = state.try_next().unwrap();
    assert!(state.is_ended());

    let verify = if with_extra {
        VERIFY_SINGLE_HEADER_BODY_AND_EXTRA
    } else {
        VERIFY_SINGLE_HEADER_BODY_ONLY
    };
    let result = tm.into_bytes();
    assert_eq!(result, verify);
}
