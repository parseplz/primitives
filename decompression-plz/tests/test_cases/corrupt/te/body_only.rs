use super::*;

#[test]
fn test_corrupt_te_brotli() {
    assert_corrupt_encoding(TRANSFER_ENCODING, &ContentEncoding::Brotli, None)
}

#[test]
fn test_corrupt_te_compress() {
    assert_corrupt_encoding(
        TRANSFER_ENCODING,
        &ContentEncoding::Compress,
        None,
    )
}

#[test]
fn test_corrupt_te_deflate() {
    assert_corrupt_encoding(TRANSFER_ENCODING, &ContentEncoding::Deflate, None)
}

#[test]
fn test_corrupt_te_gzip() {
    assert_corrupt_encoding(TRANSFER_ENCODING, &ContentEncoding::Gzip, None)
}

#[test]
fn test_corrupt_te_zstd() {
    assert_corrupt_encoding(TRANSFER_ENCODING, &ContentEncoding::Zstd, None)
}
