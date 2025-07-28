use super::*;

#[test]
fn assert_decode_state_te_all_multi_header_extra_compressed_together() {
    let compressed = all_compressed_data();
    let (body, extra) = compressed.split_at(compressed.len() / 2);
    let tm = build_test_message_all_encodings_multi_header(
        TRANSFER_ENCODING,
        body,
        Some(extra),
    );
    assert_case_multi_compression(tm, TRANSFER_ENCODING, VERIFY_MULTI_HEADER);
}
