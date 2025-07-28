pub use body_plz::variants::Body;
pub use bytes::BytesMut;
use decompression_plz::DecompressTrait;
use std::io::Write;

use flate2::Compression;
use header_plz::{
    HeaderMap,
    body_headers::{
        BodyHeader, content_encoding::ContentEncoding,
        encoding_info::EncodingInfo,
    },
};

pub const INPUT: &[u8] = b"hello world";

pub const ALL_COMPRESSIONS: &str = "br, deflate, gzip, zstd";

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

pub fn all_compressed_data() -> BytesMut {
    let brotli_compressed = compress_brotli(INPUT);
    let deflate_compressed = compress_deflate(&brotli_compressed);
    let gzip_compressed = compress_gzip(&deflate_compressed);
    let zstd_compressed = compress_zstd(&gzip_compressed);
    BytesMut::from(zstd_compressed.as_slice())
}

pub fn single_compression(encoding: &ContentEncoding) -> BytesMut {
    let compressed = match encoding {
        ContentEncoding::Brotli => compress_brotli(INPUT),
        ContentEncoding::Deflate => compress_deflate(INPUT),
        ContentEncoding::Gzip => compress_gzip(INPUT),
        ContentEncoding::Zstd | ContentEncoding::Compress => {
            compress_zstd(INPUT)
        }
        ContentEncoding::Identity => INPUT.to_vec(),
        _ => panic!(),
    };
    BytesMut::from(compressed.as_slice())
}

pub fn all_encoding_info_multi_header() -> Vec<EncodingInfo> {
    vec![
        EncodingInfo::new(1, vec![ContentEncoding::Brotli]),
        EncodingInfo::new(3, vec![ContentEncoding::Deflate]),
        EncodingInfo::new(5, vec![ContentEncoding::Identity]),
        EncodingInfo::new(7, vec![ContentEncoding::Gzip]),
        EncodingInfo::new(9, vec![ContentEncoding::Zstd]),
    ]
}

pub fn all_encoding_info_single_header() -> Vec<EncodingInfo> {
    vec![EncodingInfo::new(
        0,
        vec![
            ContentEncoding::Brotli,
            ContentEncoding::Deflate,
            ContentEncoding::Identity,
            ContentEncoding::Gzip,
            ContentEncoding::Zstd,
        ],
    )]
}

pub fn compress_brotli(data: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    {
        let mut writer =
            brotli::CompressorWriter::new(&mut compressed, 4096, 0, 22);
        writer.write_all(data).unwrap();
        writer.flush().unwrap();
    }
    compressed
}

pub fn compress_deflate(data: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    let mut encoder =
        flate2::write::ZlibEncoder::new(&mut compressed, Compression::fast());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap();
    compressed
}

pub fn compress_gzip(data: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    let mut encoder =
        flate2::write::GzEncoder::new(&mut compressed, Compression::fast());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap();
    compressed
}

pub fn compress_zstd(data: &[u8]) -> Vec<u8> {
    zstd::encode_all(data, 1).unwrap()
}
