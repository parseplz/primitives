use super::*;

#[track_caller]
fn assert_partial_decompressed_single_header(
    encoding: &str,
    extra: Option<BytesMut>,
) {
    let headers = format!(
        "Host: example.com\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        {encoding}: br, deflate, gzip, {ALL_COMPRESSIONS}\r\n\
        Content-Length: 11\r\n\r\n",
    );

    let expected_state = encoding_state(encoding);

    let verify = if extra.is_none() {
        format!(
            "Host: example.com\r\n\
            Content-Type: text/html; charset=utf-8\r\n\
            {encoding}: br, deflate, gzip\r\n\
            Content-Length: 11\r\n\r\n\
            hello world",
        )
    } else {
        format!(
            "Host: example.com\r\n\
            Content-Type: text/html; charset=utf-8\r\n\
            {encoding}: br, deflate, gzip\r\n\
            Content-Length: 22\r\n\r\n\
            hello worldhello world"
        )
    };

    let mut tm = TestMessage::new(
        headers.as_bytes().into(),
        Body::Raw(all_compressed_data()),
        extra,
    );

    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!((expected_state)(&state));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLengthAndErr(..)));

    if let Err(e) = state.try_next() {
        assert!(matches!(e, MultiDecompressErrorReason::Partial { .. }));
        let result = tm.into_bytes();

        assert_eq!(result, verify);
    } else {
        panic!()
    }
}

#[test]
fn test_partial_te_single_header() {
    assert_partial_decompressed_single_header(TRANSFER_ENCODING, None);
}

#[test]
fn test_partial_te_single_header_extra_raw() {
    assert_partial_decompressed_single_header(
        TRANSFER_ENCODING,
        Some(INPUT.into()),
    );
}

// No implementation
//#[test]
//fn test_partial_te_single_header_extra_compressed_separate() {
//    assert_partial_decompressed_single_header(
//        TRANSFER_ENCODING,
//        Some(all_compressed_data()),
//    );
//}

// No implementation
// if fix needed
// decompression/state.rs:148
// add if error is partial
//
//#[test]
//fn test_partial_te_single_header_extra_compressed_together() {
//    assert_partial_decompressed_single_header_compressed_together(
//        TRANSFER_ENCODING,
//    );
//}

//
#[test]
fn test_partial_ce_single_header() {
    assert_partial_decompressed_single_header(CONTENT_ENCODING, None);
}

#[test]
fn test_partial_ce_single_header_extra_raw() {
    assert_partial_decompressed_single_header(
        CONTENT_ENCODING,
        Some(INPUT.into()),
    );
}
