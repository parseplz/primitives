use super::*;

#[test]
fn assert_decode_state_te_all_single_header() {
    let tm = build_test_message_all_encodings_single_header(
        TRANSFER_ENCODING,
        None,
    );
    assert_case_multi_compression(
        tm,
        TRANSFER_ENCODING,
        VERIFY_SINGLE_HEADER_BODY_ONLY,
    );
}
