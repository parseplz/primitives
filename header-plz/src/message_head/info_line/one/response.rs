use bytes::BytesMut;
use thiserror::Error;

use super::{InfoLine, InfoLineError};

// Response Info Line
#[derive(Debug, PartialEq)]
pub struct ResponseLine {
    version: BytesMut, // Version + space
    status: BytesMut,  // status
    reason: BytesMut,  // space + Reason + CRLF
}

/* Steps:
 *      1. For http/1.1 | http/1.0  => version = len(http/1.*) + space + 1 = 9
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
            // TODO: Add Checks for http/2 and http/3
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
}

#[derive(Error, Debug)]
pub enum StatusCodeError {
    // Utf8 Error
    #[error("Not valid utf8| {0}")]
    Utf8(#[from] std::str::Utf8Error),
    // Parse int
    #[error("Parse int| {0}")]
    ParseInt(#[from] std::num::ParseIntError),
}

impl ResponseLine {
    pub fn new(version: BytesMut, status: BytesMut, reason: BytesMut) -> Self {
        Self {
            version,
            status,
            reason,
        }
    }

    pub fn status(&self) -> &[u8] {
        &self.status
    }

    pub fn status_as_u8(&self) -> Result<u16, StatusCodeError> {
        Ok(std::str::from_utf8(&self.status)?.parse::<u16>()?)
    }

    pub fn is_ws_handshake(&self) -> Result<bool, StatusCodeError> {
        self.status_as_u8().map(|x| x == 101)
    }

    pub fn into_parts(self) -> (BytesMut, BytesMut, BytesMut) {
        (self.version, self.status, self.reason)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let toverify = response.into_bytes();
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
}
