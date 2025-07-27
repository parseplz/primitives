use tests_utils::INPUT;

use super::*;

#[test]
fn assert_decode_state_ce_all_single_header_extra_raw() {
    let tm = build_test_message_all_encodings_single_header(
        CONTENT_ENCODING,
        Some(INPUT.into()),
    );
    assert_case_multi_compression(
        tm,
        CONTENT_ENCODING,
        VERIFY_SINGLE_HEADER_BODY_AND_EXTRA,
    );
}
