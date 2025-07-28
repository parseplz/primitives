use super::*;
use tests_utils::all_compressed_data;

#[test]
fn test_both_te_ce() {
    let body = all_compressed_data();
    let headers = format!(
        "Host: example.com\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        Transfer-Encoding: gzip, zstd\r\n\
        Content-Encoding: br, deflate\r\n\
        Content-Length: {}\r\n\r\n",
        body.len()
    );

    let mut tm =
        TestMessage::build(headers.as_bytes().into(), Body::Raw(body), None);
    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::TransferEncoding(..)));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::ContentEncoding(..)));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(_)));
    state = state.try_next().unwrap();
    assert!(state.is_ended());

    let result = tm.into_bytes();
    assert_eq!(result, VERIFY_SINGLE_HEADER_BODY_ONLY);
}
