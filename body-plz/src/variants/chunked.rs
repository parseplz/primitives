use bytes::BytesMut;
use header_plz::HeaderMap;

// Enum to represent different types of Chunked Body
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq))]
pub enum ChunkType {
    Size(BytesMut),
    Chunk(BytesMut),
    LastChunk(BytesMut),
    Trailers(HeaderMap),
    EndCRLF(BytesMut),
    Extra(BytesMut),
}

impl ChunkType {
    fn len(&self) -> usize {
        match self {
            ChunkType::Size(buf) | ChunkType::Chunk(buf) | ChunkType::Extra(buf) => buf.len(),
            ChunkType::LastChunk(_) => 3,
            ChunkType::EndCRLF(_) => 2,
            ChunkType::Trailers(header_map) => header_map.len(),
        }
    }

    pub fn into_bytes(self) -> BytesMut {
        match self {
            ChunkType::Size(buf)
            | ChunkType::Chunk(buf)
            | ChunkType::Extra(buf)
            | ChunkType::LastChunk(buf)
            | ChunkType::EndCRLF(buf) => buf,
            ChunkType::Trailers(header_map) => header_map.into_bytes(),
        }
    }
}

pub fn total_chunk_size(chunks: &[ChunkType]) -> usize {
    chunks.iter().fold(0, |acc, chunk| {
        if let ChunkType::Chunk(data) = chunk {
            acc + data.len() - 2 // CRLF
        } else {
            acc
        }
    })
}

#[cfg(test)]
mod tests {
    use buffer_plz::Cursor;

    use crate::reader::chunked_reader::ChunkReaderState;

    use super::*;

    fn parse_chunked_body(input: &str, with_trailers: bool) -> Vec<ChunkType> {
        let mut buf = BytesMut::from(input);
        let mut cbuf = Cursor::new(&mut buf);
        let mut state = ChunkReaderState::ReadSize;
        let mut chunk_vec = vec![];
        loop {
            match state.next(&mut cbuf) {
                Some(chunk_to_add) => {
                    chunk_vec.push(chunk_to_add);
                    match state {
                        ChunkReaderState::LastChunk => {
                            state = if with_trailers {
                                ChunkReaderState::ReadTrailers
                            } else {
                                ChunkReaderState::EndCRLF
                            };
                            continue;
                        }
                        ChunkReaderState::End => break,
                        ChunkReaderState::Failed(e) => panic!("{}", e),
                        _ => continue,
                    }
                }
                None => continue,
            }
        }
        chunk_vec
    }

    #[test]
    fn test_chunk_type_len() {
        let buf = BytesMut::from("data\r\n");
        let size_chunk = ChunkType::Size(buf.clone());
        let len = size_chunk.len();
        assert_eq!(len, 6);

        let chunk = ChunkType::Chunk(buf.clone());
        assert_eq!(chunk.len(), 6);

        let extra = ChunkType::Extra(buf.clone());
        assert_eq!(extra.len(), 6);

        let last_chunk = ChunkType::LastChunk(buf.clone());
        assert_eq!(last_chunk.len(), 3);

        let end_crlf = ChunkType::EndCRLF(buf.clone());
        assert_eq!(end_crlf.len(), 2);

        let raw_headers = "content-type: application/json\r\n\
                           content-encoding: gzip\r\n\r\n";

        let header_map = HeaderMap::from(BytesMut::from(raw_headers));
        let trailers = ChunkType::Trailers(header_map);
        assert_eq!(trailers.len(), 58);
    }

    #[test]
    fn test_total_chunk_size() {
        let buf = BytesMut::from("data\r\n");
        let mut vec_body = Vec::with_capacity(20);
        for _ in 0..10 {
            vec_body.push(ChunkType::Size(buf.clone()));
            vec_body.push(ChunkType::Chunk(buf.clone()));
        }
        assert_eq!(total_chunk_size(&vec_body), 40);
    }

    #[test]
    fn test_chunk_into_data() {
        let data = "7; hola amigo\r\n\
                   Mozilla\r\n\
                   9\r\n\
                   Developer\r\n\
                   7\r\n\
                   Network\r\n\
                   0\r\n\
                   a: b\r\n\
                   c: d\r\n\
                   \r\n";
        let mut chunk_vec = parse_chunked_body(data, true);
        assert_eq!(
            chunk_vec.pop().unwrap().into_bytes(),
            "a: b\r\nc: d\r\n\r\n"
        );
        assert_eq!(chunk_vec.pop().unwrap().into_bytes(), "0\r\n");
        assert_eq!(chunk_vec.pop().unwrap().into_bytes(), "Network\r\n");
        assert_eq!(chunk_vec.pop().unwrap().into_bytes(), "7\r\n");
        assert_eq!(chunk_vec.pop().unwrap().into_bytes(), "Developer\r\n");
        assert_eq!(chunk_vec.pop().unwrap().into_bytes(), "9\r\n");
        assert_eq!(chunk_vec.pop().unwrap().into_bytes(), "Mozilla\r\n");
        assert_eq!(chunk_vec.pop().unwrap().into_bytes(), "7; hola amigo\r\n");
    }
}
