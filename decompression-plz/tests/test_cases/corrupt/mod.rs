use decompression_plz::MultiDecompressErrorReason;
use header_plz::body_headers::content_encoding::ContentEncoding;
use tests_utils::INPUT;

use super::*;
mod ce;
mod te;

fn assert_corrupt_encoding(
    encoding: &str,
    compression: &ContentEncoding,
    extra: Option<BytesMut>,
) {
    let headers = format!(
        "Host: example.com\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        {}: {}\r\n\
        Content-Length: 11\r\n\r\n",
        encoding,
        compression.as_ref()
    );

    let verify = format!(
        "Host: example.com\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        {}: {}\r\n\
        Content-Length: {}\r\n\r\n\
        hello world{}",
        encoding,
        compression.as_ref(),
        11 + extra.as_ref().map_or(0, |b| b.len()),
        extra
            .as_ref()
            .map_or(String::new(), |b| String::from_utf8_lossy(b).to_string())
    );

    let expected_state = encoding_state(encoding);

    let mut tm = TestMessage::new(
        headers.as_bytes().into(),
        Body::Raw(INPUT.into()),
        extra,
    );

    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!((expected_state)(&state));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLengthAndErr(..)));

    if let Err(e) = state.try_next() {
        assert!(matches!(e, MultiDecompressErrorReason::Corrupt));
        let result = tm.into_bytes();
        assert_eq!(result, verify);
    } else {
        panic!()
    }
}

#[test]
fn test_corrupt_te_gzip_extra() {
    assert_corrupt_encoding(
        TRANSFER_ENCODING,
        &ContentEncoding::Gzip,
        Some(INPUT.into()),
    );
}

#[test]
fn test_corrupt_ce_gzip() {
    assert_corrupt_encoding(CONTENT_ENCODING, &ContentEncoding::Gzip, None);
}
