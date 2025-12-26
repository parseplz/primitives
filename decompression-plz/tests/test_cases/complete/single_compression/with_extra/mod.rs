use super::*;
mod ce;
mod te;

pub fn assert_case_single_compression_compressed_together(
    header_name: &str,
    content_encoding: ContentEncoding,
) {
    let compressed = single_compression(&content_encoding);
    let (body, extra) = compressed.split_at(compressed.len() / 2);
    let headers = format!(
        "Host: example.com\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        {}: {}\r\n\
        Content-Length: {}\r\n\r\n",
        header_name,
        content_encoding.as_ref(),
        compressed.len()
    );

    let expected_state = encoding_state(header_name);
    let mut tm = TestMessage::new(
        headers.as_bytes().into(),
        Body::Raw(body.into()),
        Some(extra.into()),
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
    assert_eq!(result, VERIFY_SINGLE_HEADER_BODY_ONLY);
}
