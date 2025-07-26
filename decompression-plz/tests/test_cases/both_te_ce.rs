use super::*;
use tests_utils::all_compressed_data;

#[test]
fn test_both_te_ce() {
    let body: Vec<u8> = all_compressed_data();
    let headers = format!(
        "Host: example.com\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Transfer-Encoding: gzip, zstd\r\n\
         Content-Encoding: br, deflate\r\n\
         Content-Length: {}\r\n\r\n",
        body.len()
    );

    let mut tm = TestMessage::build(
        headers.as_bytes().into(),
        Body::Raw(body.as_slice().into()),
        None,
    );
    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::TransferEncoding(..)));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::ContentEncoding(..)));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(_)));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::End));

    let result = tm.into_bytes();
    let verify = "Host: example.com\r\n\
                  Content-Type: text/html; charset=utf-8\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(result, verify);
}
