use super::*;

use rstest::rstest;

#[rstest]
fn assert_decode_state_all_multi_header_extra_compressed_separate(
    #[values(TRANSFER_ENCODING, CONTENT_ENCODING)] header_type: &str,
) {
    let body = all_compressed_data();
    let tm = build_test_message_all_encodings_multi_header::<OneHeader>(
        header_type,
        &body,
        Some(&body),
    );
    assert_case_multi_compression_one(
        tm,
        header_type,
        VERIFY_MULTI_HEADER_EXTRA,
    );
}

#[rstest]
fn assert_decode_state_all_multi_header_extra_compressed_together(
    #[values(TRANSFER_ENCODING, CONTENT_ENCODING)] header_type: &str,
) {
    let compressed = all_compressed_data();
    let (body, extra) = compressed.split_at(compressed.len() / 2);
    let tm = build_test_message_all_encodings_multi_header::<OneHeader>(
        header_type,
        body,
        Some(extra),
    );
    assert_case_multi_compression_one(tm, header_type, VERIFY_MULTI_HEADER);
}

#[rstest]
fn assert_decode_state_all_multi_header_extra_raw(
    #[values(TRANSFER_ENCODING, CONTENT_ENCODING)] header_type: &str,
) {
    let body = all_compressed_data();
    let tm = build_test_message_all_encodings_multi_header::<OneHeader>(
        header_type,
        &body,
        None,
    );
    assert_case_multi_compression_one(tm, header_type, VERIFY_MULTI_HEADER);
}
