use super::MessageHead;
use crate::{
    OneMessageHead,
    message_head::{
        OneHeaderMap, error::MessageHeadError, info_line::one::InfoLine,
    },
};
use bytes::BytesMut;

impl<T> TryFrom<BytesMut> for OneMessageHead<T>
where
    T: InfoLine,
{
    type Error = MessageHeadError;

    fn try_from(mut input: BytesMut) -> Result<Self, MessageHeadError> {
        if let Some(infoline_index) = input.iter().position(|&x| x == 13) {
            let crlf = input.split_off(input.len() - 2);
            let info_line_buf = input.split_to(infoline_index + 2);
            match T::try_build_infoline(info_line_buf) {
                Ok(info_line) => Ok(MessageHead::new(
                    info_line,
                    OneHeaderMap::from(input),
                    crlf,
                )),
                Err(mut e) => {
                    input.unsplit(crlf);
                    e.bytes_mut().unsplit(input);
                    Err(e.into())
                }
            }
        } else {
            Err(MessageHeadError::NoInfoLine(input))
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{
        OneRequestLine, OneResponseLine,
        message_head::info_line::one::error::{
            InfoLineError, InfoLineErrorKind,
        },
    };

    use super::*;

    #[test]
    fn test_message_head_request_try_from() {
        let input = "GET / HTTP/1.1\r\n\
                       Host: localhost\r\n\
                       Accept: text/html\r\n\
                       Accept-Language: en-US,en;q=0.5\r\n\
                       Accept-Encoding: gzip, deflate\r\n\
                       User-Agent: curl/7.29.0\r\n\
                       Connection: keep-alive\r\n\r\n";
        let buf = BytesMut::from(input);
        let org = buf.as_ptr_range();
        let result = OneMessageHead::<OneRequestLine>::try_from(buf).unwrap();
        assert_eq!(result.info_line.method_bytes(), b"GET");
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
        let result = OneMessageHead::<OneResponseLine>::try_from(buf).unwrap();
        assert_eq!(result.info_line.status().unwrap(), 200);
        let verify = result.into_bytes();
        assert_eq!(verify, input);
        assert_eq!(verify.as_ptr_range(), org);
    }

    #[test]
    fn test_message_head_error_no_info_line() {
        let input = "This is not a valid message";
        let buf = BytesMut::from(input);
        let result = OneMessageHead::<OneRequestLine>::try_from(buf);
        assert_eq!(result, Err(MessageHeadError::NoInfoLine(input.into())));
    }

    #[rstest]
    #[case("GET\r\n\r\n", InfoLineErrorKind::FirstOws)]
    #[case("GET /testHTTP/1.1\r\na: b\r\n\r\n", InfoLineErrorKind::SecondOws)]
    fn test_message_head_error_info_line(
        #[case] input: &str,
        #[case] err_kind: InfoLineErrorKind,
    ) {
        let buf = BytesMut::from(input);
        let result = OneMessageHead::<OneRequestLine>::try_from(buf);
        if let Err(MessageHeadError::ParseInfoLine(InfoLineError {
            bytes,
            error,
        })) = result
        {
            assert_eq!(bytes, input);
            assert_eq!(error, err_kind);
        } else {
            panic!()
        }
    }
}
