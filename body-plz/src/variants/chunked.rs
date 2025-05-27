use bytes::BytesMut;
use header_plz::header_map::HeaderMap;

// Enum to represent different types of Chunked Body
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq))]
pub enum ChunkedBody {
    Size(BytesMut),
    Chunk(BytesMut),
    LastChunk(BytesMut),
    Trailers(HeaderMap),
    EndCRLF(BytesMut),
    Extra(BytesMut),
}

impl ChunkedBody {
    fn len(&self) -> usize {
        match self {
            ChunkedBody::Size(buf) | ChunkedBody::Chunk(buf) | ChunkedBody::Extra(buf) => buf.len(),
            ChunkedBody::LastChunk(_) => 3,
            ChunkedBody::EndCRLF(_) => 2,
            ChunkedBody::Trailers(header_map) => header_map.len(),
        }
    }
}

pub fn total_chunk_size(chunks: &[ChunkedBody]) -> usize {
    chunks.iter().fold(0, |acc, chunk| {
        if let ChunkedBody::Chunk(data) = chunk {
            acc + data.len() - 2 // CRLF
        } else {
            acc
        }
    })
}

pub fn total_chunk_size_unchecked(chunks: &[ChunkedBody]) -> usize {
    chunks.iter().fold(0, |acc, chunk| acc + chunk.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_chunk_size() {
        let buf = BytesMut::from("data\r\n");
        let mut vec_body = Vec::with_capacity(20);
        for _ in 0..10 {
            vec_body.push(ChunkedBody::Size(buf.clone()));
            vec_body.push(ChunkedBody::Chunk(buf.clone()));
        }
        assert_eq!(total_chunk_size(&vec_body), 40);
    }

    #[test]
    fn test_total_chunk_size_unchecked() {
        let header_buf = BytesMut::from("a: b\r\nc: d\r\n"); // 12
        let header_map = HeaderMap::new(header_buf);
        let body_vec = vec![
            ChunkedBody::Size(BytesMut::from("7; hola amigo\r\n")), // 15
            ChunkedBody::Chunk(BytesMut::from("Mozilla\r\n")),      // 9
            ChunkedBody::Size(BytesMut::from("9\r\n")),             // 3
            ChunkedBody::Chunk(BytesMut::from("Developer\r\n")),    // 11
            ChunkedBody::Size(BytesMut::from("7\r\n")),             // 3
            ChunkedBody::Chunk(BytesMut::from("Network\r\n")),      // 9
            ChunkedBody::LastChunk(BytesMut::from("0\r\n")),        // 3
            ChunkedBody::Trailers(header_map),                      // 12
            ChunkedBody::EndCRLF(BytesMut::from("\r\n")),           // 2
        ];
        let size = total_chunk_size_unchecked(&body_vec);
        assert_eq!(size, 67);
    }
}
