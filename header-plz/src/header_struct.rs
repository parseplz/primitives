use crate::{error::HttpReadError, header_map::HeaderMap, info_line::InfoLine};
use bytes::BytesMut;

#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq))]
pub struct HeaderStruct<T> {
    info_line: T,
    header_map: HeaderMap,
}

/* Struct to represent the Header region Infoline + HeaderMap.
 *
 * info_line : Request/Response
 */

impl<T> HeaderStruct<T>
where
    T: InfoLine,
{
    /* Steps:
     *      1. Find CR in buf.
     *      2. Split buf at CR_index + 2 (CRLF)
     *      3. Build Infoline
     *
     * Error:
     *      HttpDecodeError::InfoLine       [3]
     *      HttpDecodeError::HeaderStruct   [Default]
     */

    pub fn new(mut data: BytesMut) -> Result<Self, HttpReadError> {
        if let Some(infoline_index) = data.iter().position(|&x| x == 13) {
            let raw = data.split_to(infoline_index + 2);
            let info_line = T::build_infoline(raw)?;
            return Ok(Self {
                info_line,
                header_map: HeaderMap::new(data),
            });
        }
        Err(HttpReadError::HeaderStruct(
            String::from_utf8_lossy(&data).to_string(),
        ))
    }

    // Convert into Data
    pub fn into_data(self) -> BytesMut {
        let mut data = self.info_line.into_data();
        data.unsplit(self.header_map.into_data());
        data
    }

    pub fn header_map(&self) -> &HeaderMap {
        &self.header_map
    }

    pub fn infoline(&self) -> &T {
        &self.info_line
    }

    pub fn infoline_as_mut(&mut self) -> &mut T {
        &mut self.info_line
    }

    pub fn header_map_as_mut(&mut self) -> &mut HeaderMap {
        &mut self.header_map
    }
}

#[cfg(test)]
mod tests {

    use crate::info_line::{request::Request, response::Response};

    use super::*;

    #[test]
    fn test_header_struct_build_request() {
        let request = "GET / HTTP/1.1\r\n\
                       Host: localhost\r\n\
                       Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
                       Accept-Language: en-US,en;q=0.5\r\n\
                       Accept-Encoding: gzip, deflate\r\n\
                       User-Agent: curl/7.29.0\r\n\
                       Connection: keep-alive\r\n\r\n
                       ";
        let buf = BytesMut::from(request);
        let org = buf.as_ptr_range();
        let result = HeaderStruct::<Request>::new(buf).unwrap();
        assert_eq!(result.info_line.method(), b"GET");
        assert_eq!(result.info_line.uri_as_string(), "/");
        let verify = result.into_data();
        assert_eq!(verify, request);
        assert_eq!(verify.as_ptr_range(), org);
    }

    #[test]
    fn test_header_struct_build_infoline_response() {
        let response = "HTTP/1.1 200 OK\r\n\
                        Host: localhost\r\n\
                        Content-Type: text/plain\r\n\
                        Content-Length: 12\r\n\r\n";
        let buf = BytesMut::from(response);
        let org = buf.as_ptr_range();
        let result = HeaderStruct::<Response>::new(buf).unwrap();
        assert_eq!(result.info_line.status(), b"200");
        let verify = result.into_data();
        assert_eq!(verify, response);
        assert_eq!(verify.as_ptr_range(), org);
    }
}
