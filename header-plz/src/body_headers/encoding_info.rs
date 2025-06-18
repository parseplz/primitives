use crate::body_headers::content_encoding::ContentEncoding;

#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq, Clone))]
pub struct EncodingInfo {
    pub header_index: usize,
    encodings: Vec<ContentEncoding>,
}

impl From<(usize, &str)> for EncodingInfo {
    fn from((header_index, values): (usize, &str)) -> Self {
        let encodings = values
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ContentEncoding::from)
            .collect();
        EncodingInfo {
            header_index,
            encodings,
        }
    }
}

impl EncodingInfo {
    pub fn new(header_index: usize, encodings: Vec<ContentEncoding>) -> Self {
        EncodingInfo {
            header_index,
            encodings,
        }
    }

    pub fn encodings(&self) -> &[ContentEncoding] {
        &self.encodings
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encoding_info_iter_from_str() {
        let data = "gzip, deflate, br, compress,";
        let result: EncodingInfo = EncodingInfo::from((0, data));
        let verify = vec![
            ContentEncoding::Gzip,
            ContentEncoding::Deflate,
            ContentEncoding::Brotli,
            ContentEncoding::Compress,
        ];
        assert_eq!(result.encodings, verify);
    }
}
