use tests_utils::INPUT;

use super::*;

#[test]
fn assert_decode_state_single_ce_brotli_extra_raw() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Brotli,
        Some(INPUT.into()),
    );
}

#[test]
fn assert_decode_state_single_ce_compress_extra_raw() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Compress,
        Some(INPUT.into()),
    );
}

#[test]
fn assert_decode_state_single_ce_deflate_extra_raw() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Deflate,
        Some(INPUT.into()),
    );
}

#[test]
fn assert_decode_state_single_ce_gzip_extra_raw() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Gzip,
        Some(INPUT.into()),
    );
}

#[test]
fn assert_decode_state_single_ce_identity_extra_raw() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Identity,
        Some(INPUT.into()),
    );
}

#[test]
fn assert_decode_state_single_ce_zstd_extra_raw() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Zstd,
        Some(INPUT.into()),
    );
}
