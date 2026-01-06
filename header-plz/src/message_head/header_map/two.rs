use bytes::{Bytes, BytesMut};

use crate::{
    abnf::*,
    message_head::header_map::{Hmap, one::OneHeader, split_header},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Header {
    key: Bytes,
    value: Bytes,
    is_removed: bool,
}

impl Header {
    pub fn new(key: Bytes, value: Bytes) -> Self {
        Header {
            key,
            value,
            is_removed: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.key.is_empty() && self.value.is_empty() && self.is_removed
    }
}

impl Hmap for Header {
    fn key_as_ref(&self) -> &[u8] {
        &self.key
    }

    fn value_as_ref(&self) -> &[u8] {
        &self.value
    }

    fn change_key(&mut self, key: &[u8]) {
        self.key = Bytes::from(key.to_owned());
    }

    fn change_value(&mut self, value: &[u8]) {
        self.value = Bytes::from(value.to_owned());
    }

    fn clear(&mut self) {
        self.key.clear();
        self.value.clear();
        self.is_removed = true;
    }

    fn len(&self) -> usize {
        self.key.len() + self.value.len()
    }

    fn truncate_value(&mut self, pos: usize) {
        self.value.truncate(pos)
    }
}

impl From<(Bytes, Bytes)> for Header {
    fn from((key, value): (Bytes, Bytes)) -> Self {
        Header {
            key,
            value,
            is_removed: false,
        }
    }
}

impl From<(&str, &str)> for Header {
    fn from((key, value): (&str, &str)) -> Self {
        let key = Bytes::from(key.to_owned());
        let value = Bytes::from(value.to_owned());
        Header {
            key,
            value,
            is_removed: false,
        }
    }
}

impl From<&str> for Header {
    fn from(hdr: &str) -> Self {
        let (key, val) = split_header(hdr);
        Header {
            key: Bytes::copy_from_slice(key.as_bytes()),
            value: Bytes::copy_from_slice(val.as_bytes()),
            is_removed: false,
        }
    }
}

impl From<Header> for OneHeader {
    fn from(two: Header) -> OneHeader {
        let mut key = BytesMut::from(two.key);
        key.extend_from_slice(HEADER_FS.as_bytes());
        let mut value = BytesMut::from(two.value);
        value.extend_from_slice(CRLF.as_bytes());
        OneHeader::from((key, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // from

    #[test]
    fn test_two_header_from_tuple() {
        let key = "Content-Type";
        let value = "application/json";

        let _header: Header = (key, value).into();
        let _expected = Header {
            key: Bytes::from(key.to_owned()),
            value: Bytes::from(value.to_owned()),
            is_removed: false,
        };
    }

    #[test]
    fn test_two_header_from_str() {
        let input = "Content-Type: application/json\r\n";
        let header: Header = (input).into();
        let expected = Header {
            key: Bytes::from("Content-Type".to_owned()),
            value: Bytes::from("application/json".to_owned()),
            is_removed: false,
        };
        assert_eq!(header, expected);
    }

    #[test]
    fn test_two_header_from_str_key_only() {
        let input = "Content-Type:";
        let header: Header = (input).into();
        let expected = Header {
            key: Bytes::from("Content-Type".to_owned()),
            value: Bytes::from("".to_owned()),
            is_removed: false,
        };
        assert_eq!(header, expected);
    }

    #[test]
    fn test_two_header_as_ref() {
        let two = Header::from(("key", "value"));
        assert_eq!(two.key_as_ref(), b"key");
        assert_eq!(two.value_as_ref(), b"value");
    }

    #[test]
    fn test_two_header_len() {
        let buf = "content-type: application/json\r\n";
        let header = Header::from(buf);
        assert_eq!(header.len(), 28);
    }

    #[test]
    fn test_two_to_one_perfect() {
        let two = Header::from(("content-type", "application/json"));
        let one = OneHeader::from(two);
        assert_eq!(one.key_as_ref(), b"content-type");
        assert_eq!(one.value_as_ref(), b"application/json");

        let verify = "content-type: application/json\r\n";
        assert_eq!(one.into_bytes(), verify);

        let buf: BytesMut = "content-type: application/json\r\n".into();
        let one = OneHeader::from(buf);
        let two = Header::from(one);
        assert_eq!(two.key_as_ref(), b"content-type");
        assert_eq!(two.value_as_ref(), b"application/json");
    }

    #[test]
    fn test_two_to_one_no_key() {
        let two = Header::from(("", "application/json"));
        let one = OneHeader::from(two);
        assert_eq!(one.key_as_ref(), b"");
        assert_eq!(one.value_as_ref(), b"application/json");
        let verify = ": application/json\r\n";
        assert_eq!(one.into_bytes(), verify);
    }

    #[test]
    fn test_two_to_one_no_value() {
        let two = Header::from(("content-type", ""));
        let one = OneHeader::from(two);
        assert_eq!(one.key_as_ref(), b"content-type");
        assert_eq!(one.value_as_ref(), b"");
        let verify = "content-type: \r\n";
        assert_eq!(one.into_bytes(), verify);
    }

    #[test]
    fn test_truncate_value() {
        let mut input = Header::from(("key", "hola, que, tal"));
        input.truncate_value(9);
        let verify = Header::from(("key", "hola, que"));
        assert_eq!(input, verify);
    }
}
