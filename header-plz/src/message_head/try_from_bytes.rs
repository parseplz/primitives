use super::MessageHead;
use crate::{HeaderMap, error::HeaderReadError, message_head::info_line::InfoLine};
use bytes::BytesMut;

/* Steps:
 *      1. Find CR in buf.
 *      2. Split buf at CR_index + 2 (CRLF)
 *      3. Build Infoline
 *
 * Error:
 *      HttpReadError::InfoLine       [3]
 *      HttpReadError::HeaderStruct   [Default]
 */

impl<T> TryFrom<BytesMut> for MessageHead<T>
where
    T: InfoLine,
{
    type Error = HeaderReadError;

    fn try_from(mut data: BytesMut) -> Result<Self, HeaderReadError> {
        if let Some(infoline_index) = data.iter().position(|&x| x == 13) {
            let raw = data.split_to(infoline_index + 2);
            let info_line = T::try_build_infoline(raw)?;
            return Ok(Self {
                info_line,
                header_map: HeaderMap::from(data),
            });
        }
        Err(HeaderReadError::HeaderStruct(
            String::from_utf8_lossy(&data).to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {

    use crate::{Request, Response};

    use super::*;

    #[test]
    fn test_message_head_request_try_from() {
        let input = "GET / HTTP/1.1\r\n\
                       Host: localhost\r\n\
                       Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
                       Accept-Language: en-US,en;q=0.5\r\n\
                       Accept-Encoding: gzip, deflate\r\n\
                       User-Agent: curl/7.29.0\r\n\
                       Connection: keep-alive\r\n\r\n";
        let buf = BytesMut::from(input);
        let org = buf.as_ptr_range();
        let result = MessageHead::<Request>::try_from(buf).unwrap();
        assert_eq!(result.info_line.method(), b"GET");
        assert_eq!(result.info_line.uri_as_string(), "/");
        let verify = result.into_bytes();
        assert_eq!(verify, input);
        assert_eq!(verify.as_ptr_range(), org);
    }

    #[test]
    fn test_message_head_response_try_from() {
        let input = "HTTP/1.1 200 OK\r\n\
                        Host: localhost\r\n\
                        Content-Type: text/plain\r\n\
                        Content-Length: 12\r\n\r\n";
        let buf = BytesMut::from(input);
        let org = buf.as_ptr_range();
        let result = MessageHead::<Response>::try_from(buf).unwrap();
        assert_eq!(result.info_line.status(), b"200");
        let verify = result.into_bytes();
        assert_eq!(verify, input);
        assert_eq!(verify.as_ptr_range(), org);
    }
}
