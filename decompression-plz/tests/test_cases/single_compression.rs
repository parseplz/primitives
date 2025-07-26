use header_plz::body_headers::content_encoding::ContentEncoding;
use tests_utils::single_compression;

use super::*;

fn run_case_single_compression(
    case: &Case,
    content_encoding: ContentEncoding,
) {
    let body: Vec<u8> = single_compression(&content_encoding);
    let headers = format!(
        "Host: example.com\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         {}: {}\r\n\
         Content-Length: {}\r\n\r\n",
        case.header_name,
        content_encoding.as_ref(),
        body.len()
    );

    let mut tm = TestMessage::build(
        headers.as_bytes().into(),
        Body::Raw(body.as_slice().into()),
        None,
    );
    let mut buf = BytesMut::new();
    let mut state = DecodeState::init(&mut tm, &mut buf);
    state = state.try_next().unwrap();
    assert!((case.expected_state)(&state));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::UpdateContentLength(_)));
    state = state.try_next().unwrap();
    assert!(matches!(state, DecodeState::End));

    let result = tm.into_bytes();
    let verify = "Host: example.com\r\n\
                  Content-Type: text/html; charset=utf-8\r\n\
                  Content-Length: 11\r\n\r\n\
                  hello world";
    assert_eq!(result, verify);
}

// Transfer-Encoding

#[test]
fn assert_decode_state_single_te_brotli() {
    let case = Case {
        header_name: "Transfer-Encoding",
        expected_state: |s| matches!(s, DecodeState::TransferEncoding(_, _)),
    };
    run_case_single_compression(&case, ContentEncoding::Brotli);
}

#[test]
fn assert_decode_state_single_te_compress() {
    let case = Case {
        header_name: "Transfer-Encoding",
        expected_state: |s| matches!(s, DecodeState::TransferEncoding(_, _)),
    };
    run_case_single_compression(&case, ContentEncoding::Compress);
}

#[test]
fn assert_decode_state_single_te_deflate() {
    let case = Case {
        header_name: "Transfer-Encoding",
        expected_state: |s| matches!(s, DecodeState::TransferEncoding(_, _)),
    };
    run_case_single_compression(&case, ContentEncoding::Deflate);
}

#[test]
fn assert_decode_state_single_te_gzip() {
    let case = Case {
        header_name: "Transfer-Encoding",
        expected_state: |s| matches!(s, DecodeState::TransferEncoding(_, _)),
    };
    run_case_single_compression(&case, ContentEncoding::Gzip);
}

#[test]
fn assert_decode_state_single_te_identity() {
    let case = Case {
        header_name: "Transfer-Encoding",
        expected_state: |s| matches!(s, DecodeState::TransferEncoding(_, _)),
    };
    run_case_single_compression(&case, ContentEncoding::Identity);
}

#[test]
fn assert_decode_state_single_te_zstd() {
    let case = Case {
        header_name: "Transfer-Encoding",
        expected_state: |s| matches!(s, DecodeState::TransferEncoding(_, _)),
    };
    run_case_single_compression(&case, ContentEncoding::Zstd);
}

// CE only
#[test]
fn assert_decode_state_single_ce_brotli() {
    let case = Case {
        header_name: "Content-Encoding",
        expected_state: |s| matches!(s, DecodeState::ContentEncoding(_, _)),
    };
    run_case_single_compression(&case, ContentEncoding::Brotli);
}

#[test]
fn assert_decode_state_single_ce_compress() {
    let case = Case {
        header_name: "Content-Encoding",
        expected_state: |s| matches!(s, DecodeState::ContentEncoding(_, _)),
    };
    run_case_single_compression(&case, ContentEncoding::Compress);
}

#[test]
fn assert_decode_state_single_ce_deflate() {
    let case = Case {
        header_name: "Content-Encoding",
        expected_state: |s| matches!(s, DecodeState::ContentEncoding(_, _)),
    };
    run_case_single_compression(&case, ContentEncoding::Deflate);
}

#[test]
fn assert_decode_state_single_ce_gzip() {
    let case = Case {
        header_name: "Content-Encoding",
        expected_state: |s| matches!(s, DecodeState::ContentEncoding(_, _)),
    };
    run_case_single_compression(&case, ContentEncoding::Gzip);
}

#[test]
fn assert_decode_state_single_ce_identity() {
    let case = Case {
        header_name: "Content-Encoding",
        expected_state: |s| matches!(s, DecodeState::ContentEncoding(_, _)),
    };
    run_case_single_compression(&case, ContentEncoding::Identity);
}

#[test]
fn assert_decode_state_single_ce_zstd() {
    let case = Case {
        header_name: "Content-Encoding",
        expected_state: |s| matches!(s, DecodeState::ContentEncoding(_, _)),
    };
    run_case_single_compression(&case, ContentEncoding::Zstd);
}
