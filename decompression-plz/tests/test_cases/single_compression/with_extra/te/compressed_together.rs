use super::*;

#[test]
fn assert_decode_state_single_te_brotli_extra_compressed_together() {
    assert_case_single_compression_compressed_together(
        TRANSFER_ENCODING,
        ContentEncoding::Brotli,
    );
}

#[test]
fn assert_decode_state_single_te_compress_extra_compressed_together() {
    assert_case_single_compression_compressed_together(
        TRANSFER_ENCODING,
        ContentEncoding::Compress,
    );
}

#[test]
fn assert_decode_state_single_te_deflate_extra_compressed_together() {
    assert_case_single_compression_compressed_together(
        TRANSFER_ENCODING,
        ContentEncoding::Deflate,
    );
}

#[test]
fn assert_decode_state_single_te_gzip_extra_compressed_together() {
    assert_case_single_compression_compressed_together(
        TRANSFER_ENCODING,
        ContentEncoding::Gzip,
    );
}

#[test]
fn assert_decode_state_single_te_identity_extra_compressed_together() {
    assert_case_single_compression_compressed_together(
        TRANSFER_ENCODING,
        ContentEncoding::Identity,
    );
}

#[test]
fn assert_decode_state_single_te_zstd_extra_compressed_together() {
    assert_case_single_compression_compressed_together(
        TRANSFER_ENCODING,
        ContentEncoding::Zstd,
    );
}
