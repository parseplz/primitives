use std::borrow::Cow;
use std::str::{self};

use bytes::BytesMut;

use super::{InfoLine, InfoLineError};
use crate::abnf::SP;
use crate::uri::InvalidUri;
use crate::uri::path::PathAndQuery;

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
        let mut index = data.iter().position(|&x| x == SP).ok_or(
            InfoLineError::FirstOWS(
                String::from_utf8_lossy(&data).to_string(),
            ),
        )?;
        let method = data.split_to(index + 1);
        // 2. Second OWS
        index = data.iter().position(|&x| x == SP).ok_or(
            InfoLineError::SecondOWS(
                String::from_utf8_lossy(&data).to_string(),
            ),
        )?;
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
}

impl RequestLine {
    pub fn new(method: BytesMut, uri: BytesMut, version: BytesMut) -> Self {
        Self {
            method,
            uri,
            version,
        }
    }

    pub fn method(&self) -> &[u8] {
        self.method.trim_ascii_end()
    }

    pub fn method_raw(&self) -> &BytesMut {
        &self.method
    }

    pub fn set_method_raw(&mut self, method: BytesMut) {
        self.method = method;
    }

    // Uri Related
    pub fn uri_as_mut(&mut self) -> &mut BytesMut {
        &mut self.uri
    }

    pub fn uri_as_string(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.uri)
    }

    pub fn uri(&self) -> Result<PathAndQuery, InvalidUri> {
        PathAndQuery::try_from(self.uri.as_ref())
    }

    pub fn into_parts(self) -> (BytesMut, BytesMut, BytesMut) {
        (self.method, self.uri, self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_infoline_request_basic() -> Result<(), Box<dyn Error>> {
        let req = "GET /echo HTTP/1.1\r\n";
        let buf = BytesMut::from(req);
        let verify = buf[0..20].to_owned();
        let verify_ptr = buf[0..20].as_ptr_range();
        let request = RequestLine::try_build_infoline(buf)?;
        assert_eq!(request.method(), b"GET");
        assert_eq!(request.uri_as_string(), "/echo");
        assert_eq!(request.version, " HTTP/1.1\r\n");
        let toverify = request.into_bytes();
        assert_eq!(verify_ptr, toverify.as_ptr_range());
        assert_eq!(toverify, verify);
        Ok(())
    }

    #[test]
    fn test_infoline_request_connect() {
        let req = "CONNECT www.google.com:443 HTTP/1.1\r\n";
        let buf = BytesMut::from(req);
        let verify_ptr = buf[..37].as_ptr_range();
        let verify = buf.clone();
        match RequestLine::try_build_infoline(buf) {
            Ok(info_line) => {
                assert_eq!(info_line.method, "CONNECT ");
                assert_eq!(info_line.uri, "www.google.com:443");
                assert_eq!(info_line.version, " HTTP/1.1\r\n");
                let assembled = info_line.into_bytes();
                assert_eq!(assembled, verify);
                assert_eq!(verify_ptr, assembled.as_ptr_range());
            }
            _ => {
                panic!();
            }
        }
    }

    #[test]
    fn test_infoline_request_http() {
        let req = "GET http://www.google.com/ HTTP/1.1\r\n";
        let buf = BytesMut::from(req);
        let verify_ptr = buf[..].as_ptr_range();
        let verify = buf.clone();
        match RequestLine::try_build_infoline(buf) {
            Ok(info_line) => {
                assert_eq!(info_line.method, "GET ");
                assert_eq!(info_line.uri, "http://www.google.com/");
                assert_eq!(info_line.version, " HTTP/1.1\r\n");
                let assembled = info_line.into_bytes();
                assert_eq!(assembled, verify);
                assert_eq!(verify_ptr, assembled.as_ptr_range());
            }
            _ => {
                panic!();
            }
        }
    }

    #[test]
    fn test_infoline_request_http_port() {
        let req = "GET http://www.google.com:8080/ HTTP/1.1\r\n";
        let buf = BytesMut::from(req);
        let verify_ptr = buf[..].as_ptr_range();
        let verify = buf.clone();
        match RequestLine::try_build_infoline(buf) {
            Ok(info_line) => {
                assert_eq!(info_line.method, "GET ");
                assert_eq!(info_line.uri, "http://www.google.com:8080/");
                assert_eq!(info_line.version, " HTTP/1.1\r\n");
                let assembled = info_line.into_bytes();
                assert_eq!(assembled, verify);
                assert_eq!(verify_ptr, assembled.as_ptr_range());
            }
            _ => {
                panic!();
            }
        }
    }

    #[test]
    fn test_return_queries() {
        let req = "GET /users?param=value&param2=value2 HTTP/1.1\r\n\r\n";
        let buf = BytesMut::from(req);
        let info_line = RequestLine::try_build_infoline(buf).unwrap();
        let uri = info_line.uri().unwrap();
        let query = uri.query().unwrap();
        assert_eq!("param=value&param2=value2", query);
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
