use super::*;

#[test]
fn assert_decode_state_single_te_brotli_extra_compressed_separate() {
    let content_encoding = ContentEncoding::Brotli;
    let extra = single_compression(&content_encoding);
    assert_case_single_compression(
        TRANSFER_ENCODING,
        content_encoding,
        Some(extra),
    );
}

#[test]
fn assert_decode_state_single_te_compress_extra_compressed_separate() {
    let content_encoding = ContentEncoding::Compress;
    let extra = single_compression(&content_encoding);
    assert_case_single_compression(
        TRANSFER_ENCODING,
        content_encoding,
        Some(extra),
    );
}

#[test]
fn assert_decode_state_single_te_deflate_extra_compressed_separate() {
    let content_encoding = ContentEncoding::Deflate;
    let extra = single_compression(&content_encoding);
    assert_case_single_compression(
        TRANSFER_ENCODING,
        content_encoding,
        Some(extra),
    );
}

#[test]
fn assert_decode_state_single_te_gzip_extra_compressed_separate() {
    let content_encoding = ContentEncoding::Gzip;
    let extra = single_compression(&content_encoding);
    assert_case_single_compression(
        TRANSFER_ENCODING,
        content_encoding,
        Some(extra),
    );
}

#[test]
fn assert_decode_state_single_te_identity_extra_compressed_separate() {
    let content_encoding = ContentEncoding::Identity;
    let extra = single_compression(&content_encoding);
    assert_case_single_compression(
        TRANSFER_ENCODING,
        content_encoding,
        Some(extra),
    );
}

#[test]
fn assert_decode_state_single_te_zstd_extra_compressed_separate() {
    let content_encoding = ContentEncoding::Zstd;
    let extra = single_compression(&content_encoding);
    assert_case_single_compression(
        TRANSFER_ENCODING,
        content_encoding,
        Some(extra),
    );
}
