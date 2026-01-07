use rstest::rstest;

use super::*;

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

#[rstest]
fn assert_decode_state_single_raw(
    #[values(TRANSFER_ENCODING, CONTENT_ENCODING)] encoding_type: &str,
    #[values(
        ContentEncoding::Brotli,
        ContentEncoding::Compress,
        ContentEncoding::Deflate,
        ContentEncoding::Gzip,
        ContentEncoding::Identity,
        ContentEncoding::Zstd
    )]
    encoding: ContentEncoding,
) {
    use tests_utils::INPUT;

    assert_case_single_compression_one(
        encoding_type,
        encoding,
        Some(INPUT.into()),
    );
}

#[rstest]
fn assert_decode_state_single_compressed_together(
    #[values(TRANSFER_ENCODING, CONTENT_ENCODING)] encoding_type: &str,
    #[values(
        ContentEncoding::Brotli,
        ContentEncoding::Compress,
        ContentEncoding::Deflate,
        ContentEncoding::Gzip,
        ContentEncoding::Identity,
        ContentEncoding::Zstd
    )]
    encoding: ContentEncoding,
) {
    assert_case_single_compression_compressed_together(
        encoding_type,
        encoding,
    );
}

#[rstest]
fn assert_decode_state_single_compressed_separate(
    #[values(TRANSFER_ENCODING, CONTENT_ENCODING)] encoding_type: &str,
    #[values(
        ContentEncoding::Brotli,
        ContentEncoding::Compress,
        ContentEncoding::Deflate,
        ContentEncoding::Gzip,
        ContentEncoding::Identity,
        ContentEncoding::Zstd
    )]
    encoding: ContentEncoding,
) {
    let extra = single_compression(&encoding);
    assert_case_single_compression_one(encoding_type, encoding, Some(extra));
}
