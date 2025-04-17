use super::*;
use crate::abnf::{CRLF, HEADER_FS};

/* Steps:
 *      1. Convert str to BytesMut
 *      2. Extend key with ": "
 *      3. Extend value with CRLF
 *      4. Return Header
 */

impl From<(&str, &str)> for Header {
    fn from((key, value): (&str, &str)) -> Self {
        let mut bkey = BytesMut::from(key);
        bkey.extend_from_slice(HEADER_FS.as_bytes());
        let mut bvalue = BytesMut::from(value);
        bvalue.extend_from_slice(CRLF.as_bytes());
        Header {
            key: bkey,
            value: bvalue,
        }
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
}
