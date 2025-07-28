use super::*;

#[test]
fn assert_decode_state_single_ce_brotli_extra_compressed_together() {
    assert_case_single_compression_compressed_together(
        CONTENT_ENCODING,
        ContentEncoding::Brotli,
    );
}

#[test]
fn assert_decode_state_single_ce_compress_extra_compressed_together() {
    assert_case_single_compression_compressed_together(
        CONTENT_ENCODING,
        ContentEncoding::Compress,
    );
}

#[test]
fn assert_decode_state_single_ce_deflate_extra_compressed_together() {
    assert_case_single_compression_compressed_together(
        CONTENT_ENCODING,
        ContentEncoding::Deflate,
    );
}

#[test]
fn assert_decode_state_single_ce_gzip_extra_compressed_together() {
    assert_case_single_compression_compressed_together(
        CONTENT_ENCODING,
        ContentEncoding::Gzip,
    );
}

#[test]
fn assert_decode_state_single_ce_identity_extra_compressed_together() {
    assert_case_single_compression_compressed_together(
        CONTENT_ENCODING,
        ContentEncoding::Identity,
    );
}

#[test]
fn assert_decode_state_single_ce_zstd_extra_compressed_together() {
    assert_case_single_compression_compressed_together(
        CONTENT_ENCODING,
        ContentEncoding::Zstd,
    );
}
