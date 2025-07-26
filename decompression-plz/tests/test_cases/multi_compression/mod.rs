use header_plz::body_headers::content_encoding::ContentEncoding;
use tests_utils::{ALL_COMPRESSIONS, all_compressed_data};

use super::*;
mod multi_header;
mod single_header;

fn assert_case_multi_compression(
    f: fn(&DecodeState<TestMessage>) -> bool,
    mut tm: TestMessage,
    verify: &str,
) {
    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!((f)(&state));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(_)));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::End));

    let result = tm.into_bytes();
    assert_eq!(result, verify);
}
