use std::io::Read;
use std::io::Write;
use std::io::copy;

use header_plz::body_headers::content_encoding::ContentEncoding;
pub mod error;
use error::DecompressError;

pub fn decompress<R, W>(
    mut input: R,
    mut writer: W,
    content_encoding: ContentEncoding,
) -> Result<u64, DecompressError>
where
    R: Read + AsRef<[u8]>,
    W: Write,
{
    let mut input = std::io::Cursor::new(input);
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

pub fn decompress_brotli<R, W>(input: R, mut buf: W) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    copy(&mut brotli::Decompressor::new(input, 4096), &mut buf).map_err(DecompressError::Brotli)
}

pub fn decompress_deflate<R, W>(input: R, mut buf: W) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    copy(&mut flate2::read::ZlibDecoder::new(input), &mut buf).map_err(DecompressError::Deflate)
}

pub fn decompress_gzip<R, W>(input: R, mut buf: W) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    copy(&mut flate2::read::GzDecoder::new(input), &mut buf).map_err(DecompressError::Gzip)
}

pub fn decompress_zstd<R, W>(input: R, mut buf: W) -> Result<u64, DecompressError>
where
    R: Read,
    W: Write,
{
    //zstd::stream::copy_decode(input, &mut buf)?;
    //Ok(0)
    // -----
    copy(
        &mut zstd::stream::read::Decoder::new(input).map_err(DecompressError::Zstd)?,
        &mut buf,
    )
    .map_err(DecompressError::Zstd)
}
