use super::*;

#[test]
fn test_corrupt_ce_brotli() {
    assert_partial_encoding(CONTENT_ENCODING, &ContentEncoding::Brotli, None)
}

#[test]
fn test_corrupt_ce_compress() {
    assert_partial_encoding(CONTENT_ENCODING, &ContentEncoding::Compress, None)
}

#[test]
fn test_corrupt_ce_deflate() {
    assert_partial_encoding(CONTENT_ENCODING, &ContentEncoding::Deflate, None)
}

#[test]
fn test_corrupt_ce_gzip() {
    assert_partial_encoding(CONTENT_ENCODING, &ContentEncoding::Gzip, None)
}

#[test]
fn test_corrupt_ce_zstd() {
    assert_partial_encoding(CONTENT_ENCODING, &ContentEncoding::Zstd, None)
}
