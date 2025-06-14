use mime_plz::ContentType;

use super::{BodyHeader, content_encoding::ContentEncoding, transfer_types::TransferType};
use crate::{
    body_headers::encoding_info::EncodingInfo,
    const_headers::*,
    header_map::{HeaderMap, header::Header},
};

/* Steps:
 *      3. If header.key "Content-Length", and if body_headers.transfer_type is
 *         not set then convert content length to TransferType by calling
 *         cl_to_transfer_type()
 *
 *      4. If header.key is "te" or "Transfer-Encoding",
 *          a. build Vec<TransferEncoding> by calling match_compression() with
 *          header.value_as_str().
 *
 *          b. If chunked value is present, remove it and set transfer_type to
 *          TansferType of chunked
 *
 *      5. If header.key is "ce" or "Content-Encoding", set
 *         body_header.content_encoding to vec built by calling
 *         match_compression() with header.value_as_str().
 *
 *      6. If header.key is "ct" or "Content-Type", split at "/" to get
 *         main content type. Use From<&str> for ContentType to create
 *         ContentType from string. Assign to body_header.content_type.
 *
 *      7. If TransferType is Unknown, and if content_encoding or
 *         transfer_encoding or content_type is present, then set TransferType
 *         to Close
 *
 *      8. Call sanitize() on BodyHeader to remove empty values.
 */

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
            .for_each(|(index, header)| parse_body_headers(&mut bh, index, header));

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
    if (key.eq_ignore_ascii_case(CONTENT_LENGTH)) && bh.transfer_type.is_none() {
        let transfer_type = TransferType::from_cl(header.value_as_str());
        let _ = bh.transfer_type.get_or_insert(transfer_type);
    } else if key.eq_ignore_ascii_case(TRANSFER_ENCODING) {
        let mut einfo_iter = EncodingInfo::iter_from_str(index, header.value_as_str());
        bh.transfer_encoding
            .get_or_insert_with(Vec::new)
            .extend(einfo_iter);
        if bh.is_chunked_te() {
            bh.transfer_type = Some(TransferType::Chunked)
        }
    } else if key.eq_ignore_ascii_case(CONTENT_ENCODING) {
        let einfo_iter = EncodingInfo::iter_from_str(index, header.value_as_str());
        bh.content_encoding
            .get_or_insert_with(Vec::new)
            .extend(einfo_iter);
    } else if key.eq_ignore_ascii_case(CONTENT_TYPE) {
        if let Some((main_type, _)) = header.value_as_str().split_once('/') {
            bh.content_type = Some(ContentType::from(main_type));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    // ----- chunked
    #[test]
    fn test_body_header_from_header_map_chunked() {
        let input = "Transfer-Encoding: chunked\r\n\r\n";
        let result = build_body_header(input);
        let einfo = EncodingInfo::new(0, ContentEncoding::Chunked);
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
        let input = "Content-Length: 20\r\nTransfer-Encoding: chunked\r\n\r\n";
        let result = build_body_header(input);
        let einfo = EncodingInfo::new(1, ContentEncoding::Chunked);
        let verify = BodyHeader {
            transfer_type: Some(TransferType::Chunked),
            transfer_encoding: Some(vec![einfo]),
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

    // ----- Content Encoding
    #[test]
    fn test_body_headers_from_header_map_ce_only() {
        let input = "Content-Encoding: gzip\r\n\r\n";
        let result = build_body_header(input);
        let einfo = EncodingInfo::new(0, ContentEncoding::Gzip);
        let verify = BodyHeader {
            content_encoding: Some(vec![einfo]),
            transfer_type: Some(TransferType::Close),
            ..Default::default()
        };
        assert_eq!(result, verify);
    }
}
