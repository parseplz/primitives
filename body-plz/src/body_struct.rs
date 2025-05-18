use bytes::BytesMut;
use header_plz::header_map::HeaderMap;
use tracing::error;

// Enum to represent Body
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq))]
pub enum Body {
    Chunked(Vec<ChunkedBody>),
    Raw(BytesMut),
}

impl Body {
    pub fn push_chunk(&mut self, body: ChunkedBody) {
        if let &mut Body::Chunked(ref mut chunks) = self {
            chunks.push(body);
        }
    }

    pub fn into_data(self) -> Option<BytesMut> {
        match self {
            Body::Raw(data) => Some(data),
            _ => {
                error!("Not Raw Body");
                None
            }
        }
    }

    pub fn into_chunks(self) -> Vec<ChunkedBody> {
        match self {
            Body::Chunked(chunks) => chunks,
            _ => Vec::new(),
        }
    }
}

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

pub fn total_chunk_size(chunks: &[ChunkedBody]) -> usize {
    chunks.iter().fold(0, |acc, chunk| {
        if let ChunkedBody::Chunk(data) = chunk {
            acc + data.len() - 2 // CRLF
        } else {
            acc
        }
    })
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
}
