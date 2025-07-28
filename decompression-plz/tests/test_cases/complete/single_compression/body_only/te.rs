use super::*;

// Transfer-Encoding
#[test]
fn assert_decode_state_single_te_brotli() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Brotli,
        None,
    );
}

#[test]
fn assert_decode_state_single_te_compress() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Compress,
        None,
    );
}

#[test]
fn assert_decode_state_single_te_deflate() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Deflate,
        None,
    );
}

#[test]
fn assert_decode_state_single_te_gzip() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Gzip,
        None,
    );
}

#[test]
fn assert_decode_state_single_te_identity() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Identity,
        None,
    );
}

#[test]
fn assert_decode_state_single_te_zstd() {
    assert_case_single_compression(
        TRANSFER_ENCODING,
        ContentEncoding::Zstd,
        None,
    );
}
