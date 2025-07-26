use super::*;
use body_plz::{
    reader::chunked_reader::ChunkReaderState, variants::chunked::ChunkType,
};

fn build_chunked_body_large() -> Body {
    /*
    7\r\n\
    Mozilla\r\n\
    9\r\n\
    Developer\r\n\
    7\r\n\
    Network\r\n\
    0\r\n\
    Header: Val\r\n\
    */

    let chunk_vec = vec![
        ChunkType::Size("7".into()),
        ChunkType::Chunk("Mozilla\r\n".into()),
        ChunkType::Size("9".into()),
        ChunkType::Chunk("Developer\r\n".into()),
        ChunkType::Size("7".into()),
        ChunkType::Chunk("Network\r\n".into()),
        ChunkType::LastChunk("0\r\n".into()),
        ChunkType::EndCRLF("\r\n".into()),
    ];
    Body::Chunked(chunk_vec)
}

#[test]
fn test_chunked_body_large() {
    let headers = "Host: example.com\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Transfer-Encoding: chunked\r\n\r\n";
    let mut tm =
        TestMessage::build(headers.into(), build_chunked_body_large(), None);

    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::TransferEncoding(..)));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(_)));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::End));

    let verify = "Host: example.com\r\n\
                  Content-Type: text/html; charset=utf-8\r\n\
                  Content-Length: 23\r\n\r\n\
                  MozillaDeveloperNetwork";
    let result = tm.into_bytes();
    assert_eq!(result, verify);
}
