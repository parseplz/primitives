use header_plz::body_headers::content_encoding::ContentEncoding;
use tests_utils::single_compression;

use super::*;

fn run_case(case: &Case, content_encoding: ContentEncoding) {
    let body: Vec<u8> = single_compression(&content_encoding);
    let headers = format!(
        "Host: example.com\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         {}: {}\r\n\
         Content-Length: {}\r\n\r\n",
        case.header_name,
        content_encoding.as_ref(),
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
    assert!((case.expected_state)(&state));
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

#[test]
fn assert_decode_states_single() {
    let case = Case {
        header_name: "Transfer-Encoding",
        expected_state: |s| matches!(s, DecodeState::TransferEncoding(_, _)),
    };
    run_case(&case, ContentEncoding::Brotli);
}
