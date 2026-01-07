use body_plz::variants::Body;
use bytes::BytesMut;
use header_plz::const_headers::CONTENT_LENGTH;

use crate::decompress_trait::DecompressTrait;

pub fn add_body_and_update_cl<T>(message: &mut T, body: BytesMut)
where
    T: DecompressTrait,
{
    if body.is_empty() {
        return;
    }

    update_content_length(message, body.len());
    message.set_body(Body::Raw(body));
}

pub fn update_content_length<T>(message: &mut T, len: usize)
where
    T: DecompressTrait,
{
    let len_string = len.to_string();
    match message.has_header_key(CONTENT_LENGTH) {
        Some(pos) => message.update_header_value_on_position(pos, &len_string),
        None => message.insert_header(CONTENT_LENGTH, len_string.as_str()),
    }
}
