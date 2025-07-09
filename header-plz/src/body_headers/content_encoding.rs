pub const BROTLI: &str = "br";
pub const CHUNKED: &str = "chunked";
pub const COMPRESS: &str = "compress";
pub const DEFLATE: &str = "deflate";
pub const GZIP: &str = "gzip";
pub const IDENTITY: &str = "identity";
pub const ZSTD: &str = "zstd";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentEncoding {
    Brotli,
    Chunked,
    Compress,
    Deflate,
    Gzip,
    Identity,
    Zstd,
    Unknown(String),
}

impl AsRef<str> for ContentEncoding {
    fn as_ref(&self) -> &str {
        match self {
            ContentEncoding::Brotli => BROTLI,
            ContentEncoding::Chunked => CHUNKED,
            ContentEncoding::Compress => COMPRESS,
            ContentEncoding::Deflate => DEFLATE,
            ContentEncoding::Gzip => GZIP,
            ContentEncoding::Identity => IDENTITY,
            ContentEncoding::Zstd => ZSTD,
            ContentEncoding::Unknown(s) => s,
        }
    }
}

impl From<&str> for ContentEncoding {
    fn from(s: &str) -> Self {
        match s {
            BROTLI => ContentEncoding::Brotli,
            CHUNKED => ContentEncoding::Chunked,
            COMPRESS => ContentEncoding::Compress,
            DEFLATE => ContentEncoding::Deflate,
            GZIP => ContentEncoding::Gzip,
            IDENTITY => ContentEncoding::Identity,
            ZSTD => ContentEncoding::Zstd,
            &_ => ContentEncoding::Unknown(s.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_encoding_from_str() {
        let ce = ContentEncoding::from("br");
        assert_eq!(ce, ContentEncoding::Brotli);
        assert_eq!(ce.as_ref(), BROTLI);

        let ce = ContentEncoding::from("chunked");
        assert_eq!(ce, ContentEncoding::Chunked);
        assert_eq!(ce.as_ref(), CHUNKED);

        let ce = ContentEncoding::from("compress");
        assert_eq!(ce, ContentEncoding::Compress);
        assert_eq!(ce.as_ref(), COMPRESS);

        let ce = ContentEncoding::from("deflate");
        assert_eq!(ce, ContentEncoding::Deflate);
        assert_eq!(ce.as_ref(), DEFLATE);

        let ce = ContentEncoding::from("gzip");
        assert_eq!(ce, ContentEncoding::Gzip);
        assert_eq!(ce.as_ref(), GZIP);

        let ce = ContentEncoding::from("identity");
        assert_eq!(ce, ContentEncoding::Identity);
        assert_eq!(ce.as_ref(), IDENTITY);

        let ce = ContentEncoding::from("zstd");
        assert_eq!(ce, ContentEncoding::Zstd);
        assert_eq!(ce.as_ref(), ZSTD);

        let ce = ContentEncoding::from("hola");
        assert_eq!(ce, ContentEncoding::Unknown("hola".to_string()));
        assert_eq!(ce.as_ref(), "hola");
    }
}
