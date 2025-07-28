use super::*;

#[test]
fn assert_decode_state_te_all_multi_header_extra_raw() {
    let body = all_compressed_data();
    let tm = build_test_message_all_encodings_multi_header(
        TRANSFER_ENCODING,
        &body,
        None,
    );
    assert_case_multi_compression(tm, TRANSFER_ENCODING, VERIFY_MULTI_HEADER);
}
