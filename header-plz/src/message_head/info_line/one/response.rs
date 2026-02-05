use bytes::{Buf, BufMut, BytesMut};

use crate::{
    Version,
    abnf::CRLF,
    status::{InvalidStatusCode, StatusCode},
};

use super::{InfoLine, InfoLineError};

// Response Info Line
#[derive(Debug, PartialEq)]
pub struct ResponseLine {
    version: BytesMut, // Version + space
    status: BytesMut,  // status
    reason: BytesMut,  // space + Reason + CRLF
}

/* Steps:
 *      1. For http/1.1 | http/1.0 | http/0.9  => version = len(http/1.*) + space + 1 = 9
 *      2. Status code is always 3 digits
 *      3. Remainder is reason + CRLF
 */
impl InfoLine for ResponseLine {
    fn try_build_infoline(
        mut data: BytesMut,
    ) -> Result<ResponseLine, InfoLineError> {
        // "1" in decimal
        let index = if data[5] == 49 {
            9
        } else {
            7
        };
        let version = data.split_to(index);
        // status code always 3 digits
        let status = data.split_to(3);
        Ok(ResponseLine {
            version,
            status,
            reason: data,
        })
    }

    fn into_bytes(mut self) -> BytesMut {
        self.status.unsplit(self.reason);
        self.version.unsplit(self.status);
        self.version
    }

    fn as_chain(&self) -> impl Buf {
        (self.version[..].chain(&self.status[..])).chain(&self.reason[..])
    }
}

impl ResponseLine {
    pub fn new(version: BytesMut, status: BytesMut, reason: BytesMut) -> Self {
        Self {
            version,
            status,
            reason,
        }
    }

    pub fn status(&self) -> Result<StatusCode, InvalidStatusCode> {
        StatusCode::from_bytes(&self.status)
    }

    pub fn is_ws_handshake(&self) -> Result<bool, InvalidStatusCode> {
        self.status().map(|x| x == 101)
    }

    pub fn into_parts(self) -> (BytesMut, BytesMut, BytesMut) {
        (self.version, self.status, self.reason)
    }

    pub fn set_status(&mut self, status: u16) {
        self.status.clear();
        self.status.extend_from_slice(status.to_string().as_bytes());
    }
}

impl From<(StatusCode, Version)> for ResponseLine {
    fn from((status, version): (StatusCode, Version)) -> Self {
        let version = BytesMut::from(version.for_response_line());
        let reason_str =
            StatusCode::canonical_reason(&status).unwrap_or_default();
        let mut reason = BytesMut::with_capacity(1 + reason_str.len() + 2);
        reason.put_u8(b' ');
        reason.extend_from_slice(reason_str.as_bytes());
        reason.extend_from_slice(CRLF.as_ref());

        let status = status.as_str().into();

        Self {
            version,
            reason,
            status,
        }
    }
}

impl From<StatusCode> for ResponseLine {
    fn from(status: StatusCode) -> Self {
        Self::from((status, Version::H11))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_status_code_for_response_line() {
        let expected = "HTTP/1.1 200 OK\r\n";
        let status = StatusCode::OK;
        let response = ResponseLine::from(status);
        let mut chain = response.as_chain();
        let result = chain.copy_to_bytes(chain.remaining());
        drop(chain);
        let verify = response.into_bytes();
        assert_eq!(result, verify);
        assert_eq!(verify, expected);
    }

    #[test]
    fn test_from_status_code_and_version_for_response_line() {
        let expected = "HTTP/2 200 OK\r\n";
        let status = StatusCode::OK;
        let version = Version::H2;
        let response = ResponseLine::from((status, version));
        let mut chain = response.as_chain();
        let result = chain.copy_to_bytes(chain.remaining());
        drop(chain);
        let verify = response.into_bytes();
        assert_eq!(result, verify);
        assert_eq!(verify, expected);
    }

    #[test]
    fn test_infoline_response_oneone() {
        let response = "HTTP/1.1 200 OK\r\n";
        let buf = BytesMut::from(response);
        let verify = buf.clone();
        let initial_ptr = buf.as_ptr_range();
        let response = ResponseLine::try_build_infoline(buf).unwrap();
        assert_eq!(response.version, "HTTP/1.1 ");
        assert_eq!(response.status, "200");
        assert_eq!(response.reason, " OK\r\n");
        let mut chain = response.as_chain();
        let result = chain.copy_to_bytes(chain.remaining());
        drop(chain);
        assert_eq!(result, verify);
        let toverify = response.into_bytes();
        assert_eq!(toverify.as_ptr_range(), initial_ptr);
        assert_eq!(toverify, verify);
    }

    #[test]
    fn test_infoline_response_two() {
        let response = "HTTP/2 200 OK\r\n";
        let buf = BytesMut::from(response);
        let verify = buf.clone();
        let initial_ptr = buf.as_ptr_range();
        let response = ResponseLine::try_build_infoline(buf).unwrap();
        assert_eq!(response.version, "HTTP/2 ");
        assert_eq!(response.status, "200");
        assert_eq!(response.reason, " OK\r\n");
        let mut chain = response.as_chain();
        let result = chain.copy_to_bytes(chain.remaining());
        drop(chain);
        let toverify = response.into_bytes();
        assert_eq!(result, verify);
        assert_eq!(toverify.as_ptr_range(), initial_ptr);
        assert_eq!(toverify, verify);
    }

    #[test]
    fn test_infoline_response_is_ws_handshake_true() {
        let response = "HTTP/1.1 101 Switching Protocols\r\n";
        let buf = BytesMut::from(response);
        let response = ResponseLine::try_build_infoline(buf).unwrap();
        assert!(response.is_ws_handshake().unwrap());
    }

    #[test]
    fn test_infoline_response_is_ws_handshake_false() {
        let response = "HTTP/1.1 200 OK\r\n";
        let buf = BytesMut::from(response);
        let response = ResponseLine::try_build_infoline(buf).unwrap();
        assert!(!response.is_ws_handshake().unwrap());
    }

    #[test]
    fn test_infoline_response_set_status() {
        let response = "HTTP/1.1 200 OK\r\n";
        let buf = BytesMut::from(response);
        let mut response = ResponseLine::try_build_infoline(buf).unwrap();
        response.set_status(300);
        let expected = "HTTP/1.1 300 OK\r\n";
        assert_eq!(expected, response.into_bytes());
    }
}
