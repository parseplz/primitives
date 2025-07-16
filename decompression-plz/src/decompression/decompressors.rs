use std::io::Read;
use std::io::Write;
use std::io::copy;

use crate::decompression::error::DecompressError;

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
