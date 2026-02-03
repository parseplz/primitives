use tracing::error;

use crate::{
    Method, OneMessageHead, OneRequestLine, OneResponseLine, RequestLine,
    ResponseLine, StatusCode, message_head::header_map::HMap,
    method::METHODS_WITH_BODY, status::InvalidStatusCode,
};

use super::BodyHeader;

pub trait ParseBodyHeaders {
    fn parse_body_headers(&self) -> Option<BodyHeader>;
}

impl ParseBodyHeaders for OneMessageHead<OneRequestLine> {
    fn parse_body_headers(&self) -> Option<BodyHeader> {
        parse_body_headers_request(self.info_line(), self.header_map())
    }
}

impl ParseBodyHeaders for OneMessageHead<OneResponseLine> {
    fn parse_body_headers(&self) -> Option<BodyHeader> {
        parse_body_headers_response(self.info_line(), self.header_map())
    }
}

pub trait RequestMethod {
    fn req_method(&self) -> Method;
}

impl RequestMethod for &OneRequestLine {
    fn req_method(&self) -> Method {
        self.method_enum()
    }
}

impl RequestMethod for &RequestLine {
    fn req_method(&self) -> Method {
        self.method().clone()
    }
}

pub trait ResponseStatus {
    fn status_code(&self) -> Result<StatusCode, InvalidStatusCode>;
}

impl ResponseStatus for &OneResponseLine {
    fn status_code(&self) -> Result<StatusCode, InvalidStatusCode> {
        self.status()
    }
}

impl ResponseStatus for &ResponseLine {
    fn status_code(&self) -> Result<StatusCode, InvalidStatusCode> {
        Ok(*self.status())
    }
}

// If request method is in METHODS_WITH_BODY , build BodyHeader from HeaderMap
#[inline]
fn parse_body_headers_request<T, E>(
    info_line: T,
    headers: &HMap<E>,
) -> Option<BodyHeader>
where
    T: RequestMethod,
    Option<BodyHeader>: for<'a> From<&'a HMap<E>>,
{
    let method = info_line.req_method();
    if METHODS_WITH_BODY.contains(&method) {
        return Option::<BodyHeader>::from(headers);
    }
    None
}

// If status code is in 100-199, 204, 304, then return None else build
// BodyHeader from HeaderMap
#[inline]
fn parse_body_headers_response<T, E>(
    info_line: T,
    headers: &HMap<E>,
) -> Option<BodyHeader>
where
    T: ResponseStatus,
    Option<BodyHeader>: for<'a> From<&'a HMap<E>>,
{
    match info_line.status_code() {
        Ok(scode) => match scode.into() {
            100..=199 | 204 | 304 => None,
            _ => Option::<BodyHeader>::from(headers),
        },
        Err(e) => {
            error!("scode| {:?}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Uri;
    use crate::{
        HeaderMap,
        body_headers::{
            TransferType, content_encoding::ContentEncoding,
            encoding_info::EncodingInfo,
        },
    };
    use bytes::BytesMut;
    use mime_plz::ContentType;

    use super::*;

    //////////////////////////////////////
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
        let msg_head =
            OneMessageHead::<OneRequestLine>::try_from(buf).unwrap();
        let body_headers = msg_head.parse_body_headers();
        assert!(body_headers.is_none());
        let (_, hmap) = msg_head.into_parts();
        let hmap = HeaderMap::from(hmap);
        let info_line = RequestLine::new(Method::GET, Uri::default());
        let body_headers = parse_body_headers_request(&info_line, &hmap);
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
        let result = OneMessageHead::<OneRequestLine>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers();
        assert!(body_headers.is_none());
        let (_, hmap) = result.into_parts();
        let hmap = HeaderMap::from(hmap);
        let info_line = RequestLine::new(Method::POST, Uri::default());
        let body_headers = parse_body_headers_request(&info_line, &hmap);
        assert!(body_headers.is_none());
    }

    #[test]
    fn test_parse_body_headers_req_post_with_ct() {
        let request = "POST /echo HTTP/1.1\r\n\
                       Host: localhost\r\n\
                       Content-Type: application/json\r\n\
                       \r\n";
        let buf = BytesMut::from(request);
        let result = OneMessageHead::<OneRequestLine>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers().unwrap();
        assert!(body_headers.content_type.is_some());
        assert!(body_headers.content_encoding.is_none());
        assert_eq!(body_headers.transfer_type, Some(TransferType::Close));
        assert_eq!(body_headers.transfer_encoding, None);

        let (_, hmap) = result.into_parts();
        let hmap = HeaderMap::from(hmap);
        let info_line = RequestLine::new(Method::POST, Uri::default());
        let body_headers =
            parse_body_headers_request(&info_line, &hmap).unwrap();
        assert!(body_headers.content_type.is_some());
        assert!(body_headers.content_encoding.is_none());
        assert!(body_headers.transfer_type.is_none());
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
        let result = OneMessageHead::<OneRequestLine>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers().unwrap();
        assert_eq!(
            body_headers.content_type.unwrap(),
            ContentType::Application
        );

        assert_eq!(
            body_headers.content_encoding.unwrap(),
            vec![EncodingInfo::new(2, vec![ContentEncoding::Gzip])]
        );
        assert_eq!(body_headers.transfer_type.unwrap(), TransferType::Chunked);
        assert_eq!(
            body_headers.transfer_encoding.unwrap(),
            vec![EncodingInfo::new(3, vec![ContentEncoding::Chunked])]
        );

        let (_, hmap) = result.into_parts();
        let hmap = HeaderMap::from(hmap);
        let info_line = RequestLine::new(Method::POST, Uri::default());
        let body_headers =
            parse_body_headers_request(&info_line, &hmap).unwrap();
        assert_eq!(
            body_headers.content_type.unwrap(),
            ContentType::Application
        );
        assert_eq!(
            body_headers.content_encoding.unwrap(),
            vec![EncodingInfo::new(2, vec![ContentEncoding::Gzip])]
        );
    }

    #[test]
    fn test_parse_body_headers_res_with_cl() {
        let response = "HTTP/1.1 200 OK\r\n\
                                Host: localhost\r\n\
                                Content-Type: text/plain\r\n\
                                Content-Length: 12\r\n\r\n";
        let buf = BytesMut::from(response);
        let result = OneMessageHead::<OneResponseLine>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers().unwrap();
        assert!(body_headers.content_encoding.is_none());
        assert_eq!(body_headers.content_type.unwrap(), ContentType::Text);
        assert!(body_headers.transfer_encoding.is_none());
        assert_eq!(
            body_headers.transfer_type.unwrap(),
            TransferType::ContentLength(12)
        );

        let (_, hmap) = result.into_parts();
        let hmap = HeaderMap::from(hmap);
        let info_line = ResponseLine::new(200.try_into().unwrap());
        let body_headers =
            parse_body_headers_response(&info_line, &hmap).unwrap();
        assert!(body_headers.content_encoding.is_none());
        assert_eq!(body_headers.content_type.unwrap(), ContentType::Text);
        assert!(body_headers.transfer_encoding.is_none());
        assert!(body_headers.transfer_type.is_none());
    }

    #[test]
    fn test_parse_body_headers_res_with_ct() {
        let response = "HTTP/1.1 200 OK\r\n\
                            Host: localhost\r\n\
                            Content-Type: text/plain\r\n\r\n";
        let buf = BytesMut::from(response);
        let result = OneMessageHead::<OneResponseLine>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers().unwrap();
        assert!(body_headers.content_encoding.is_none());
        assert_eq!(body_headers.content_type.unwrap(), ContentType::Text);
        assert!(body_headers.transfer_encoding.is_none());
        assert_eq!(body_headers.transfer_type, Some(TransferType::Close));

        let (_, hmap) = result.into_parts();
        let hmap = HeaderMap::from(hmap);
        let info_line = ResponseLine::new(200.try_into().unwrap());
        let body_headers =
            parse_body_headers_response(&info_line, &hmap).unwrap();
        assert!(body_headers.content_encoding.is_none());
        assert_eq!(body_headers.content_type.unwrap(), ContentType::Text);
        assert!(body_headers.transfer_encoding.is_none());
        assert!(body_headers.transfer_type.is_none());
    }

    #[test]
    fn test_parse_body_headers_res_no_body() {
        let response = "HTTP/1.1 304 OK\r\n\
                        Host: localhost\r\n\
                        Content-Length: 0\r\n\
                        Content-Type: text/plain\r\n\r\n";
        let buf = BytesMut::from(response);
        let result = OneMessageHead::<OneResponseLine>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers();
        assert!(body_headers.is_none());

        let (_, hmap) = result.into_parts();
        let hmap = HeaderMap::from(hmap);
        let info_line = ResponseLine::new(304.try_into().unwrap());
        let body_headers = parse_body_headers_response(&info_line, &hmap);
        assert!(body_headers.is_none());
    }

    #[test]
    fn test_parse_body_headers_status_code_error() {
        let response = "HTTP/1.1 aaa OK\r\n\
                        Host: localhost\r\n\
                        Content-Length: 0\r\n\
                        Content-Type: text/plain\r\n\r\n";
        let buf = BytesMut::from(response);
        let result = OneMessageHead::<OneResponseLine>::try_from(buf).unwrap();
        let body_headers = result.parse_body_headers();
        assert!(body_headers.is_none());

        let (_, hmap) = result.into_parts();
        let hmap = HeaderMap::from(hmap);
        let info_line = ResponseLine::new(304.try_into().unwrap());
        let body_headers = parse_body_headers_response(&info_line, &hmap);
        assert!(body_headers.is_none());
    }
}
