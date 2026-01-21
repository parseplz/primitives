use crate::HeaderMap;
use crate::message_head::header_map::{HMap, Hmap};
use crate::message_head::header_map::{HeaderStr, HeaderVersion};

use super::{BodyHeader, transfer_types::TransferType};
use crate::{
    OneHeaderMap, body_headers::encoding_info::EncodingInfo, const_headers::*,
};

impl From<&OneHeaderMap> for Option<BodyHeader> {
    fn from(header_map: &OneHeaderMap) -> Self {
        let mut bh = BodyHeader::from(header_map);

        // if TransferType is Unknown, and if content_encoding or
        // transfer_encoding or content_type is present, then set TransferType
        // to Close
        if bh.transfer_type.is_none()
            && (bh.content_encoding.is_some()
                || bh.transfer_encoding.is_some()
                || bh.content_type.is_some())
        {
            bh.transfer_type = Some(TransferType::Close);
        }

        bh.sanitize()
    }
}

impl From<&HeaderMap> for Option<BodyHeader> {
    fn from(header_map: &HeaderMap) -> Self {
        let bh = BodyHeader::from(header_map);
        bh.sanitize()
    }
}

impl<T> From<&HMap<T>> for BodyHeader
where
    T: Hmap + HeaderVersion + HeaderStr,
{
    #[inline(always)]
    fn from(header_map: &HMap<T>) -> BodyHeader {
        let mut bh = BodyHeader::default();
        header_map.into_iter().enumerate().for_each(|(index, header)| {
            parse_body_headers(&mut bh, index, header)
        });
        bh
    }
}

enum BodyHeaderId {
    ContentLength,
    TransferEncoding,
    ContentEncoding,
    ContentType,
    None,
}

pub fn parse_body_headers<T>(bh: &mut BodyHeader, index: usize, header: &T)
where
    T: Hmap + HeaderVersion + HeaderStr,
{
    let Some(value) = header.value_as_str() else {
        return;
    };
    let key = header.key_as_ref();

    use BodyHeaderId::*;
    match identify_header(header.is_one_one(), key) {
        ContentLength => {
            let transfer_type = TransferType::from_cl(value);
            bh.update_transfer_type(transfer_type);
        }
        TransferEncoding => {
            let einfo = EncodingInfo::from((index, value));
            bh.transfer_encoding.get_or_insert_with(Vec::new).push(einfo);
            if bh.chunked_te_position().is_some() {
                bh.transfer_type = Some(TransferType::Chunked);
            }
        }
        ContentEncoding => {
            let einfo = EncodingInfo::from((index, value));
            bh.content_encoding.get_or_insert_with(Vec::new).push(einfo);
        }
        ContentType => {
            if let Some((main_type, _)) = value.split_once('/') {
                bh.content_type = Some(mime_plz::ContentType::from(main_type));
            }
        }
        None => {}
    }
}

#[inline(always)]
fn identify_header(is_one_one: bool, key: &[u8]) -> BodyHeaderId {
    if is_one_one {
        if key.eq_ignore_ascii_case(CONTENT_LENGTH) {
            return BodyHeaderId::ContentLength;
        }
        if key.eq_ignore_ascii_case(TRANSFER_ENCODING) {
            return BodyHeaderId::TransferEncoding;
        }
    }
    if key.eq_ignore_ascii_case(CONTENT_ENCODING) {
        return BodyHeaderId::ContentEncoding;
    }
    if key.eq_ignore_ascii_case(CONTENT_TYPE) {
        return BodyHeaderId::ContentType;
    }
    BodyHeaderId::None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body_headers::ContentEncoding;
    use bytes::BytesMut;
    use mime_plz::ContentType;

    fn build_body_header(input: &str) -> BodyHeader {
        let header_map = OneHeaderMap::from(BytesMut::from(input));
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
    fn test_body_headers_from_header_map_ce_only_one() {
        let input = "Content-Encoding: gzip\r\n\r\n";
        let header_map = OneHeaderMap::from(BytesMut::from(input));
        let result = Option::<BodyHeader>::from(&header_map);
        let einfo = EncodingInfo::new(0, vec![ContentEncoding::Gzip]);
        let verify = BodyHeader {
            content_encoding: Some(vec![einfo]),
            transfer_type: Some(TransferType::Close),
            ..Default::default()
        };
        assert_eq!(result.unwrap(), verify);
    }

    #[test]
    fn test_body_headers_from_header_map_ce_only_two() {
        let input = "Content-Encoding: gzip\r\n\r\n";
        let header_map =
            HeaderMap::from(OneHeaderMap::from(BytesMut::from(input)));
        let result = Option::<BodyHeader>::from(&header_map);
        let einfo = EncodingInfo::new(0, vec![ContentEncoding::Gzip]);
        let verify = BodyHeader {
            content_encoding: Some(vec![einfo]),
            ..Default::default()
        };
        assert_eq!(result.unwrap(), verify);
    }

    #[test]
    fn test_body_header_from_header_map_ce_multiple_one() {
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

    #[test]
    fn test_body_header_from_header_map_ce_multiple_two() {
        let input = "Host: localhost\r\n\
                     Content-Encoding: br, compress\r\n\
                     Content-Length: 20\r\n\
                     Content-Encoding: deflate, gzip\r\n\
                     Authentication: bool\r\n\
                     Content-Encoding: identity, zstd\r\n\
                     Connection: close\r\n\r\n";
        let header_map =
            HeaderMap::from(OneHeaderMap::from(BytesMut::from(input)));
        let result = Option::<BodyHeader>::from(&header_map);
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
            ..Default::default()
        };
        assert_eq!(result.unwrap(), verify);
    }

    // ----- Content type
    #[test]
    fn test_body_header_from_header_map_ct_only_one() {
        let input = "Content-Type: application/json\r\n\r\n";
        let header_map = OneHeaderMap::from(BytesMut::from(input));
        let result = Option::<BodyHeader>::from(&header_map);
        let verify = BodyHeader {
            content_type: Some(ContentType::Application),
            transfer_type: Some(TransferType::Close),
            ..Default::default()
        };
        assert_eq!(result.unwrap(), verify);
    }

    #[test]
    fn test_body_header_from_header_map_ct_only_two() {
        let input = "Content-Type: application/json\r\n\r\n";
        let header_map =
            HeaderMap::from(OneHeaderMap::from(BytesMut::from(input)));
        let result = Option::<BodyHeader>::from(&header_map);
        let verify = BodyHeader {
            content_type: Some(ContentType::Application),
            ..Default::default()
        };
        assert_eq!(result.unwrap(), verify);
    }
}
