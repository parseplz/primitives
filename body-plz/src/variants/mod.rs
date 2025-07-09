use bytes::BytesMut;
use chunked::ChunkType;
use tracing::error;
pub mod chunked;

// Enum to represent Body
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq))]
pub enum Body {
    Chunked(Vec<ChunkType>),
    Raw(BytesMut),
}

impl Body {
    pub fn push_chunk(&mut self, body: ChunkType) {
        if let &mut Body::Chunked(ref mut chunks) = self {
            chunks.push(body);
        }
    }

    pub fn into_bytes(self) -> Option<BytesMut> {
        match self {
            Body::Raw(data) => Some(data),
            _ => {
                error!("Not Raw Body");
                None
            }
        }
    }

    pub fn into_chunks(self) -> Vec<ChunkType> {
        match self {
            Body::Chunked(chunks) => chunks,
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variants_body_push_chunk() {
        let mut body = Body::Chunked(Vec::new());
        let buf = BytesMut::from("data\r\n");
        body.push_chunk(ChunkType::Chunk(buf.clone()));
        assert_eq!(body, Body::Chunked(vec![ChunkType::Chunk(buf.clone())]));

        body.push_chunk(ChunkType::Size(buf.clone()));
        assert_eq!(
            body,
            Body::Chunked(vec![
                ChunkType::Chunk(buf.clone()),
                ChunkType::Size(buf.clone())
            ])
        );
    }

    #[test]
    fn test_variants_body_into_bytes_raw() {
        let buf = BytesMut::from("data\r\n");
        let body = Body::Raw(buf.clone());
        assert_eq!(body.into_bytes(), Some(buf));
    }

    #[test]
    fn test_variants_body_into_bytes_chunk() {
        let body = Body::Chunked(Vec::new());
        assert_eq!(body.into_bytes(), None);
    }

    #[test]
    fn test_variants_body_into_chunks_raw() {
        let body = Body::Raw(BytesMut::from("data\r\n"));
        assert_eq!(body.into_chunks(), Vec::new());
    }

    #[test]
    fn test_variants_body_into_chunks_chunk() {
        let buf = BytesMut::from("data\r\n");
        let body = Body::Chunked(vec![ChunkType::Chunk(buf.clone())]);
        assert_eq!(body.into_chunks(), vec![ChunkType::Chunk(buf.clone())]);
    }
}
