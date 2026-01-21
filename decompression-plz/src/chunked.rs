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
use header_plz::{Header, OneHeader};

use crate::{DecompressTrait, decode_struct::DecodeStruct};

pub trait ChunkedConverter<T> {
    fn convert_chunked(&mut self);
}

impl<'a, T> ChunkedConverter<OneHeader> for DecodeStruct<'a, T>
where
    T: DecompressTrait<HmapType = OneHeader> + std::fmt::Debug,
{
    fn convert_chunked(&mut self) {
        chunked_to_raw(self.message, self.buf);
        self.body = self.message.get_body().into_bytes().unwrap();
    }
}

impl<'a, T> ChunkedConverter<Header> for DecodeStruct<'a, T>
where
    T: DecompressTrait<HmapType = Header> + std::fmt::Debug,
{
    #[inline(always)]
    fn convert_chunked(&mut self) {}
}

pub fn chunked_to_raw<T>(message: &mut T, buf: &mut BytesMut)
where
    T: DecompressTrait<HmapType = OneHeader>,
{
    let body = message.get_body().into_chunks();
    buf.reserve(total_chunk_size(&body));
    body.into_iter().for_each(|chunk| {
        match chunk {
            // 1. Combine ChunkType::Chunk into one body.
            ChunkType::Chunk(data) => {
                buf.extend_from_slice(&data[..data.len() - 2])
            }
            // 2. If trailer is present,
            ChunkType::Trailers(trailer) => {
                // 2.a. Remove trailer header
                message.remove_header_on_key("trailer");
                // 2.b. Add trailer to header_map
                message.extend(trailer);
            }
            _ => (),
        }
    });
    message.set_body(Body::Raw(buf.split()));
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
