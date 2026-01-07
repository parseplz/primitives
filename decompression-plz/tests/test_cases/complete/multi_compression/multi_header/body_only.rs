use super::*;
use rstest::rstest;

#[rstest]
#[case::content_encoding(CONTENT_ENCODING)]
#[case::transfer_encoding(TRANSFER_ENCODING)]
fn assert_decode_state_all_multi_header_one(#[case] header_type: &str) {
    let body = all_compressed_data();
    let tm = build_test_message_all_encodings_multi_header::<OneHeader>(
        header_type,
        &body,
        None,
    );
    assert_case_multi_compression_one(tm, header_type, VERIFY_MULTI_HEADER);
}

#[rstest]
#[case::content_encoding(CONTENT_ENCODING)]
#[case::transfer_encoding(TRANSFER_ENCODING)]
fn assert_decode_state_all_multi_header_two(#[case] header_type: &str) {
    let body = all_compressed_data();
    let tm = build_test_message_all_encodings_multi_header::<Header>(
        header_type,
        &body,
        None,
    );
    assert_case_multi_compression_two_multi_headers(tm, header_type);
}
