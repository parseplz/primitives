use content_encoding::ContentEncoding;
use mime_plz::ContentType;
use transfer_types::TransferType;

use crate::body_headers::encoding_info::EncodingInfo;
pub mod content_encoding;
pub mod encoding_info;
pub mod transfer_types;

mod from_header_map;
pub mod parse;

#[derive(Default)]
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq, Clone))]
pub struct BodyHeader {
    pub content_encoding: Option<Vec<EncodingInfo>>,
    pub content_type: Option<ContentType>,
    pub transfer_encoding: Option<Vec<EncodingInfo>>,
    pub transfer_type: Option<TransferType>,
}

impl BodyHeader {
    pub fn sanitize(self) -> Option<Self> {
        if self.content_encoding.is_some()
            || self.content_type.is_some()
            || self.transfer_encoding.is_some()
            || self.transfer_type.is_some()
        {
            Some(self)
        } else {
            None
        }
    }

    pub fn content_type(&self) -> ContentType {
        self.content_type
            .map_or(ContentType::Unknown, |ct| ct)
    }

    pub fn chunked_te_position(&self) -> Option<(usize, usize)> {
        self.transfer_encoding
            .as_ref()
            .and_then(|einfo_vec| {
                einfo_vec
                    .iter()
                    .enumerate()
                    .find_map(|(outer_idx, ei)| {
                        ei.encodings()
                            .iter()
                            .position(|enc| *enc == ContentEncoding::Chunked)
                            .map(|inner_idx| (outer_idx, inner_idx))
                    })
            })
    }

    pub fn update_transfer_type(&mut self, transfer_type: TransferType) {
        if self
            .transfer_type
            .is_none_or(|tt| transfer_type >= tt)
        {
            self.transfer_type = Some(transfer_type);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_bodyheader_sanitize_all() {
        let body = BodyHeader {
            content_encoding: Some(vec![EncodingInfo::new(
                0,
                vec![ContentEncoding::Gzip],
            )]),
            content_type: Some(ContentType::Application),
            transfer_encoding: Some(vec![EncodingInfo::new(
                0,
                vec![ContentEncoding::Gzip],
            )]),
            transfer_type: Some(TransferType::Close),
        };
        let sbody = body.clone().sanitize();
        assert_eq!(sbody.unwrap(), body);
    }

    #[test]
    fn test_bodyheader_sanitize_none() {
        let body = BodyHeader::default();
        assert!(body.sanitize().is_none());
    }

    #[test]
    fn test_bodyheader_contenttype() {
        let body = BodyHeader {
            content_type: Some(ContentType::Application),
            ..Default::default()
        };
        assert_eq!(body.content_type(), ContentType::Application);
    }

    #[test]
    fn test_bodyheader_contenttype_default() {
        let body = BodyHeader::default();
        assert_eq!(body.content_type(), ContentType::Unknown);
    }
}
