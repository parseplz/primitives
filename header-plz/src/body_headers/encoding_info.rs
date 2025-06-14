use crate::body_headers::content_encoding::ContentEncoding;

#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq, Clone))]
pub struct EncodingInfo {
    header_index: usize,
    pub encoding: ContentEncoding,
}

impl From<(usize, ContentEncoding)> for EncodingInfo {
    fn from((header_index, encoding): (usize, ContentEncoding)) -> Self {
        EncodingInfo {
            header_index,
            encoding,
        }
    }
}

impl EncodingInfo {
    pub fn new(header_index: usize, encoding: ContentEncoding) -> Self {
        EncodingInfo {
            header_index,
            encoding,
        }
    }

    pub fn iter_from_str(index: usize, val: &str) -> impl Iterator<Item = EncodingInfo> {
        val.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ContentEncoding::from)
            .map(move |enc| EncodingInfo::from((index, enc)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encoding_info_iter_from_str() {
        let data = "gzip, deflate, br, compress,";
        let result: Vec<EncodingInfo> = EncodingInfo::iter_from_str(0, data).collect();
        let verify = vec![
            EncodingInfo::from((0, ContentEncoding::Gzip)),
            EncodingInfo::from((0, ContentEncoding::Deflate)),
            EncodingInfo::from((0, ContentEncoding::Brotli)),
            EncodingInfo::from((0, ContentEncoding::Compress)),
        ];
        assert_eq!(result, verify);
    }
}
