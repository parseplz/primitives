use tests_utils::{ALL_COMPRESSIONS, all_compressed_data};

use super::*;
mod multi_header;
mod single_header;

fn assert_case_multi_compression(
    mut tm: TestMessage,
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
