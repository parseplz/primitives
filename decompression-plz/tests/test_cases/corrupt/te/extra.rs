use super::*;

#[test]
fn test_corrupt_te_brotli_extra() {
    assert_corrupt_encoding(
        TRANSFER_ENCODING,
        &ContentEncoding::Brotli,
        Some(INPUT.into()),
    )
}

#[test]
fn test_corrupt_te_compress_extra() {
    assert_corrupt_encoding(
        TRANSFER_ENCODING,
        &ContentEncoding::Compress,
        Some(INPUT.into()),
    )
}

#[test]
fn test_corrupt_te_deflate_extra() {
    assert_corrupt_encoding(
        TRANSFER_ENCODING,
        &ContentEncoding::Deflate,
        Some(INPUT.into()),
    )
}

#[test]
fn test_corrupt_te_gzip_extra() {
    assert_corrupt_encoding(
        TRANSFER_ENCODING,
        &ContentEncoding::Gzip,
        Some(INPUT.into()),
    )
}

#[test]
fn test_corrupt_te_zstd_extra() {
    assert_corrupt_encoding(
        TRANSFER_ENCODING,
        &ContentEncoding::Zstd,
        Some(INPUT.into()),
    )
}
