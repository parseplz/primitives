use header_plz::const_headers::{CONTENT_ENCODING, TRANSFER_ENCODING};

use super::*;
pub mod both_te_ce;
pub mod chunked;
pub mod extra;
pub mod multi_compression;
pub mod no_encodings;
pub mod partial;
pub mod single_compression;

fn encoding_state(
    header: &str,
) -> impl FnOnce(&DecodeState<TestMessage>) -> bool {
    if header == TRANSFER_ENCODING {
        |s: &DecodeState<TestMessage>| {
            matches!(s, DecodeState::TransferEncoding(_, _))
        }
    } else if header == CONTENT_ENCODING {
        |s: &DecodeState<TestMessage>| {
            matches!(s, DecodeState::ContentEncoding(_, _))
        }
    } else {
        panic!("Unknown header");
    }
}
