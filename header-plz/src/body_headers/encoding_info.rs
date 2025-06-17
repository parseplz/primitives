use crate::body_headers::content_encoding::ContentEncoding;

#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq, Clone))]
pub struct EncodingInfo {
    header_index: usize,
    encoding: ContentEncoding,
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

    pub fn encoding(&self) -> &ContentEncoding {
        &self.encoding
    }

    pub fn iter_from_str(index: usize, val: &str) -> impl Iterator<Item = EncodingInfo> {
        val.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ContentEncoding::from)
            .map(move |enc| EncodingInfo::from((index, enc)))
    }
}

pub fn encodings_in_single_header(encoding_info: &[EncodingInfo]) -> Option<usize> {
    let mut iter = encoding_info.iter();
    let first = iter.next().unwrap().header_index;
    iter.all(|elem| elem.header_index == first)
        .then(|| encoding_info[0].header_index)
}

pub fn iter_encoding_header_positions(
    encoding_info: &[EncodingInfo],
) -> impl DoubleEndedIterator<Item = usize> {
    let mut last = None;
    encoding_info
        .iter()
        .map(|e| e.header_index)
        .filter(move |&index| match last {
            Some(prev) if prev == index => false,
            _ => {
                last = Some(index);
                true
            }
        })
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

    #[test]
    fn test_are_encodings_in_same_header_same() {
        let input = vec![
            EncodingInfo::from((9, ContentEncoding::Gzip)),
            EncodingInfo::from((9, ContentEncoding::Deflate)),
            EncodingInfo::from((9, ContentEncoding::Brotli)),
            EncodingInfo::from((9, ContentEncoding::Compress)),
        ];

        assert_eq!(encodings_in_single_header(&input), Some(9));
    }

    #[test]
    fn test_are_encodings_in_same_header_diff() {
        let input = vec![
            EncodingInfo::from((0, ContentEncoding::Gzip)),
            EncodingInfo::from((1, ContentEncoding::Deflate)),
            EncodingInfo::from((2, ContentEncoding::Brotli)),
            EncodingInfo::from((3, ContentEncoding::Compress)),
        ];

        assert!(encodings_in_single_header(&input).is_none());
    }

    #[test]
    fn test_header_positions() {
        let input = vec![
            EncodingInfo::from((0, ContentEncoding::Gzip)),
            EncodingInfo::from((1, ContentEncoding::Deflate)),
            EncodingInfo::from((2, ContentEncoding::Brotli)),
            EncodingInfo::from((3, ContentEncoding::Compress)),
        ];
        let pos: Vec<usize> = iter_encoding_header_positions(&input).collect();

        assert_eq!(pos, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_header_positions_duplicate() {
        let input = vec![
            EncodingInfo::from((0, ContentEncoding::Gzip)),
            EncodingInfo::from((0, ContentEncoding::Deflate)),
            EncodingInfo::from((1, ContentEncoding::Brotli)),
            EncodingInfo::from((1, ContentEncoding::Compress)),
        ];
        let pos: Vec<usize> = iter_encoding_header_positions(&input).collect();
        assert_eq!(pos, vec![0, 1]);
    }
}
