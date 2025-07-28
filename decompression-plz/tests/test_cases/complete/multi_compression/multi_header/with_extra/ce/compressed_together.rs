use super::*;

#[test]
fn assert_decode_state_ce_all_multi_header_extra_compressed_together() {
    let compressed = all_compressed_data();
    let (body, extra) = compressed.split_at(compressed.len() / 2);
    let tm = build_test_message_all_encodings_multi_header(
        CONTENT_ENCODING,
        body,
        Some(extra),
    );
    assert_case_multi_compression(tm, CONTENT_ENCODING, VERIFY_MULTI_HEADER);
}
