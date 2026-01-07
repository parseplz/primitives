use rstest::rstest;

use super::*;

#[rstest]
fn assert_decode_state_single(
    #[values(TRANSFER_ENCODING, CONTENT_ENCODING)] encoding_type: &str,
    #[values(
        ContentEncoding::Brotli,
        ContentEncoding::Compress,
        ContentEncoding::Deflate,
        ContentEncoding::Gzip,
        ContentEncoding::Identity,
        ContentEncoding::Zstd
    )]
    encoding: ContentEncoding,
    #[values(true, false)] use_one: bool,
) {
    if use_one {
        assert_case_single_compression_one(encoding_type, encoding, None);
    } else {
        assert_case_single_compression_two(encoding_type, encoding);
    }
}
