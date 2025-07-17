use mime_plz::ContentType;

use super::{BodyHeader, transfer_types::TransferType};
use crate::{
    Header, HeaderMap, body_headers::encoding_info::EncodingInfo,
    const_headers::*,
};

impl From<&HeaderMap> for Option<BodyHeader> {
    fn from(header_map: &HeaderMap) -> Self {
        let bh = BodyHeader::from(header_map);
        bh.sanitize()
    }
}

impl From<&HeaderMap> for BodyHeader {
    #[inline(always)]
    fn from(header_map: &HeaderMap) -> BodyHeader {
        let mut bh = BodyHeader::default();
        header_map
            .headers()
            .iter()
            .enumerate()
            .for_each(|(index, header)| {
                parse_body_headers(&mut bh, index, header)
            });

        // if TransferType is Unknown, and if content_encoding or transfer_encoding
        // or content_type is present, then set TransferType to Close
        if bh.transfer_type.is_none()
            && (bh.content_encoding.is_some()
                || bh.transfer_encoding.is_some()
                || bh.content_type.is_some())
        {
            bh.transfer_type = Some(TransferType::Close);
        }
        bh
    }
}

pub fn parse_body_headers(bh: &mut BodyHeader, index: usize, header: &Header) {
    let key = header.key_as_str();
    if key.eq_ignore_ascii_case(CONTENT_LENGTH) {
        let transfer_type = TransferType::from_cl(header.value_as_str());
        bh.update_transfer_type(transfer_type);
    } else if key.eq_ignore_ascii_case(TRANSFER_ENCODING) {
        let einfo = EncodingInfo::from((index, header.value_as_str()));
        bh.transfer_encoding
            .get_or_insert_with(Vec::new)
            .push(einfo);
        if bh.chunked_te_position().is_some() {
            bh.transfer_type = Some(TransferType::Chunked)
        }
    } else if key.eq_ignore_ascii_case(CONTENT_ENCODING) {
        let einfo = EncodingInfo::from((index, header.value_as_str()));
        bh.content_encoding
            .get_or_insert_with(Vec::new)
            .push(einfo);
    } else if key.eq_ignore_ascii_case(CONTENT_TYPE) {
        if let Some((main_type, _)) = header.value_as_str().split_once('/') {
            bh.content_type = Some(ContentType::from(main_type));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body_headers::ContentEncoding;
    use bytes::BytesMut;

    fn build_body_header(input: &str) -> BodyHeader {
        let header_map = HeaderMap::from(BytesMut::from(input));
        BodyHeader::from(&header_map)
    }

    // ---- Content Length
    #[test]
    fn test_body_header_from_header_map_cl() {
        let input = "Content-Length: 10\r\n\r\n";
        let result = build_body_header(input);
        let verify = BodyHeader {
            transfer_type: Some(TransferType::ContentLength(10)),
            ..Default::default()
        };
        assert_eq!(result, verify);
    }

    #[test]
    fn test_body_header_from_header_map_cl_invalid() {
        let input = "Content-Length: invalid\r\n\r\n";
        let result = build_body_header(input);
        let verify = BodyHeader {
            transfer_type: Some(TransferType::Close),
            ..Default::default()
        };
        assert_eq!(result, verify);
    }

    #[test]
    fn test_body_header_from_header_map_cl_multiple() {
        let input = "Content-Length: 10\r\n\
                     Content-Length: 20\r\n\
                     Content-Length: 30\r\n\r\n";
        let result = build_body_header(input);
        let verify = BodyHeader {
            transfer_type: Some(TransferType::ContentLength(30)),
            ..Default::default()
        };
        assert_eq!(result, verify);
    }

    // ----- chunked
    #[test]
    fn test_body_header_from_header_map_chunked() {
        let input = "Transfer-Encoding: chunked\r\n\r\n";
        let result = build_body_header(input);
        let einfo = EncodingInfo::new(0, vec![ContentEncoding::Chunked]);
        let verify = BodyHeader {
            transfer_encoding: Some(vec![einfo]),
            transfer_type: Some(TransferType::Chunked),
            ..Default::default()
        };
        assert_eq!(result, verify);
    }

    // ----- chunked + cl
    #[test]
    fn test_body_header_from_header_map_cl_and_chunked() {
        let input = "Content-Length: 20\r\n\
                     Transfer-Encoding: chunked\r\n\r\n";
        let result = build_body_header(input);
        let einfo = EncodingInfo::new(1, vec![ContentEncoding::Chunked]);
        let verify = BodyHeader {
            transfer_type: Some(TransferType::Chunked),
            transfer_encoding: Some(vec![einfo]),
            ..Default::default()
        };
        assert_eq!(result, verify);
    }

    // ----- multiple cl + chunked
    #[test]
    fn test_body_header_from_header_map_multiple_cl_and_chunked() {
        let input = "Content-Length: 20\r\n\
                     Transfer-Encoding: chunked\r\n\
                     Content-Length: 30\r\n\r\n";
        let result = build_body_header(input);
        let einfo = EncodingInfo::new(1, vec![ContentEncoding::Chunked]);
        let verify = BodyHeader {
            transfer_type: Some(TransferType::Chunked),
            transfer_encoding: Some(vec![einfo]),
            ..Default::default()
        };
        assert_eq!(result, verify);
    }

    // ----- Transfer Encoding
    #[test]
    fn test_body_header_from_header_map_te_multiple() {
        let input = "Host: localhost\r\n\
                     Transfer-Encoding: br, compress\r\n\
                     Content-Length: 20\r\n\
                     Transfer-Encoding: deflate, gzip\r\n\
                     Authentication: bool\r\n\
                     Transfer-Encoding: identity, zstd\r\n\
                     Connection: close\r\n\
                     Transfer-Encoding: chunked\r\n\r\n";
        let result = build_body_header(input);
        let einfo = vec![
            EncodingInfo::new(
                1,
                vec![ContentEncoding::Brotli, ContentEncoding::Compress],
            ),
            EncodingInfo::new(
                3,
                vec![ContentEncoding::Deflate, ContentEncoding::Gzip],
            ),
            EncodingInfo::new(
                5,
                vec![ContentEncoding::Identity, ContentEncoding::Zstd],
            ),
            EncodingInfo::new(7, vec![ContentEncoding::Chunked]),
        ];
        let verify = BodyHeader {
            transfer_encoding: Some(einfo),
            transfer_type: Some(TransferType::Chunked),
            ..Default::default()
        };
        assert_eq!(result, verify);
    }

    // ----- Content Encoding
    #[test]
    fn test_body_headers_from_header_map_ce_only() {
        let input = "Content-Encoding: gzip\r\n\r\n";
        let result = build_body_header(input);
        let einfo = EncodingInfo::new(0, vec![ContentEncoding::Gzip]);
        let verify = BodyHeader {
            content_encoding: Some(vec![einfo]),
            transfer_type: Some(TransferType::Close),
            ..Default::default()
        };
        assert_eq!(result, verify);
    }

    #[test]
    fn test_body_header_from_header_map_ce_multiple() {
        let input = "Host: localhost\r\n\
                     Content-Encoding: br, compress\r\n\
                     Content-Length: 20\r\n\
                     Content-Encoding: deflate, gzip\r\n\
                     Authentication: bool\r\n\
                     Content-Encoding: identity, zstd\r\n\
                     Connection: close\r\n\r\n";
        let result = build_body_header(input);
        let einfo = vec![
            EncodingInfo::new(
                1,
                vec![ContentEncoding::Brotli, ContentEncoding::Compress],
            ),
            EncodingInfo::new(
                3,
                vec![ContentEncoding::Deflate, ContentEncoding::Gzip],
            ),
            EncodingInfo::new(
                5,
                vec![ContentEncoding::Identity, ContentEncoding::Zstd],
            ),
        ];
        let verify = BodyHeader {
            content_encoding: Some(einfo),
            transfer_type: Some(TransferType::ContentLength(20)),
            ..Default::default()
        };
        assert_eq!(result, verify);
    }

    // ----- Content type
    #[test]
    fn test_body_header_from_header_map_ct_only() {
        let input = "Content-Type: application/json\r\n\r\n";
        let result = build_body_header(input);
        let verify = BodyHeader {
            content_type: Some(ContentType::Application),
            transfer_type: Some(TransferType::Close),
            ..Default::default()
        };
        assert_eq!(result, verify);
    }
}
