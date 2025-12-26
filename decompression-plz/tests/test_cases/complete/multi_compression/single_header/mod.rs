use header_plz::const_headers::{CONTENT_ENCODING, TRANSFER_ENCODING};

use super::*;
mod body_only;
mod with_extra;

fn build_test_message_all_encodings_single_header(
    header_name: &str,
    extra: Option<BytesMut>,
) -> TestMessage {
    let body = all_compressed_data();
    let headers = format!(
        "Host: example.com\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        {}: {}\r\n\
        Content-Length: {}\r\n\r\n",
        header_name,
        ALL_COMPRESSIONS,
        body.len()
    );

    TestMessage::new(headers.as_bytes().into(), Body::Raw(body), extra)
}

fn build_test_message_all_encodings_single_header_compressed_together(
    header_name: &str,
) -> TestMessage {
    let compressed = all_compressed_data();
    let (body, extra) = compressed.split_at(compressed.len() / 2);
    let headers = format!(
        "Host: example.com\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        {}: {}\r\n\
        Content-Length: {}\r\n\r\n",
        header_name,
        ALL_COMPRESSIONS,
        body.len()
    );

    TestMessage::new(
        headers.as_bytes().into(),
        Body::Raw(body.as_ref().into()),
        Some(extra.as_ref().into()),
    )
}
