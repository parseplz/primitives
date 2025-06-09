use super::*;
use crate::abnf::{CRLF, HEADER_FS};

/* Steps:
 *      1. Convert str to BytesMut
 *      2. Extend key with ": "
 *      3. Extend value with CRLF
 *      4. Return Header
 */

// (Content-Type, application/json)
impl From<(&str, &str)> for Header {
    fn from((key, value): (&str, &str)) -> Self {
        let mut bkey = BytesMut::from(key);
        if !bkey.ends_with(HEADER_FS.as_bytes()) {
            bkey.extend_from_slice(HEADER_FS.as_bytes());
        }
        let mut bvalue = BytesMut::from(value);
        if !bvalue.ends_with(CRLF.as_bytes()) {
            bvalue.extend_from_slice(CRLF.as_bytes());
        }
        Header::new(bkey, bvalue)
    }
}

// Content-Type: application/json
impl From<&str> for Header {
    fn from(input: &str) -> Self {
        let fs_index = input.find(HEADER_FS);

        // key: val\r\n
        let (key, val) = if let Some(index) = fs_index {
            input.split_at(index + 2)
            // key\r\n
        } else if input.ends_with(CRLF) {
            input.split_at(input.len() - 2)
            // key
        } else {
            (input, CRLF)
        };

        Header::from((key, val))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header_from_tuple() {
        let key = "Content-Type";
        let value = "application/json";

        let header: Header = (key, value).into();
        let expected = Header {
            key: BytesMut::from("Content-Type: "),
            value: BytesMut::from("application/json\r\n"),
        };

        assert_eq!(header, expected);
    }

    #[test]
    fn test_header_from_str() {
        let input = "Content-Type: application/json\r\n";
        let header: Header = (input).into();
        let expected = Header {
            key: BytesMut::from("Content-Type: "),
            value: BytesMut::from("application/json\r\n"),
        };
        assert_eq!(header, expected);
    }

    #[test]
    fn test_header_from_str_key_only() {
        let input = "Content-Type";
        let header: Header = (input).into();
        let expected = Header {
            key: BytesMut::from("Content-Type: "),
            value: BytesMut::from(CRLF),
        };
        assert_eq!(header, expected);
    }

    #[test]
    fn test_header_from_str_no_crlf() {
        let input = "Content-Type: application/json";
        let header: Header = (input).into();
        let expected = Header {
            key: BytesMut::from("Content-Type: "),
            value: BytesMut::from("application/json\r\n"),
        };
        assert_eq!(header, expected);
    }
}
