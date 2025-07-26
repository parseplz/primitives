#![allow(warnings)]
pub use body_plz::variants::Body;
pub use bytes::BytesMut;
use decompression_plz::DecompressTrait;
use decompression_plz::state::DecodeState;
use header_plz::{HeaderMap, body_headers::BodyHeader};

pub const INPUT: &[u8] = b"hello world";
pub mod test_cases;

#[derive(Debug, PartialEq)]
pub struct TestMessage {
    header_map: HeaderMap,
    body_header: Option<BodyHeader>,
    body: Option<Body>,
    extra_body: Option<BytesMut>,
}

impl DecompressTrait for TestMessage {
    fn get_body(&mut self) -> Body {
        self.body.take().unwrap()
    }

    fn get_extra_body(&mut self) -> Option<BytesMut> {
        self.extra_body.take()
    }

    fn set_body(&mut self, body: Body) {
        self.body = Some(body);
    }

    fn body_headers_as_mut(&mut self) -> &mut Option<BodyHeader> {
        &mut self.body_header
    }

    fn header_map(&self) -> &HeaderMap {
        &self.header_map
    }

    fn header_map_as_mut(&mut self) -> &mut HeaderMap {
        &mut self.header_map
    }
}

impl TestMessage {
    pub fn build(
        headers: BytesMut,
        body: Body,
        extra: Option<BytesMut>,
    ) -> Self {
        let header_map = HeaderMap::from(headers);
        let body_header = BodyHeader::from(&header_map);
        Self {
            header_map,
            body_header: Some(body_header),
            body: Some(body),
            extra_body: extra,
        }
    }

    pub fn into_bytes(self) -> BytesMut {
        let mut bytes = self.header_map.into_bytes();
        bytes.unsplit(self.body.unwrap().into_bytes().unwrap());
        bytes
    }
}

enum EncodingKind {
    Te,
    Ce,
}

struct Case {
    header_name: &'static str,
    expected_state: fn(&DecodeState<TestMessage>) -> bool,
}
