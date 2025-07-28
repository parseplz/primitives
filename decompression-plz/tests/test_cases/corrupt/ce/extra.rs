use super::*;

#[test]
fn test_corrupt_ce_brotli_extra() {
    assert_corrupt_encoding(
        CONTENT_ENCODING,
        &ContentEncoding::Brotli,
        Some(INPUT.into()),
    )
}

#[test]
fn test_corrupt_ce_compress_extra() {
    assert_corrupt_encoding(
        CONTENT_ENCODING,
        &ContentEncoding::Compress,
        Some(INPUT.into()),
    )
}

#[test]
fn test_corrupt_ce_deflate_extra() {
    assert_corrupt_encoding(
        CONTENT_ENCODING,
        &ContentEncoding::Deflate,
        Some(INPUT.into()),
    )
}

#[test]
fn test_corrupt_ce_gzip_extra() {
    assert_corrupt_encoding(
        CONTENT_ENCODING,
        &ContentEncoding::Gzip,
        Some(INPUT.into()),
    )
}

#[test]
fn test_corrupt_ce_zstd_extra() {
    assert_corrupt_encoding(
        CONTENT_ENCODING,
        &ContentEncoding::Zstd,
        Some(INPUT.into()),
    )
}
