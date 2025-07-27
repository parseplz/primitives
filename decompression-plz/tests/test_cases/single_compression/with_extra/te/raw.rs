use tests_utils::INPUT;

use super::*;

#[test]
fn assert_decode_state_single_te_brotli_extra_raw() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Brotli,
        Some(INPUT.into()),
    );
}

#[test]
fn assert_decode_state_single_te_compress_extra_raw() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Compress,
        Some(INPUT.into()),
    );
}

#[test]
fn assert_decode_state_single_te_deflate_extra_raw() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Deflate,
        Some(INPUT.into()),
    );
}

#[test]
fn assert_decode_state_single_te_gzip_extra_raw() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Gzip,
        Some(INPUT.into()),
    );
}

#[test]
fn assert_decode_state_single_te_identity_extra_raw() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Identity,
        Some(INPUT.into()),
    );
}

#[test]
fn assert_decode_state_single_te_zstd_extra_raw() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Zstd,
        Some(INPUT.into()),
    );
}
