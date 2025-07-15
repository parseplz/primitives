use std::io::Read;
use std::io::Write;
use std::io::copy;

use brotli::Decompressor;

pub fn decompress_brotli<R, W>(input: R, mut buf: W) -> Result<u64, std::io::Error>
where
    R: Read,
    W: Write,
{
    let mut dec = Decompressor::new(input, 4096);
    copy(&mut dec, &mut buf)
}

pub fn decompress_deflate<R, W>(input: R, mut buf: W) -> Result<u64, std::io::Error>
where
    R: Read,
    W: Write,
{
    let mut dec = flate2::read::ZlibDecoder::new(input);
    copy(&mut dec, &mut buf)
}

pub fn decompress_gzip<R, W>(input: R, mut buf: W) -> Result<u64, std::io::Error>
where
    R: Read,
    W: Write,
{
    let mut dec = flate2::read::GzDecoder::new(input);
    copy(&mut dec, &mut buf)
}

pub fn decompress_zstd<R, W>(input: R, mut buf: W) -> Result<u64, std::io::Error>
where
    R: Read,
    W: Write,
{
    //zstd::stream::copy_decode(input, &mut buf)?;
    //Ok(0)
    // -----
    let mut reader = zstd::stream::read::Decoder::new(input)?;
    copy(&mut reader, &mut buf)
}
