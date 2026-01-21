use header_plz::{OneHeader, message_head::header_map::HMap};

use super::*;
mod body_only;
mod with_extra;

pub fn build_test_message_all_encodings_single_header<T>(
    header_name: &str,
    extra: Option<BytesMut>,
) -> TestMessage<T>
where
    T: From<OneHeader>,
    HMap<T>: From<HMap<header_plz::OneHeader>>,
{
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

    TestMessage::<T>::new(headers.as_bytes().into(), Body::Raw(body), extra)
}

fn build_test_message_all_encodings_single_header_compressed_together(
    header_name: &str,
) -> TestMessage<OneHeader> {
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
