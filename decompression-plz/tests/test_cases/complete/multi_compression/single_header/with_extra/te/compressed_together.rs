use super::*;

#[test]
fn assert_decode_state_te_all_single_header_extra_compressed_together() {
    let tm =
        build_test_message_all_encodings_single_header_compressed_together(
            TRANSFER_ENCODING,
        );
    assert_case_multi_compression(
        tm,
        TRANSFER_ENCODING,
        VERIFY_SINGLE_HEADER_BODY_ONLY,
    );
}
