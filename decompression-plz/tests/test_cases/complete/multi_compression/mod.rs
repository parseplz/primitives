use decompression_plz::DecompressTrait;
use header_plz::{Header, OneHeader, const_headers::CONTENT_LENGTH};
use tests_utils::{ALL_COMPRESSIONS, all_compressed_data};

use crate::test_cases::complete::multi_compression::{
    multi_header::build_test_message_all_encodings_multi_header,
    single_header::build_test_message_all_encodings_single_header,
};

use super::*;
mod multi_header;
mod single_header;

fn assert_case_multi_compression_one(
    mut tm: TestMessage<OneHeader>,
    header: &str,
    verify: &str,
) {
    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    let encoding_state = encoding_state(header);
    assert!((encoding_state)(&state));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(_)));
    state = state.try_next().unwrap();
    assert!(state.is_ended());

    let result = tm.into_bytes();
    assert_eq!(result, verify);
}

fn assert_case_multi_compression_two(
    mut tm: TestMessage<Header>,
    header: &str,
) {
    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    let encoding_state = encoding_state(header);
    assert!((encoding_state)(&state));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(_)));
    state = state.try_next().unwrap();
    assert!(state.is_ended());

    let mut verify =
        build_test_message_all_encodings_single_header::<Header>(header, None);

    verify.set_body(Body::Raw("hello world".into()));

    assert!(verify.header_map_as_mut().remove_header_on_key(header));
    assert!(
        verify
            .header_map_as_mut()
            .update_header_value_on_key(CONTENT_LENGTH, b"11")
    );
    assert_eq!(verify, tm);
}

fn assert_case_multi_compression_two_multi_headers(
    mut tm: TestMessage<Header>,
    header: &str,
) {
    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    let encoding_state = encoding_state(header);
    assert!((encoding_state)(&state));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(_)));
    state = state.try_next().unwrap();
    assert!(state.is_ended());

    let body = all_compressed_data();
    let mut verify = build_test_message_all_encodings_multi_header::<Header>(
        header, &body, None,
    );

    verify.set_body(Body::Raw("hello world".into()));

    assert!(verify.header_map_as_mut().remove_header_on_key_all(header));
    assert!(
        verify
            .header_map_as_mut()
            .update_header_value_on_key(CONTENT_LENGTH, b"11")
    );
    assert_eq!(verify, tm);
}
