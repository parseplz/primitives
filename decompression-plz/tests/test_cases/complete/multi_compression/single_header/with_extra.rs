use rstest::rstest;

use super::*;

#[rstest]
fn assert_decode_state_all_single_header_extra_compressed_separate(
    #[values(TRANSFER_ENCODING, CONTENT_ENCODING)] header_type: &str,
) {
    let tm = build_test_message_all_encodings_single_header::<OneHeader>(
        header_type,
        Some(all_compressed_data()),
    );
    assert_case_multi_compression_one(
        tm,
        header_type,
        VERIFY_SINGLE_HEADER_BODY_AND_EXTRA,
    );
}

#[rstest]
fn assert_decode_state_all_single_header_extra_compressed_together(
    #[values(TRANSFER_ENCODING, CONTENT_ENCODING)] header_type: &str,
) {
    let tm =
        build_test_message_all_encodings_single_header_compressed_together(
            header_type,
        );
    assert_case_multi_compression_one(
        tm,
        header_type,
        VERIFY_SINGLE_HEADER_BODY_ONLY,
    );
}

#[rstest]
fn assert_decode_state_all_single_header_extra_raw(
    #[values(TRANSFER_ENCODING, CONTENT_ENCODING)] header_type: &str,
) {
    use tests_utils::INPUT;

    let tm = build_test_message_all_encodings_single_header::<OneHeader>(
        header_type,
        Some(INPUT.into()),
    );
    assert_case_multi_compression_one(
        tm,
        header_type,
        VERIFY_SINGLE_HEADER_BODY_AND_EXTRA,
    );
}
