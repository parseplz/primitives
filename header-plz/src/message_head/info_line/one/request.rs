use bytes::{Buf, BufMut, BytesMut};
use std::borrow::Cow;
use std::str::{self};

use super::{InfoLine, InfoLineError};
use crate::abnf::SP;
use crate::uri::InvalidUri;
use crate::{Method, Uri, Version};

// Request Info Line
#[derive(Debug, PartialEq)]
pub struct RequestLine {
    method: BytesMut,  // Method + Space
    uri: BytesMut,     //  Uri
    version: BytesMut, // Space + Version + CRLF
}

/* Steps:
 *      1. Find first OWS
 *      2. Call split_to(index)
 *      3. Find second OWS
 *      4. Call split_to(index)
 *      5. Return first, second, remaining (contains CRLF).
 *
 * Error:
 *      InfoLineError::FirstOWS     [1]
 *      InfoLineError::SecondOWS    [2]
 */

impl InfoLine for RequestLine {
    fn try_build_infoline(
        mut data: BytesMut,
    ) -> Result<RequestLine, InfoLineError> {
        let index = match data.iter().position(|&x| x == SP) {
            Some(i) => i,
            None => {
                return Err(InfoLineError::first_ows(data));
            }
        };
        let mut method = data.split_to(index + 1);
        let index = match data.iter().position(|&x| x == SP) {
            Some(i) => i,
            None => {
                method.unsplit(data);
                return Err(InfoLineError::second_ows(method));
            }
        };
        let uri = data.split_to(index);
        Ok(RequestLine {
            method,
            uri,
            version: data,
        })
    }

    fn into_bytes(mut self) -> BytesMut {
        self.uri.unsplit(self.version);
        self.method.unsplit(self.uri);
        self.method
    }

    fn as_chain(&self) -> impl Buf {
        (self.method[..].chain(&self.uri[..])).chain(&self.version[..])
    }
}

impl RequestLine {
    pub fn new(method: BytesMut, uri: BytesMut, version: BytesMut) -> Self {
        Self {
            method,
            uri,
            version,
        }
    }

    pub fn method_bytes(&self) -> &[u8] {
        self.method.trim_ascii_end()
    }

    pub fn method_enum(&self) -> Method {
        Method::from(self.method_bytes())
    }

    pub fn set_method(&mut self, method: Method) {
        self.method.clear();
        self.method.extend_from_slice(method.as_str().as_bytes());
        self.method.put_u8(SP);
    }

    // Uri Related
    pub fn set_uri(&mut self, uri: &[u8]) {
        self.uri.clear();
        self.uri.extend_from_slice(uri);
    }

    pub fn uri_as_ref(&self) -> &[u8] {
        &self.uri
    }

    pub fn uri_as_string(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.uri)
    }

    pub fn uri(&self) -> Result<Uri, InvalidUri> {
        Uri::try_from(self.uri.as_ref())
    }

    pub fn into_parts(self) -> (BytesMut, BytesMut, BytesMut) {
        (self.method, self.uri, self.version)
    }
}

impl From<(Method, &Uri, Version)> for RequestLine {
    fn from((method, uri, version): (Method, &Uri, Version)) -> Self {
        let mut method_bytes = BytesMut::with_capacity(method.len() + 1);
        method_bytes.extend_from_slice(method.as_ref());
        method_bytes.put_u8(b' ');
        RequestLine::new(
            method_bytes,
            uri.path_and_query().as_str().into(),
            version.for_request_line().into(),
        )
    }
}

impl From<(Method, &Uri)> for RequestLine {
    fn from((method, uri): (Method, &Uri)) -> Self {
        RequestLine::from((method, uri, Version::H11))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uri::path::PathAndQuery;

    #[test]
    fn test_infoline_request_basic() {
        let req = "GET /echo HTTP/1.1\r\n";
        let buf = BytesMut::from(req);
        let verify = buf[0..20].to_owned();
        let verify_ptr = buf[0..20].as_ptr_range();
        let request = RequestLine::try_build_infoline(buf).unwrap();
        assert_eq!(request.method_bytes(), b"GET");
        assert_eq!(request.uri_as_string(), "/echo");
        assert_eq!(request.version, " HTTP/1.1\r\n");
        let mut chain = request.as_chain();
        let result = chain.copy_to_bytes(chain.remaining());
        drop(chain);
        assert_eq!(result, verify);
        let toverify = request.into_bytes();
        assert_eq!(verify_ptr, toverify.as_ptr_range());
        assert_eq!(toverify, verify);
    }

    #[test]
    fn test_infoline_request_connect() {
        let req = "CONNECT www.google.com:443 HTTP/1.1\r\n";
        let buf = BytesMut::from(req);
        let verify_ptr = buf[..37].as_ptr_range();
        let verify = buf.clone();
        let info_line = RequestLine::try_build_infoline(buf).unwrap();
        assert_eq!(info_line.method, "CONNECT ");
        assert_eq!(info_line.uri, "www.google.com:443");
        assert_eq!(info_line.version, " HTTP/1.1\r\n");
        let mut chain = info_line.as_chain();
        let result = chain.copy_to_bytes(chain.remaining());
        drop(chain);
        assert_eq!(result, verify);
        let assembled = info_line.into_bytes();
        assert_eq!(assembled, verify);
        assert_eq!(verify_ptr, assembled.as_ptr_range());
    }

    #[test]
    fn test_infoline_request_http() {
        let req = "GET http://www.google.com/ HTTP/1.1\r\n";
        let buf = BytesMut::from(req);
        let verify_ptr = buf[..].as_ptr_range();
        let verify = buf.clone();
        let info_line = RequestLine::try_build_infoline(buf).unwrap();
        assert_eq!(info_line.method, "GET ");
        assert_eq!(info_line.uri, "http://www.google.com/");
        assert_eq!(info_line.version, " HTTP/1.1\r\n");
        let mut chain = info_line.as_chain();
        let result = chain.copy_to_bytes(chain.remaining());
        drop(chain);
        assert_eq!(result, verify);
        let assembled = info_line.into_bytes();
        assert_eq!(assembled, verify);
        assert_eq!(verify_ptr, assembled.as_ptr_range());
    }

    #[test]
    fn test_infoline_request_http_port() {
        let req = "GET http://www.google.com:8080/ HTTP/1.1\r\n";
        let buf = BytesMut::from(req);
        let verify_ptr = buf[..].as_ptr_range();
        let verify = buf.clone();
        let info_line = RequestLine::try_build_infoline(buf).unwrap();
        assert_eq!(info_line.method, "GET ");
        assert_eq!(info_line.uri, "http://www.google.com:8080/");
        assert_eq!(info_line.version, " HTTP/1.1\r\n");
        let mut chain = info_line.as_chain();
        let result = chain.copy_to_bytes(chain.remaining());
        drop(chain);
        assert_eq!(result, verify);
        let assembled = info_line.into_bytes();
        assert_eq!(assembled, verify);
        assert_eq!(verify_ptr, assembled.as_ptr_range());
    }

    #[test]
    fn test_return_queries() {
        let req = "GET /users?param=value&param2=value2 HTTP/1.1\r\n\r\n";
        let buf = BytesMut::from(req);
        let info_line = RequestLine::try_build_infoline(buf).unwrap();
        let uri = info_line.uri().unwrap();
        let query = uri.path_and_query().query().unwrap();
        assert_eq!("param=value&param2=value2", query);
    }

    #[test]
    fn test_one_request_set_method() {
        let input = "GET / HTTP/1.1\r\n";
        let mut line = RequestLine::try_build_infoline(input.into()).unwrap();
        line.set_method(Method::POST);
        assert_eq!(line.method_bytes(), b"POST");
        assert_eq!(line.method_enum(), Method::POST);
    }

    #[test]
    fn test_one_request_uri() {
        let input = "GET / HTTP/1.1\r\n";
        let mut line = RequestLine::try_build_infoline(input.into()).unwrap();
        let uri = b"/dead_end";
        line.set_uri(uri);
        let verify = "GET /dead_end HTTP/1.1\r\n";
        assert_eq!(line.into_bytes(), verify);
    }

    #[test]
    fn test_one_request_line_from_method_uri() {
        let uri = Uri::builder().path("/foo?a=1&b=2#23").build().unwrap();
        let line = RequestLine::from((Method::GET, &uri, Version::H11));
        let input = "GET /foo?a=1&b=2 HTTP/1.1\r\n";
        let verify = RequestLine::try_build_infoline(input.into()).unwrap();
        assert_eq!(line, verify);
    }

    #[test]
    fn test_build_one_request_line_minimal() {
        let uri = Uri::default();
        let line = RequestLine::from((Method::GET, &uri, Version::H2));
        let input = "GET / HTTP/2\r\n";
        let verify = RequestLine::try_build_infoline(input.into()).unwrap();
        assert_eq!(line, verify);
    }

    #[test]
    fn test_build_one_request_line_encoded_query() {
        let method = Method::GET;
        let path = PathAndQuery::from_shared(
            "/search?q=hello%20world&lang=en".into(),
        )
        .unwrap();
        let uri = Uri::builder().path(path).build().unwrap();
        let line = RequestLine::from((method, &uri, Version::H3));

        let input = "GET /search?q=hello%20world&lang=en HTTP/3\r\n";
        let verify = RequestLine::try_build_infoline(input.into()).unwrap();
        assert_eq!(line, verify);
    }

    #[test]
    fn test_missing_first_space_returns_full_input() {
        let raw = "GET/index.htmlHTTP/1.1";
        let input = BytesMut::from(raw);
        let expected = input.clone();
        let err = RequestLine::try_build_infoline(input).unwrap_err();
        let verify = InfoLineError::first_ows(expected);
        assert_eq!(verify, err);
    }
    #[test]
    fn test_missing_second_space_unsplit_works() {
        let raw = "GET /index.htmlHTTP/1.1";
        let input = BytesMut::from(raw);
        let expected = input.clone();
        let err = RequestLine::try_build_infoline(input).unwrap_err();
        let verify = InfoLineError::second_ows(expected);
        assert_eq!(verify, err);
    }

    /*
    #[test]
    fn it_should_return_first_line_query_params() {
        let raw = HttpRaw::new(
            b"GET /users?param=value&param2=value2 HTTP/1.1\r\n\r\n".to_vec(),
        );
        let mut params = raw.first_line().unwrap().query().unwrap().params();
        assert_eq!(2, params.len());

        let param2 = params.pop().unwrap();
        let param2_raw = param2.raw();
        assert_eq!(b"param2=value2", param2_raw.data);
        assert_eq!(Bound::Included(&23), param2_raw.range.start_bound());
        assert_eq!(Bound::Excluded(&36), param2_raw.range.end_bound());
        let param2_parts = param2.parts().unwrap();
        assert_eq!(b"param2", param2_parts.0.data);
        assert_eq!(Bound::Included(&23), param2_parts.0.range.start_bound());
        assert_eq!(Bound::Excluded(&29), param2_parts.0.range.end_bound());
        assert_eq!(b"value2", param2_parts.1.data);
        assert_eq!(Bound::Included(&30), param2_parts.1.range.start_bound());
        assert_eq!(Bound::Excluded(&36), param2_parts.1.range.end_bound());

        let param1 = params.pop().unwrap();
        let param1_raw = param1.raw();
        assert_eq!(b"param=value", param1_raw.data);
        assert_eq!(Bound::Included(&11), param1_raw.range.start_bound());
        assert_eq!(Bound::Excluded(&22), param1_raw.range.end_bound());
        let param1_parts = param1.parts().unwrap();
        assert_eq!(b"param", param1_parts.0.data);
        assert_eq!(Bound::Included(&11), param1_parts.0.range.start_bound());
        assert_eq!(Bound::Excluded(&16), param1_parts.0.range.end_bound());
        assert_eq!(b"value", param1_parts.1.data);
        assert_eq!(Bound::Included(&17), param1_parts.1.range.start_bound());
        assert_eq!(Bound::Excluded(&22), param1_parts.1.range.end_bound());
    }


    #[test]
    fn test_return_first_line_query_params_end_ampersand() {
        let req = "GET /users?param=value& HTTP/1.1\r\n\r\n";
        let buf = BytesMut::from(req);
        assert_eq!(1, params.len());

        let param1 = params.pop().unwrap();
        let param1_raw = param1.raw();
        assert_eq!(b"param=value", param1_raw.data);
        assert_eq!(Bound::Included(&11), param1_raw.range.start_bound());
        assert_eq!(Bound::Excluded(&22), param1_raw.range.end_bound());
        let param1_parts = param1.parts().unwrap();
        assert_eq!(b"param", param1_parts.0.data);
        assert_eq!(Bound::Included(&11), param1_parts.0.range.start_bound());
        assert_eq!(Bound::Excluded(&16), param1_parts.0.range.end_bound());
        assert_eq!(b"value", param1_parts.1.data);
        assert_eq!(Bound::Included(&17), param1_parts.1.range.start_bound());
        assert_eq!(Bound::Excluded(&22), param1_parts.1.range.end_bound());
    }

    */
}
