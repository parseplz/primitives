use super::*;

#[test]
fn assert_decode_state_single_ce_brotli_extra_compressed_separate() {
    let content_encoding = ContentEncoding::Brotli;
    let extra = single_compression(&content_encoding);
    assert_case_single_compression(
        CONTENT_ENCODING,
        content_encoding,
        Some(extra),
    );
}

#[test]
fn assert_decode_state_single_ce_compress_extra_compressed_separate() {
    let content_encoding = ContentEncoding::Compress;
    let extra = single_compression(&content_encoding);
    assert_case_single_compression(
        CONTENT_ENCODING,
        content_encoding,
        Some(extra),
    );
}

#[test]
fn assert_decode_state_single_ce_deflate_extra_compressed_separate() {
    let content_encoding = ContentEncoding::Deflate;
    let extra = single_compression(&content_encoding);
    assert_case_single_compression(
        CONTENT_ENCODING,
        content_encoding,
        Some(extra),
    );
}

#[test]
fn assert_decode_state_single_ce_gzip_extra_compressed_separate() {
    let content_encoding = ContentEncoding::Gzip;
    let extra = single_compression(&content_encoding);
    assert_case_single_compression(
        CONTENT_ENCODING,
        content_encoding,
        Some(extra),
    );
}

#[test]
fn assert_decode_state_single_ce_identity_extra_compressed_separate() {
    let content_encoding = ContentEncoding::Identity;
    let extra = single_compression(&content_encoding);
    assert_case_single_compression(
        CONTENT_ENCODING,
        content_encoding,
        Some(extra),
    );
}

#[test]
fn assert_decode_state_single_ce_zstd_extra_compressed_separate() {
    let content_encoding = ContentEncoding::Zstd;
    let extra = single_compression(&content_encoding);
    assert_case_single_compression(
        CONTENT_ENCODING,
        content_encoding,
        Some(extra),
    );
}
