use super::*;
pub mod both_te_ce;
pub mod chunked;
pub mod complete;
pub mod corrupt;
pub mod no_encodings;
pub mod partial;

fn encoding_state<T>(
    header: &str,
) -> impl FnOnce(&DecodeState<TestMessage<T>>) -> bool {
    if header == TRANSFER_ENCODING {
        |s: &DecodeState<TestMessage<T>>| {
            matches!(s, DecodeState::TransferEncoding(_, _))
        }
    } else if header == CONTENT_ENCODING {
        |s: &DecodeState<TestMessage<T>>| {
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
