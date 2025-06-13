use mime_plz::ContentType;

use super::{
    BodyHeader,
    content_encoding::ContentEncoding,
    transfer_types::{TransferType, cl_to_transfer_type, parse_and_remove_chunked},
};
use crate::{const_headers::*, header_map::HeaderMap};

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
    fn from(header_map: &HeaderMap) -> Option<BodyHeader> {
        let mut bh = BodyHeader::default();
        for header in header_map.headers().iter() {
            let key = header.key_as_str();
            if (key.eq_ignore_ascii_case(CONTENT_LENGTH)) && bh.transfer_type.is_none() {
                let transfer_type = cl_to_transfer_type(header.value_as_str());
                bh.transfer_type = Some(transfer_type);
            } else if key.eq_ignore_ascii_case(TRANSFER_ENCODING) {
                bh.transfer_encoding = match_compression(header.value_as_str());
                bh.transfer_type = parse_and_remove_chunked(&mut bh.transfer_encoding);
            } else if key.eq_ignore_ascii_case(CONTENT_ENCODING) {
                bh.content_encoding = match_compression(header.value_as_str());
            } else if key.eq_ignore_ascii_case(CONTENT_TYPE) {
                if let Some((main_type, _)) = header.value_as_str().split_once('/') {
                    bh.content_type = Some(ContentType::from(main_type));
                }
            }
        }

        // if TransferType is Unknown, and if content_encoding or transfer_encoding
        // or content_type is present, then set TransferType to Close
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

//  Convert compression header values to Vec<ContentEncoding>.
pub fn match_compression(value: &str) -> Option<Vec<ContentEncoding>> {
    let encoding: Vec<ContentEncoding> = value
        .split(',')
        .map(|x| x.trim())
        .filter_map(|x| {
            if x.is_empty() {
                None
            } else {
                Some(ContentEncoding::from(x))
            }
        })
        .collect();
    Some(encoding)
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_match_compression() {
        let data = "gzip, deflate, br, compress,";
        let result = match_compression(data);
        let verify = vec![
            ContentEncoding::Gzip,
            ContentEncoding::Deflate,
            ContentEncoding::Brotli,
            ContentEncoding::Compress,
        ];
        assert_eq!(result, Some(verify));
    }

    #[test]
    fn test_header_map_to_body_headers_cl_only() {
        let data = "Content-Length: 10\r\n\r\n";
        let buf = BytesMut::from(data);
        let header_map = HeaderMap::from(buf);
        let result = Option::<BodyHeader>::from(&header_map).unwrap();
        let verify = BodyHeader {
            transfer_type: Some(TransferType::ContentLength(10)),
            ..Default::default()
        };
        assert_eq!(result, verify);
    }

    #[test]
    fn test_header_map_to_body_headers_cl_invalid() {
        let data = "Content-Length: invalid\r\n\r\n";
        let buf = BytesMut::from(data);
        let header_map = HeaderMap::from(buf);
        let verify = BodyHeader {
            transfer_type: Some(TransferType::Close),
            ..Default::default()
        };
        let result = Option::<BodyHeader>::from(&header_map).unwrap();
        assert_eq!(result, verify);
    }

    #[test]
    fn test_header_map_to_body_headers_te_chunked() {
        let data = "Transfer-Encoding: chunked\r\n\r\n";
        let buf = BytesMut::from(data);
        let header_map = HeaderMap::from(buf);
        let verify = BodyHeader {
            transfer_type: Some(TransferType::Chunked),
            ..Default::default()
        };
        let result = Option::<BodyHeader>::from(&header_map).unwrap();
        assert_eq!(result, verify);
    }

    #[test]
    fn test_header_map_to_body_headers_content_length_and_chunked() {
        let data = "Content-Length: 20\r\nTransfer-Encoding: chunked\r\n\r\n";
        let buf = BytesMut::from(data);
        let header_map = HeaderMap::from(buf);
        let verify = BodyHeader {
            transfer_type: Some(TransferType::Chunked),
            ..Default::default()
        };
        let result = Option::<BodyHeader>::from(&header_map).unwrap();
        assert_eq!(result, verify);
    }

    #[test]
    fn test_header_map_to_body_headers_ct_only() {
        let data = "Content-Type: application/json\r\n\r\n";
        let buf = BytesMut::from(data);
        let header_map = HeaderMap::from(buf);
        let verify = BodyHeader {
            content_type: Some(ContentType::Application),
            transfer_type: Some(TransferType::Close),
            ..Default::default()
        };
        let result = Option::<BodyHeader>::from(&header_map).unwrap();
        assert_eq!(result, verify);
    }

    #[test]
    fn test_header_map_to_body_headers_ce_only() {
        let data = "Content-Encoding: gzip\r\n\r\n";
        let buf = BytesMut::from(data);
        let header_map = HeaderMap::from(buf);
        let verify = BodyHeader {
            content_encoding: Some(vec![ContentEncoding::Gzip]),
            transfer_type: Some(TransferType::Close),
            ..Default::default()
        };
        let result = Option::<BodyHeader>::from(&header_map).unwrap();
        assert_eq!(result, verify);
    }
}
