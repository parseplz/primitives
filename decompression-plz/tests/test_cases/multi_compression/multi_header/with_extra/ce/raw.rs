use super::*;

#[test]
fn assert_decode_state_ce_all_multi_header_extra_raw() {
    let body = all_compressed_data();
    let tm = build_test_message_all_encodings_multi_header(
        CONTENT_ENCODING,
        &body,
        None,
    );
    assert_case_multi_compression(tm, CONTENT_ENCODING, VERIFY_MULTI_HEADER);
}
