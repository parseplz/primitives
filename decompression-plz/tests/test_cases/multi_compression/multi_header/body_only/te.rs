use super::*;

#[test]
fn assert_decode_state_te_all_multi_header() {
    let tm = build_test_message_all_encodings_multi_header(TRANSFER_ENCODING);
    assert_case_multi_compression(tm, TRANSFER_ENCODING, VERIFY_MULTI_HEADER);
}
