use tracing::error;

use crate::{
    header_struct::HeaderStruct,
    info_line::{request::Request, response::Response},
    methods::{METHODS_WITH_BODY, Method},
};

use super::BodyHeader;

pub trait ParseBodyHeaders {
    fn parse_body_headers(&self) -> Option<BodyHeader>;
}

/*  Steps:
 *      If request method is in METHODS_WITH_BODY , build BodyHeader
 *      from HeaderMap
 */
impl ParseBodyHeaders for HeaderStruct<Request> {
    fn parse_body_headers(&self) -> Option<BodyHeader> {
        let method: Method = self.infoline().method().into();
        if METHODS_WITH_BODY.contains(&method) {
            return Option::<BodyHeader>::from(self.header_map());
        }
        None
    }
}

/*  Steps:
 *      If status code is in 100-199, 204, 304, then return None
 *      else build BodyHeader from HeaderMap
 */
impl ParseBodyHeaders for HeaderStruct<Response> {
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
    use crate::body_headers::{TransferType, content_encoding::ContentEncoding};
    use bytes::BytesMut;
    use mime::ContentType;

    use super::*;

    #[test]
    fn test_parse_body_headers_req_get() {
        let request = "GET / HTTP/1.1\r\n\
                       Host: localhost\r\n\
                       Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
                       Accept-Language: en-US,en;q=0.5\r\n\
                       Accept-Encoding: gzip, deflate\r\n\
                       User-Agent: curl/7.29.0\r\n\
                       Connection: keep-alive\r\n\r\n";
        let buf = BytesMut::from(request);
        let result = HeaderStruct::<Request>::new(buf).unwrap();
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
        let result = HeaderStruct::<Request>::new(buf).unwrap();
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
        let result = HeaderStruct::<Request>::new(buf).unwrap();
        match result.parse_body_headers() {
            Some(body_headers) => {
                assert!(body_headers.content_type.is_some());
                assert!(body_headers.content_encoding.is_none());
                assert_eq!(body_headers.transfer_type, Some(TransferType::Close));
                assert_eq!(body_headers.transfer_encoding, None);
            }
            _ => {
                panic!();
            }
        }
    }

    #[test]
    fn test_parse_body_headers_req_post_with_ct_and_ce() {
        let request = "POST /echo HTTP/1.1\r\n\
                       Host: localhost\r\n\
                       Content-Type: application/json\r\n\
                       Content-Encoding: gzip\r\n\
                       Transfer-Encoding: chunked\r\n\r\n";
        let buf = BytesMut::from(request);
        let result = HeaderStruct::<Request>::new(buf).unwrap();
        match result.parse_body_headers() {
            Some(body_headers) => {
                assert_eq!(body_headers.content_type.unwrap(), ContentType::Application);
                assert_eq!(
                    body_headers.content_encoding.unwrap(),
                    vec![ContentEncoding::Gzip]
                );
                assert_eq!(body_headers.transfer_type.unwrap(), TransferType::Chunked);
                assert!(body_headers.transfer_encoding.is_none());
            }
            _ => {
                panic!();
            }
        }
    }

    #[test]
    fn test_parse_body_headers_res_with_cl() {
        let response = "HTTP/1.1 200 OK\r\n\
                        Host: localhost\r\n\
                        Content-Type: text/plain\r\n\
                        Content-Length: 12\r\n\r\n";
        let buf = BytesMut::from(response);
        let result = HeaderStruct::<Response>::new(buf).unwrap();
        let body_headers = result.parse_body_headers();
        if let Some(body_headers) = body_headers {
            assert!(body_headers.content_encoding.is_none());
            assert_eq!(body_headers.content_type.unwrap(), ContentType::Text);
            assert!(body_headers.transfer_encoding.is_none());
            assert_eq!(
                body_headers.transfer_type.unwrap(),
                TransferType::ContentLength(12)
            );
        } else {
            panic!();
        }
    }

    #[test]
    fn test_parse_body_headers_res_with_ct() {
        let response = "HTTP/1.1 200 OK\r\n\
                        Host: localhost\r\n\
                        Content-Type: text/plain\r\n\r\n";
        let buf = BytesMut::from(response);
        let result = HeaderStruct::<Response>::new(buf).unwrap();
        let body_headers = result.parse_body_headers();
        if let Some(body_headers) = body_headers {
            assert!(body_headers.content_encoding.is_none());
            assert_eq!(body_headers.content_type.unwrap(), ContentType::Text);
            assert!(body_headers.transfer_encoding.is_none());
            assert_eq!(body_headers.transfer_type, Some(TransferType::Close));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_parse_body_headers_res_no_body() {
        let response = "HTTP/1.1 304 OK\r\n\
                        Host: localhost\r\n\
                        Content-Length: 0\r\n\
                        Content-Type: text/plain\r\n\r\n";
        let buf = BytesMut::from(response);
        let result = HeaderStruct::<Response>::new(buf).unwrap();
        let body_headers = result.parse_body_headers();
        assert!(body_headers.is_none());
    }
}
