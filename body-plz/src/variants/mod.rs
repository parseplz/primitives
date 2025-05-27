use bytes::BytesMut;
use chunked::ChunkedBody;
use tracing::error;
pub mod chunked;

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
