use super::*;

// CE only
#[test]
fn assert_decode_state_single_ce_brotli() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Brotli,
        None,
    );
}

#[test]
fn assert_decode_state_single_ce_compress() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Compress,
        None,
    );
}

#[test]
fn assert_decode_state_single_ce_deflate() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Deflate,
        None,
    );
}

#[test]
fn assert_decode_state_single_ce_gzip() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Gzip,
        None,
    );
}

#[test]
fn assert_decode_state_single_ce_identity() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Identity,
        None,
    );
}

#[test]
fn assert_decode_state_single_ce_zstd() {
    assert_case_single_compression(
        CONTENT_ENCODING,
        ContentEncoding::Zstd,
        None,
    );
}
