use super::*;
use body_plz::variants::chunked::ChunkType;
use decompression_plz::chunked::{chunked_to_raw, partial_chunked_to_raw};
use tests_utils::{INPUT, all_compressed_data};

const CHUNKED_HEADER: &str = "Host: example.com\r\n\
                       Content-Type: text/html; charset=utf-8\r\n\
                       Transfer-Encoding: chunked\r\n\r\n";

// converter
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
        ChunkType::Size("7\r\n".into()),
        ChunkType::Chunk("Mozilla\r\n".into()),
        ChunkType::Size("9\r\n".into()),
        ChunkType::Chunk("Developer\r\n".into()),
        ChunkType::Size("7\r\n".into()),
        ChunkType::Chunk("Network\r\n".into()),
        ChunkType::LastChunk("0\r\n".into()),
        ChunkType::EndCRLF("\r\n".into()),
    ];
    Body::Chunked(chunk_vec)
}

#[test]
fn test_chunked_to_raw() {
    let body = build_chunked_body_large();
    let mut buf = BytesMut::new();

    let mut tm = TestMessage::new(CHUNKED_HEADER.into(), body, None);
    chunked_to_raw(&mut tm, &mut buf);
    let verify = "Host: example.com\r\n\
                  Content-Type: text/html; charset=utf-8\r\n\
                  Transfer-Encoding: chunked\r\n\r\n\
                  MozillaDeveloperNetwork";
    assert_eq!(tm.into_bytes(), verify);
}

#[test]
fn test_chunked_to_raw_with_trailer() {
    let mut body = build_chunked_body_large();
    let trailer_headers = "Header: Val\r\n\
                           Another: Val\r\n\r\n";
    let trailer_chunk =
        ChunkType::Trailers(HeaderMap::from(BytesMut::from(trailer_headers)));
    body.push_chunk(trailer_chunk);
    let mut tm = TestMessage::new(CHUNKED_HEADER.into(), body, None);
    let mut buf = BytesMut::new();
    chunked_to_raw(&mut tm, &mut buf);
    let verify = "Host: example.com\r\n\
                  Content-Type: text/html; charset=utf-8\r\n\
                  Transfer-Encoding: chunked\r\n\
                  Header: Val\r\n\
                  Another: Val\r\n\r\n\
                  MozillaDeveloperNetwork";
    assert_eq!(tm.into_bytes(), verify);
}

#[test]
fn test_partial_chunked_to_raw() {
    let chunks = build_chunked_body_large().into_chunks();
    let body = partial_chunked_to_raw(chunks);
    assert!(body.is_some());
    let verify = "7\r\n\
                  Mozilla\r\n\
                  9\r\n\
                  Developer\r\n\
                  7\r\n\
                  Network\r\n\
                  0\r\n\r\n";
    assert_eq!(body.unwrap(), verify);
}

// state
#[test]
fn test_chunked_body_large() {
    let mut tm = TestMessage::new(
        CHUNKED_HEADER.into(),
        build_chunked_body_large(),
        None,
    );
    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::TransferEncoding(..)));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(_)));
    state = state.try_next().unwrap();
    assert!(state.is_ended());
    let verify = "Host: example.com\r\n\
                  Content-Type: text/html; charset=utf-8\r\n\
                  Content-Length: 23\r\n\r\n\
                  MozillaDeveloperNetwork";
    let result = tm.into_bytes();
    assert_eq!(result, verify);
}

fn build_all_compressed_chunk_body() -> Body {
    let body = all_compressed_data(); // len 53
    let mut chunk_vec = vec![
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[0..10].into()),
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[10..20].into()),
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[20..30].into()),
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[30..40].into()),
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[40..50].into()),
        ChunkType::Size("3\r\n".into()),
        ChunkType::Chunk(body[50..].into()),
        ChunkType::EndCRLF("\r\n".into()),
    ];

    for chunk in chunk_vec.iter_mut() {
        if let ChunkType::Chunk(chunk) = chunk {
            chunk.extend_from_slice("\r\n".as_bytes());
        }
    }
    Body::Chunked(chunk_vec)
}

#[track_caller]
fn assert_chunked_encoding(
    headers: &str,
    with_ce: bool,
    extra: Option<BytesMut>,
) {
    let mut buf = BytesMut::new();

    let verify = if extra.is_none() {
        "Host: example.com\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Content-Length: 11\r\n\r\n\
         hello world"
    } else {
        "Host: example.com\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Content-Length: 22\r\n\r\n\
         hello worldhello world"
    };

    let mut tm = TestMessage::new(
        headers.into(),
        build_all_compressed_chunk_body(),
        extra,
    );

    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::TransferEncoding(..)));

    if with_ce {
        state = state.try_next().unwrap();
        assert!(matches!(state, DecodeState::ContentEncoding(..)));
    }

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(..)));

    state = state.try_next().unwrap();
    assert!(state.is_ended());

    let result = tm.into_bytes();
    assert_eq!(result, verify);
}

#[test]
fn test_chunked_with_compression() {
    let headers = "Host: example.com\r\n\
                   Content-Type: text/html; charset=utf-8\r\n\
                   Transfer-Encoding: br, deflate, identity, gzip, zstd, chunked\r\n\
                   \r\n";

    assert_chunked_encoding(headers, false, None);
}

fn build_chunked_multi_header_body(encoding: &str) -> String {
    format!(
        "Host: example.com\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        {encoding}: br\r\n\
        {encoding}: deflate\r\n\
        {encoding}: identity\r\n\
        {encoding}: gzip\r\n\
        {encoding}: zstd\r\n\
        Transfer-Encoding: chunked\r\n\r\n"
    )
}

#[test]
fn test_chunked_with_compression_multi_header() {
    assert_chunked_encoding(
        &build_chunked_multi_header_body(TRANSFER_ENCODING),
        false,
        None,
    );
}

#[test]
fn test_chunked_with_compression_extra_raw() {
    let headers = "Host: example.com\r\n\
                   Content-Type: text/html; charset=utf-8\r\n\
                   Transfer-Encoding: br, deflate, identity, gzip, zstd, chunked\r\n\
                   \r\n";

    assert_chunked_encoding(headers, false, Some(INPUT.into()));
}

#[test]
fn test_chunked_with_compression_multi_header_extra_raw() {
    assert_chunked_encoding(
        &build_chunked_multi_header_body(TRANSFER_ENCODING),
        false,
        Some(INPUT.into()),
    );
}

#[test]
fn test_chunked_with_compress_extra_compressed_together() {
    let body = all_compressed_data(); // len 53
    let mut chunk_vec = vec![
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[0..10].into()),
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[10..20].into()),
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[20..30].into()),
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[30..40].into()),
        ChunkType::Size("10\r\n".into()),
    ];

    for chunk in chunk_vec.iter_mut() {
        if let ChunkType::Chunk(chunk) = chunk {
            chunk.extend_from_slice("\r\n".as_bytes());
        }
    }
    let chunk_body = Body::Chunked(chunk_vec);
    let extra = BytesMut::from(&body[40..]);

    let headers = "Host: example.com\r\n\
                   Content-Type: text/html; charset=utf-8\r\n\
                   Transfer-Encoding: br, deflate, identity, gzip, zstd, chunked\r\n\
                   \r\n";

    let mut tm = TestMessage::new(headers.into(), chunk_body, Some(extra));

    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::TransferEncoding(..)));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(..)));

    state = state.try_next().unwrap();
    assert!(state.is_ended());
    assert_eq!(tm.into_bytes(), VERIFY_SINGLE_HEADER_BODY_ONLY);
}

#[test]
fn test_chunked_with_compress_extra_compressed_together_multi_header() {
    let body = all_compressed_data(); // len 53
    let mut chunk_vec = vec![
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[0..10].into()),
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[10..20].into()),
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[20..30].into()),
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[30..40].into()),
        ChunkType::Size("10\r\n".into()),
    ];

    for chunk in chunk_vec.iter_mut() {
        if let ChunkType::Chunk(chunk) = chunk {
            chunk.extend_from_slice("\r\n".as_bytes());
        }
    }
    let chunk_body = Body::Chunked(chunk_vec);
    let extra = BytesMut::from(&body[40..]);

    let headers = build_chunked_multi_header_body(TRANSFER_ENCODING);

    let mut tm = TestMessage::new(
        BytesMut::from(&headers[..]),
        chunk_body,
        Some(extra),
    );

    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::TransferEncoding(..)));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(..)));

    state = state.try_next().unwrap();
    assert!(state.is_ended());
    assert_eq!(tm.into_bytes(), VERIFY_SINGLE_HEADER_BODY_ONLY);
}

#[test]
fn test_chunked_with_compress_extra_compressed_separate() {
    let body = build_all_compressed_chunk_body();
    let extra = all_compressed_data();

    let headers = "Host: example.com\r\n\
                   Content-Type: text/html; charset=utf-8\r\n\
                   Transfer-Encoding: br, deflate, identity, gzip, zstd, chunked\r\n\
                   \r\n";

    let mut tm = TestMessage::new(headers.into(), body, Some(extra));

    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::TransferEncoding(..)));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(..)));

    state = state.try_next().unwrap();
    assert!(state.is_ended());
    assert_eq!(tm.into_bytes(), VERIFY_SINGLE_HEADER_BODY_AND_EXTRA);
}

#[test]
fn test_chunked_with_compress_extra_compressed_separate_multi_header() {
    let body = build_all_compressed_chunk_body();
    let extra = all_compressed_data();

    let headers = build_chunked_multi_header_body(TRANSFER_ENCODING);

    let mut tm =
        TestMessage::new(BytesMut::from(&headers[..]), body, Some(extra));

    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::TransferEncoding(..)));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(..)));

    state = state.try_next().unwrap();
    assert!(state.is_ended());
    assert_eq!(tm.into_bytes(), VERIFY_SINGLE_HEADER_BODY_AND_EXTRA);
}

/// Ce
#[test]
fn test_chunked_with_ce_compression() {
    let headers = "Host: example.com\r\n\
                   Content-Type: text/html; charset=utf-8\r\n\
                   Transfer-Encoding: chunked\r\n\
                   Content-Encoding: br, deflate, identity, gzip, zstd\r\n\r\n";

    assert_chunked_encoding(headers, true, None);
}

#[test]
fn test_chunked_with_ce_compression_extra_raw() {
    let headers = "Host: example.com\r\n\
                   Content-Type: text/html; charset=utf-8\r\n\
                   Transfer-Encoding: chunked\r\n\
                   Content-Encoding: br, deflate, identity, gzip, zstd\r\n\r\n";

    assert_chunked_encoding(headers, true, Some(INPUT.into()));
}

#[test]
fn test_chunked_with_ce_compress_extra_compressed_together() {
    let body = all_compressed_data(); // len 53
    let mut chunk_vec = vec![
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[0..10].into()),
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[10..20].into()),
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[20..30].into()),
        ChunkType::Size("10\r\n".into()),
        ChunkType::Chunk(body[30..40].into()),
        ChunkType::Size("10\r\n".into()),
    ];

    for chunk in chunk_vec.iter_mut() {
        if let ChunkType::Chunk(chunk) = chunk {
            chunk.extend_from_slice("\r\n".as_bytes());
        }
    }
    let chunk_body = Body::Chunked(chunk_vec);
    let extra = BytesMut::from(&body[40..]);

    let headers = "Host: example.com\r\n\
                   Content-Type: text/html; charset=utf-8\r\n\
                   Content-Encoding: br, deflate, identity, gzip, zstd\r\n\
                   Transfer-Encoding: chunked\r\n\
                   \r\n";

    let mut tm = TestMessage::new(headers.into(), chunk_body, Some(extra));

    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::TransferEncoding(..)));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::ContentEncoding(..)));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(..)));

    state = state.try_next().unwrap();
    assert!(state.is_ended());
    assert_eq!(tm.into_bytes(), VERIFY_SINGLE_HEADER_BODY_ONLY);
}

#[test]
fn test_chunked_with_ce_compress_extra_compressed_separate() {
    let body = build_all_compressed_chunk_body();
    let extra = all_compressed_data();

    let headers = "Host: example.com\r\n\
                   Content-Type: text/html; charset=utf-8\r\n\
                   Content-Encoding: br, deflate, identity, gzip, zstd\r\n\
                   Transfer-Encoding: chunked\r\n\
                   \r\n";

    let mut tm = TestMessage::new(headers.into(), body, Some(extra));

    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::TransferEncoding(..)));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::ContentEncoding(..)));

    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(..)));

    state = state.try_next().unwrap();
    assert!(state.is_ended());
    assert_eq!(tm.into_bytes(), VERIFY_SINGLE_HEADER_BODY_AND_EXTRA);
}
