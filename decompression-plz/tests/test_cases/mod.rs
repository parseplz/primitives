use header_plz::const_headers::{CONTENT_ENCODING, TRANSFER_ENCODING};

use super::*;
pub mod both_te_ce;
pub mod chunked;
pub mod complete;
pub mod corrupt;
pub mod no_encodings;
pub mod partial;

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

const VERIFY_SINGLE_HEADER_BODY_ONLY: &str = "Host: example.com\r\n\
                                              Content-Type: text/html; charset=utf-8\r\n\
                                              Content-Length: 11\r\n\r\n\
                                              hello world";

const VERIFY_SINGLE_HEADER_BODY_AND_EXTRA: &str = "Host: example.com\r\n\
                                                   Content-Type: text/html; charset=utf-8\r\n\
                                                   Content-Length: 22\r\n\r\n\
                                                   hello worldhello world";
