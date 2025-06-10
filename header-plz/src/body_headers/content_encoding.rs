pub const CHUNKED: &str = "chunked";
pub const BROTLI: &str = "br";
pub const COMPRESS: &str = "compress";
pub const DEFLATE: &str = "deflate";
pub const GZIP: &str = "gzip";
pub const IDENTITY: &str = "identity";
pub const ZSTD: &str = "zstd";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentEncoding {
    Brotli,
    Compress,
    Deflate,
    Gzip,
    Identity,
    Zstd,
    Chunked,
    Unknown(String),
}

impl AsRef<str> for ContentEncoding {
    fn as_ref(&self) -> &str {
        match self {
            ContentEncoding::Brotli => BROTLI,
            ContentEncoding::Compress => COMPRESS,
            ContentEncoding::Deflate => DEFLATE,
            ContentEncoding::Gzip => GZIP,
            ContentEncoding::Identity => IDENTITY,
            ContentEncoding::Zstd => ZSTD,
            ContentEncoding::Chunked => CHUNKED,
            ContentEncoding::Unknown(s) => s,
        }
    }
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
            &_ => ContentEncoding::Unknown(s.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_encoding_from_str() {
        let ce = ContentEncoding::Unknown("hola".to_string());
        assert_eq!(ContentEncoding::from("hola"), ce);
        assert_eq!(ce.as_ref(), "hola");
        dbg!(std::mem::size_of::<ContentEncoding>());
    }
}
