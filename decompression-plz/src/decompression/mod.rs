use std::io::{Read, Write, copy};

use brotli::Decompressor;
use bytes::{BufMut, BytesMut, buf::Writer};
use header_plz::body_headers::{content_encoding::ContentEncoding, encoding_info::EncodingInfo};

mod decompressors;
mod magic_bytes;
use decompressors::*;

use crate::{
    decompression::error::DecompressError,
    error::{DecompressErrorStruct, Reason},
};
pub mod error;

pub fn decompress_all(
    mut compressed: &[u8],
    mut writer: &mut Writer<&mut BytesMut>,
    encoding_info: &[EncodingInfo],
) -> Result<BytesMut, DecompressErrorStruct> {
    let mut input: &[u8] = compressed;
    let mut output: BytesMut = writer.get_mut().split();

    for (header_index, encoding_info) in encoding_info.iter().rev().enumerate() {
        for (compression_index, encoding) in encoding_info.encodings().iter().rev().enumerate() {
            let result = decompress(&mut input, &mut writer, encoding.clone());
            match result {
                Ok(_) => {
                    output = writer.get_mut().split();
                    input = &output[..];
                }
                Err(e) => {
                    writer.get_mut().clear();
                    copy(&mut input, writer).unwrap();
                    output = writer.get_mut().split();
                    let reason = if header_index == 0 && compression_index == 0 {
                        Reason::Corrupt
                    } else {
                        Reason::PartialCorrupt(header_index, compression_index)
                    };
                    return Err(DecompressErrorStruct::new(output, None, e, reason));
                }
            }
        }
    }
    Ok(output)
}

pub fn decompress<R, W>(
    mut input: R,
    mut writer: W,
    content_encoding: ContentEncoding,
) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    match content_encoding {
        ContentEncoding::Brotli => decompress_brotli(input, writer),
        ContentEncoding::Compress | ContentEncoding::Zstd => decompress_zstd(input, writer),
        ContentEncoding::Deflate => decompress_deflate(input, writer),
        ContentEncoding::Gzip => decompress_gzip(input, writer),
        ContentEncoding::Identity => {
            copy(&mut input, &mut writer).map_err(DecompressError::Identity)
        }
        ContentEncoding::Chunked => Ok(0),
        ContentEncoding::Unknown(e) => Err(DecompressError::Unknown(e.to_string())),
    }
}

#[cfg(test)]
pub mod tests {
    use crate::decompression::{
        decompress, decompress_all,
        decompressors::{decompress_brotli, decompress_deflate},
        error::DecompressError,
    };
    use bytes::{BufMut, BytesMut};
    use flate2::Compression;
    use header_plz::body_headers::{
        content_encoding::ContentEncoding, encoding_info::EncodingInfo,
    };
    use std::io::{Read, Write};

    pub const INPUT: &[u8] = b"hello world";

    pub fn all_compressed_data() -> Vec<u8> {
        let brotli_compressed = compress_brotli(INPUT);
        let deflate_compressed = compress_deflate(&brotli_compressed);
        let gzip_compressed = compress_gzip(&deflate_compressed);
        compress_zstd(&gzip_compressed)
    }

    pub fn compressed_data() -> Vec<u8> {
        let data = b"hello world";
        let brotli_compressed = compress_brotli(data);
        let deflate_compressed = compress_deflate(&brotli_compressed);
        let gzip_compressed = compress_gzip(&deflate_compressed);
        compress_zstd(&gzip_compressed)
    }

    pub fn compress_brotli(data: &[u8]) -> Vec<u8> {
        let mut compressed = Vec::new();
        {
            let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, 0, 22);
            writer.write_all(data).unwrap();
            writer.flush().unwrap();
        }
        compressed
    }

    pub fn compress_deflate(data: &[u8]) -> Vec<u8> {
        let mut compressed = Vec::new();
        let mut encoder = flate2::write::ZlibEncoder::new(&mut compressed, Compression::fast());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap();
        compressed
    }

    pub fn compress_gzip(data: &[u8]) -> Vec<u8> {
        let mut compressed = Vec::new();
        let mut encoder = flate2::write::GzEncoder::new(&mut compressed, Compression::fast());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap();
        compressed
    }

    pub fn compress_zstd(data: &[u8]) -> Vec<u8> {
        zstd::encode_all(data, 1).unwrap()
    }

    fn test_decompress(data: &[u8], content_encoding: ContentEncoding) -> BytesMut {
        let compressed = match content_encoding {
            ContentEncoding::Brotli => compress_brotli(data),
            ContentEncoding::Deflate => compress_deflate(data),
            ContentEncoding::Gzip => compress_gzip(data),
            ContentEncoding::Zstd | ContentEncoding::Compress => compress_zstd(data),
            ContentEncoding::Identity | ContentEncoding::Unknown(_) | ContentEncoding::Chunked => {
                data.to_vec()
            }
            _ => panic!(),
        };
        let mut buf = BytesMut::new();
        let mut writer = buf.writer();
        decompress(compressed.as_slice(), &mut writer, content_encoding).unwrap();
        writer.into_inner()
    }

    // Individual tests
    #[test]
    fn test_basic_brotli() {
        let result = test_decompress(INPUT, ContentEncoding::Brotli);
        assert_eq!(result.as_ref(), INPUT);
    }

    #[test]
    fn test_basic_deflate() {
        let result = test_decompress(INPUT, ContentEncoding::Deflate);
        assert_eq!(result.as_ref(), INPUT);
    }

    #[test]
    fn test_basic_gzip() {
        let result = test_decompress(INPUT, ContentEncoding::Gzip);
        assert_eq!(result.as_ref(), INPUT);
    }

    #[test]
    fn test_basic_zstd() {
        let result = test_decompress(INPUT, ContentEncoding::Zstd);
        assert_eq!(result.as_ref(), INPUT);
    }

    #[test]
    fn test_basic_compress() {
        let result = test_decompress(INPUT, ContentEncoding::Compress);
        assert_eq!(result.as_ref(), INPUT);
    }

    #[test]
    fn test_basic_identity() {
        let result = test_decompress(INPUT, ContentEncoding::Identity);
        assert_eq!(result.as_ref(), INPUT);
    }

    #[test]
    fn test_basic_chunked() {
        let result = test_decompress(INPUT, ContentEncoding::Chunked);
        assert_eq!(result.as_ref(), b"");
    }

    #[test]
    fn test_basic_unknown() {
        let mut buf = BytesMut::new();
        let mut writer = buf.writer();
        let result = decompress(
            INPUT,
            &mut writer,
            ContentEncoding::Unknown("unknown".to_string()),
        );
        if let Err(DecompressError::Unknown(e)) = result {
            assert_eq!(e, "unknown");
        } else {
            panic!();
        }
    }

    // Combined tests
    #[test]
    fn test_decompress_all_single_header() {
        let input = all_compressed_data();
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = vec![EncodingInfo::new(
            0,
            vec![
                ContentEncoding::Brotli,
                ContentEncoding::Deflate,
                ContentEncoding::Gzip,
                ContentEncoding::Zstd,
                ContentEncoding::Identity,
            ],
        )];

        let result = decompress_all(&input, &mut writer, &einfo_list).unwrap();
        assert_eq!(result, INPUT);
    }

    #[test]
    fn test_decompress_all_multi_header() {
        let input = all_compressed_data();
        let mut buf = BytesMut::new();
        let mut writer = (&mut buf).writer();
        let einfo_list = vec![
            EncodingInfo::new(0, vec![ContentEncoding::Brotli]),
            EncodingInfo::new(1, vec![ContentEncoding::Deflate]),
            EncodingInfo::new(2, vec![ContentEncoding::Gzip]),
            EncodingInfo::new(3, vec![ContentEncoding::Zstd]),
            EncodingInfo::new(4, vec![ContentEncoding::Identity]),
        ];

        let result = decompress_all(&input, &mut writer, &einfo_list).unwrap();
        assert_eq!(result, INPUT);
    }
}
