/* Description:
 *      Convert chunked body to content length.
 *
 * Steps:
 *      1. Combine ChunkType::Chunk into one body.
 *      2. If trailer is present,
 *          a. remove trailer header
 *          b. add trailer to header_map.
 */

use body_plz::variants::{
    Body,
    chunked::{ChunkType, total_chunk_size},
};
use bytes::BytesMut;
use header_plz::const_headers::TRAILER;

use crate::DecompressTrait;

pub fn chunked_to_raw<T>(message: &mut T, buf: &mut BytesMut)
where
    T: DecompressTrait,
{
    let body = message.get_body().into_chunks();
    buf.reserve(total_chunk_size(&body));
    let mut new_body = buf.split();
    body.into_iter().for_each(|chunk| {
        match chunk {
            // 1. Combine ChunkType::Chunk into one body.
            ChunkType::Chunk(data) => {
                new_body.extend_from_slice(&data[..data.len() - 2])
            }
            // 2. If trailer is present,
            ChunkType::Trailers(trailer) => {
                // 2.a. Remove trailer header
                message.remove_header_on_key(TRAILER);
                // 2.b. Add trailer to header_map
                let trailer_header = trailer.into_header_vec();
                message.add_multi_headers(trailer_header);
            }
            _ => (),
        }
    });
    message.set_body(Body::Raw(new_body));
}

// Partial chunked body
pub fn partial_chunked_to_raw(vec_body: Vec<ChunkType>) -> Option<BytesMut> {
    let mut iter = vec_body.into_iter().map(|c| c.into_bytes());
    let mut body = iter.next()?;

    for chunk in iter {
        body.unsplit(chunk);
    }

    Some(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate tests_utils;
    use tests_utils::TestMessage;

    #[test]
    fn test_chunked_to_raw() {
        let a: Option<TestMessage>;
    }
}
