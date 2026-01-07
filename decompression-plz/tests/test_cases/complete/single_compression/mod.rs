use decompression_plz::DecompressTrait;
use header_plz::{
    Header, OneHeader,
    body_headers::content_encoding::ContentEncoding,
    const_headers::{CONTENT_ENCODING, CONTENT_LENGTH, TRANSFER_ENCODING},
};
use tests_utils::single_compression;

use super::*;
mod body_only;
mod with_extra;

fn assert_case_single_compression_one(
    header_name: &str,
    encoding: ContentEncoding,
    extra: Option<BytesMut>,
) {
    let body = single_compression(&encoding);
    let headers = format!(
        "Host: example.com\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        {}: {}\r\n\
        Content-Length: {}\r\n\r\n",
        header_name,
        encoding.as_ref(),
        body.len()
    );

    let expected_state = encoding_state(header_name);
    let with_extra = extra.is_some();
    let mut tm = TestMessage::<OneHeader>::new(
        headers.as_bytes().into(),
        Body::Raw(body),
        extra,
    );
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

pub fn assert_case_single_compression_two(
    header: &str,
    encoding: ContentEncoding,
) {
    let body = single_compression(&encoding);
    let raw_headers = format!(
        "Host: example.com\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        {}: {}\r\n\
        Content-Length: {}\r\n\r\n",
        header,
        encoding.as_ref(),
        body.len()
    );

    let expected_state = encoding_state(header);

    let mut tm = TestMessage::<Header>::new(
        raw_headers.as_bytes().into(),
        Body::Raw(body),
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

    let mut verify = TestMessage::<Header>::new(
        raw_headers.as_bytes().into(),
        Body::Raw("hello world".into()),
        None,
    );

    assert!(verify.header_map_as_mut().remove_header_on_key(header));
    assert!(
        verify
            .header_map_as_mut()
            .update_header_value_on_key(CONTENT_LENGTH, "11")
    );

    assert_eq!(tm, verify);
}
