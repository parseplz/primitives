use super::*;
use body_plz::variants::Body;
use bytes::BytesMut;
use decompression_plz::state::DecodeState;

#[test]
fn test_decode_init_no_enc() {
    let headers = "Host: example.com\r\n\
                       Content-Type: text/html; charset=utf-8\r\n\
                       Content-Length: 11\r\n\r\n";
    let mut tm =
        TestMessage::build(headers.into(), Body::Raw(INPUT.into()), None);
    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::End));
    let result = tm.into_bytes();
    let verify = "Host: example.com\r\n\
                      Content-Type: text/html; charset=utf-8\r\n\
                      Content-Length: 11\r\n\r\n\
                      hello world";
    assert_eq!(result, verify);
}

#[test]
fn test_decode_init_no_enc_extra_body() {
    let headers = "Host: example.com\r\n\
                       Content-Type: text/html; charset=utf-8\r\n\
                       Content-Length: 11\r\n\r\n";
    let mut tm = TestMessage::build(
        headers.into(),
        Body::Raw(INPUT.into()),
        Some(INPUT.into()),
    );

    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(_)));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::End));
    let result = tm.into_bytes();
    let verify = "Host: example.com\r\n\
                      Content-Type: text/html; charset=utf-8\r\n\
                      Content-Length: 22\r\n\r\n\
                      hello worldhello world";
    assert_eq!(result, verify);
}
