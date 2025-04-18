use content_encoding::ContentEncoding;
use mime_plz::ContentType;
use transfer_types::TransferType;

pub mod content_encoding;
pub mod transfer_types;

mod from_header_map;
pub mod parse;

#[derive(Default)]
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq, Clone))]
pub struct BodyHeader {
    pub content_encoding: Option<Vec<ContentEncoding>>,
    pub content_type: Option<ContentType>,
    pub transfer_encoding: Option<Vec<ContentEncoding>>,
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
        self.content_type.map_or(ContentType::Unknown, |ct| ct)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_bodyheader_sanitize_all() {
        let body = BodyHeader {
            content_encoding: Some(vec![ContentEncoding::Gzip]),
            content_type: Some(ContentType::Application),
            transfer_encoding: Some(vec![ContentEncoding::Gzip]),
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
}
