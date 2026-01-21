pub use body_plz::variants::Body;
pub use bytes::BytesMut;
use decompression_plz::state::DecodeState;
use tests_utils::TestMessage;

pub mod test_cases;

pub const TRANSFER_ENCODING: &str = "Transfer-Encoding";
pub const CONTENT_ENCODING: &str = "Content-Encoding";
