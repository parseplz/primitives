use tracing::error;

use crate::{
    Request, Response,
    message_head::MessageHead,
    methods::{METHODS_WITH_BODY, Method},
};

use super::BodyHeader;

pub trait ParseBodyHeaders {
    fn parse_body_headers(&self) -> Option<BodyHeader>;
}

// If request method is in METHODS_WITH_BODY , build BodyHeader from HeaderMap
impl ParseBodyHeaders for MessageHead<Request> {
    fn parse_body_headers(&self) -> Option<BodyHeader> {
        let method: Method = self.infoline().method().into();
        if METHODS_WITH_BODY.contains(&method) {
            return Option::<BodyHeader>::from(self.header_map());
        }
        None
    }
}

// If status code is in 100-199, 204, 304, then return None else build
// BodyHeader from HeaderMap
impl ParseBodyHeaders for MessageHead<Response> {
    fn parse_body_headers(&self) -> Option<BodyHeader> {
        match self.infoline().status_as_u8() {
            Ok(scode) => match scode {
                100..=199 | 204 | 304 => None,
                _ => Option::<BodyHeader>::from(self.header_map()),
            },
            Err(e) => {
                error!("scode| {:?}", e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::body_headers::{
        TransferType, content_encoding::ContentEncoding, encoding_info::EncodingInfo,
    };
    use bytes::BytesMut;
    use mime_plz::ContentType;

    use super::*;

    #[test]
    fn test_parse_body_headers_req_get() {
        let request = "GET / HTTP/1.1\r\n\
                       Host: localhost\r\n\
                       Accept: text/html\r\n\
                       Accept-Language: en-US,en;q=0.5\r\n\
                       Accept-Encoding: gzip, deflate\r\n\
                       User-Agent: curl/7.29.0\r\n\
                       Connection: keep-alive\r\n\r\n";
        let buf = BytesMut::from(request);
        let result = MessageHead::<Request>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers();
        assert!(body_headers.is_none());
    }

    #[test]
    fn test_parse_body_headers_req_post_no_body() {
        let request = "POST /echo HTTP/1.1\r\n\
                       Host: localhost\r\n\
                       Accept-Language: en-US,en;q=0.5\r\n\
                       Accept-Encoding: gzip, deflate\r\n\
                       User-Agent: curl/7.29.0\r\n\
                       Connection: keep-alive\r\n\r\n";
        let buf = BytesMut::from(request);
        let result = MessageHead::<Request>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers();
        assert!(body_headers.is_none());
    }

    #[test]
    fn test_parse_body_headers_req_post_with_ct() {
        let request = "POST /echo HTTP/1.1\r\n\
                       Host: localhost\r\n\
                       Content-Type: application/json\r\n\
                       \r\n";
        let buf = BytesMut::from(request);
        let result = MessageHead::<Request>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers().unwrap();
        assert!(body_headers.content_type.is_some());
        assert!(body_headers.content_encoding.is_none());
        assert_eq!(body_headers.transfer_type, Some(TransferType::Close));
        assert_eq!(body_headers.transfer_encoding, None);
    }

    #[test]
    fn test_parse_body_headers_req_post_with_ct_and_ce() {
        let request = "POST /echo HTTP/1.1\r\n\
                                   Host: localhost\r\n\
                                   Content-Type: application/json\r\n\
                                   Content-Encoding: gzip\r\n\
                                   Transfer-Encoding: chunked\r\n\r\n";
        let buf = BytesMut::from(request);
        let result = MessageHead::<Request>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers().unwrap();
        assert_eq!(body_headers.content_type.unwrap(), ContentType::Application);

        assert_eq!(
            body_headers.content_encoding.unwrap(),
            vec![EncodingInfo::new(2, vec![ContentEncoding::Gzip])]
        );
        assert_eq!(body_headers.transfer_type.unwrap(), TransferType::Chunked);
        assert_eq!(
            body_headers.transfer_encoding.unwrap(),
            vec![EncodingInfo::new(3, vec![ContentEncoding::Chunked])]
        );
    }

    #[test]
    fn test_parse_body_headers_res_with_cl() {
        let response = "HTTP/1.1 200 OK\r\n\
                                Host: localhost\r\n\
                                Content-Type: text/plain\r\n\
                                Content-Length: 12\r\n\r\n";
        let buf = BytesMut::from(response);
        let result = MessageHead::<Response>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers().unwrap();
        assert!(body_headers.content_encoding.is_none());
        assert_eq!(body_headers.content_type.unwrap(), ContentType::Text);
        assert!(body_headers.transfer_encoding.is_none());
        assert_eq!(
            body_headers.transfer_type.unwrap(),
            TransferType::ContentLength(12)
        );
    }

    #[test]
    fn test_parse_body_headers_res_with_ct() {
        let response = "HTTP/1.1 200 OK\r\n\
                            Host: localhost\r\n\
                            Content-Type: text/plain\r\n\r\n";
        let buf = BytesMut::from(response);
        let result = MessageHead::<Response>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers().unwrap();
        assert!(body_headers.content_encoding.is_none());
        assert_eq!(body_headers.content_type.unwrap(), ContentType::Text);
        assert!(body_headers.transfer_encoding.is_none());
        assert_eq!(body_headers.transfer_type, Some(TransferType::Close));
    }

    #[test]
    fn test_parse_body_headers_res_no_body() {
        let response = "HTTP/1.1 304 OK\r\n\
                        Host: localhost\r\n\
                        Content-Length: 0\r\n\
                        Content-Type: text/plain\r\n\r\n";
        let buf = BytesMut::from(response);
        let result = MessageHead::<Response>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers();
        assert!(body_headers.is_none());
    }

    #[test]
    fn test_parse_body_headers_status_code_error() {
        let response = "HTTP/1.1 aaa OK\r\n\
                        Host: localhost\r\n\
                        Content-Length: 0\r\n\
                        Content-Type: text/plain\r\n\r\n";
        let buf = BytesMut::from(response);
        let result = MessageHead::<Response>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers();
        assert!(body_headers.is_none());
    }
}
