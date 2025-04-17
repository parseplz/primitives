pub const CHUNKED: &str = "chunked";
pub const BROTLI: &str = "br";
pub const COMPRESS: &str = "compress";
pub const DEFLATE: &str = "deflate";
pub const GZIP: &str = "gzip";
pub const IDENTITY: &str = "identity";
pub const ZSTD: &str = "zstd";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentEncoding {
    Brotli,
    Compress,
    Deflate,
    Gzip,
    Identity,
    Zstd,
    Chunked,
}

impl From<&str> for ContentEncoding {
    fn from(s: &str) -> Self {
        match s {
            BROTLI => ContentEncoding::Brotli,
            COMPRESS => ContentEncoding::Compress,
            DEFLATE => ContentEncoding::Deflate,
            GZIP => ContentEncoding::Gzip,
            IDENTITY => ContentEncoding::Identity,
            ZSTD => ContentEncoding::Zstd,
            CHUNKED => ContentEncoding::Chunked,
            &_ => unreachable!("unknown content encoding| {}", s),
        }
    }
}
