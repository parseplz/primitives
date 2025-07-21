use header_plz::body_headers::{BodyHeader, encoding_info::EncodingInfo};

pub enum EncodingType {
    TransferEncoding,
    ContentEncoding,
}

impl EncodingType {
    pub fn encoding_info<'a>(
        &self,
        body_headers: Option<&'a mut BodyHeader>,
    ) -> Option<&'a mut Vec<EncodingInfo>> {
        match self {
            EncodingType::TransferEncoding => {
                body_headers.and_then(|bh| bh.transfer_encoding.as_mut())
            }
            EncodingType::ContentEncoding => {
                body_headers.and_then(|bh| bh.content_encoding.as_mut())
            }
        }
    }
}
