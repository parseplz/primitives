use super::*;

use rstest::rstest;

#[rstest]
#[case::content_encoding(CONTENT_ENCODING)]
#[case::transfer_encoding(TRANSFER_ENCODING)]
fn assert_decode_state_all_single_header_one(#[case] header_type: &str) {
    let tm = build_test_message_all_encodings_single_header::<OneHeader>(
        header_type,
        None,
    );
    assert_case_multi_compression_one(
        tm,
        header_type,
        VERIFY_SINGLE_HEADER_BODY_ONLY,
    );
}

#[rstest]
#[case::content_encoding(CONTENT_ENCODING)]
#[case::transfer_encoding(TRANSFER_ENCODING)]
fn assert_decode_state_all_single_header_two(#[case] header_type: &str) {
    let tm = build_test_message_all_encodings_single_header::<Header>(
        header_type,
        None,
    );
    assert_case_multi_compression_two(tm, header_type);
}
