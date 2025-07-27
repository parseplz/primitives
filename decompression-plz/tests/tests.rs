#![allow(warnings)]
pub use body_plz::variants::Body;
pub use bytes::BytesMut;
use decompression_plz::DecompressTrait;
use decompression_plz::state::DecodeState;
use header_plz::{HeaderMap, body_headers::BodyHeader};
use tests_utils::TestMessage;

pub mod test_cases;

enum EncodingKind {
    Te,
    Ce,
}

struct Case {
    header_name: &'static str,
    expected_state: fn(&DecodeState<TestMessage>) -> bool,
}
