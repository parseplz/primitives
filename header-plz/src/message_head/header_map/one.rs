use bytes::{BufMut, Bytes, BytesMut};

use crate::{
    abnf::*,
    message_head::header_map::{Hmap, two::TwoHeader},
};

#[derive(Debug, PartialEq)]
pub struct OneHeader {
    key: BytesMut,   // key + ": "
    value: BytesMut, // value + "\r\n"
}

impl OneHeader {
    pub fn new(key: BytesMut, value: BytesMut) -> Self {
        OneHeader {
            key,
            value,
        }
    }

    pub fn into_bytes(mut self) -> BytesMut {
        self.key.unsplit(self.value);
        self.key
    }

    pub fn len(&self) -> usize {
        self.key.len() + self.value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.key.is_empty() && self.value.is_empty()
    }

    pub fn key_len(&self) -> usize {
        self.key.len()
    }

    pub fn value_len(&self) -> usize {
        self.value.len()
    }
}

impl From<(BytesMut, BytesMut)> for OneHeader {
    fn from((key, value): (BytesMut, BytesMut)) -> Self {
        OneHeader {
            key,
            value,
        }
    }
}

// (Content-Type, application/json)
impl From<(&str, &str)> for OneHeader {
    fn from((key, value): (&str, &str)) -> Self {
        let mut key = BytesMut::from(key);
        if !key.ends_with(HEADER_FS.as_bytes()) {
            key.extend_from_slice(HEADER_FS.as_bytes());
        }
        let mut value = BytesMut::from(value);
        if !value.ends_with(CRLF.as_bytes()) {
            value.extend_from_slice(CRLF.as_bytes());
        }
        OneHeader {
            key,
            value,
        }
    }
}

// Content-Type: application/json
impl From<&str> for OneHeader {
    fn from(input: &str) -> Self {
        let fs_index = find_header_fs(input.as_bytes());

        let (key, value) = if fs_index == 0 {
            // key
            (input, "")
        } else {
            // key: val
            input.split_at(fs_index)
        };

        OneHeader::from((key, value))
    }
}

/* Description:
 *      Contains atleast CRLF.
 */
impl From<BytesMut> for OneHeader {
    fn from(mut input: BytesMut) -> Self {
        let fs_index = find_header_fs(&input);

        // 2. If no ": " found, split at index 1 as atleast CRLF if present.
        let key = if fs_index == 0 {
            BytesMut::new()
        } else {
            input.split_to(fs_index)
        };
        OneHeader::from((key, input))
    }
}

// TODO: utf-8 check ?
impl From<OneHeader> for TwoHeader {
    fn from(mut one: OneHeader) -> Self {
        let key = if one.key_len() == 0 {
            Bytes::new()
        } else if let Some(fs_index) = one.key.iter().position(|b| b == &COLON)
        {
            if fs_index == 0 {
                Bytes::new()
            } else {
                one.key.split_to(fs_index).freeze()
            }
        } else {
            Bytes::new()
        };

        // if only CRLF or no value
        let value = if one.value_len() < 3 {
            Bytes::new()
        } else {
            one.value.split_to(one.value_len() - 2).freeze()
        };
        TwoHeader::from((key, value))
    }
}

impl Hmap for OneHeader {
    fn key_as_ref(&self) -> &[u8] {
        if let Some(pos) = self.key.iter().rposition(|b| b == &COLON) {
            &self.key[..pos]
        } else {
            &self.key
        }
    }

    fn value_as_ref(&self) -> &[u8] {
        if self.value.len() < 2 {
            &self.value
        } else {
            &self.value[..self.value.len() - 2]
        }
    }

    fn change_key(&mut self, key: &[u8]) {
        let ows = self.key.last().map(|b| *b == SP).unwrap_or(false);
        reuse_or_swap(key.len() + ows as usize + 1, &mut self.key, key);
        if ows {
            self.key.extend_from_slice(HEADER_FS.as_bytes());
        } else {
            self.key.put_u8(COLON);
        }
    }

    fn change_value(&mut self, value: &[u8]) {
        reuse_or_swap(value.len() + 2, &mut self.value, value);
        self.value.extend_from_slice(CRLF.as_bytes());
    }

    fn clear(&mut self) {
        self.key.clear();
        self.value.clear();
    }

    fn len(&self) -> usize {
        self.key.len() + self.value.len()
    }
}

fn reuse_or_swap(len: usize, target: &mut BytesMut, incoming: &[u8]) {
    if target.capacity() >= len {
        clear_and_write(target, incoming);
    } else {
        *target = BytesMut::from(incoming);
    }
}

fn clear_and_write(buf: &mut BytesMut, data: &[u8]) {
    buf.clear();
    buf.extend_from_slice(data);
}

pub fn find_header_fs(input: &[u8]) -> usize {
    if let Some(index) = input.iter().position(|b| b == &COLON) {
        // check if index + 1 == Space i.e. ": "
        if input.get(index + 1) == Some(&SP) {
            index + 2
        } else {
            index + 1
        }
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // from str
    #[test]
    fn test_one_header_from_tuple() {
        let key = "Content-Type";
        let value = "application/json";

        let header: OneHeader = (key, value).into();
        let expected = OneHeader {
            key: BytesMut::from("Content-Type: "),
            value: BytesMut::from("application/json\r\n"),
        };

        assert_eq!(header, expected);
    }

    #[test]
    fn test_one_header_from_str() {
        let input = "Content-Type: application/json\r\n";
        let header: OneHeader = (input).into();
        let expected = OneHeader {
            key: BytesMut::from("Content-Type: "),
            value: BytesMut::from("application/json\r\n"),
        };
        assert_eq!(header, expected);
    }

    #[test]
    fn test_one_header_from_str_key_only() {
        let input = "Content-Type";
        let header: OneHeader = (input).into();
        let expected = OneHeader {
            key: BytesMut::from("Content-Type: "),
            value: BytesMut::from(CRLF),
        };
        assert_eq!(header, expected);
    }

    #[test]
    fn test_one_header_from_str_no_crlf() {
        let input = "Content-Type: application/json";
        let header: OneHeader = (input).into();
        let expected = OneHeader {
            key: BytesMut::from("Content-Type: "),
            value: BytesMut::from("application/json\r\n"),
        };
        assert_eq!(header, expected);
    }

    #[test]
    fn test_one_header_as_ref() {
        let one = OneHeader::from(("key: ", "value\r\n"));
        assert_eq!(one.key_as_ref(), b"key");
        assert_eq!(one.value_as_ref(), b"value");
    }

    #[test]
    fn test_one_header_as_ref_less() {
        let one = OneHeader::from(("a", "b"));
        assert_eq!(one.key_as_ref(), b"a");
        assert_eq!(one.value_as_ref(), b"b");
    }

    // From Bytesmut
    #[test]
    fn test_one_header_from_bytesmut_basic() {
        let buf = BytesMut::from("content-type: application/json\r\n");
        let verify_ptr = buf.as_ptr_range();
        let header = OneHeader::from(buf);
        assert_eq!(&header.key[..], b"content-type: ");
        assert_eq!(&header.value[..], b"application/json\r\n");
        assert_eq!(header.key_as_ref(), b"content-type");
        assert_eq!(header.value_as_ref(), b"application/json");
        assert_eq!(verify_ptr, header.into_bytes().as_ptr_range());
    }

    #[test]
    fn test_one_header_from_bytesmut_no_space() {
        let buf = BytesMut::from("content-type:application/json\r\n");
        let verify_ptr = buf.as_ptr_range();
        let header = OneHeader::from(buf);
        assert_eq!(&header.key[..], b"content-type:");
        assert_eq!(&header.value[..], b"application/json\r\n");
        assert_eq!(header.key_as_ref(), b"content-type");
        assert_eq!(header.value_as_ref(), b"application/json");
        assert_eq!(verify_ptr, header.into_bytes().as_ptr_range());
    }

    #[test]
    fn test_one_header_from_bytesmut_fail_no_fs() {
        let buf = BytesMut::from("\r\n");
        let verify_ptr = buf.as_ptr_range();
        let header = OneHeader::from(buf);
        assert_eq!(&header.key[..], b"");
        assert_eq!(&header.value[..], b"\r\n");
        assert_eq!(header.key_as_ref(), b"");
        assert_eq!(header.value_as_ref(), b"");
        assert_eq!(verify_ptr, header.into_bytes().as_ptr_range());
    }

    #[test]
    fn test_one_header_from_bytesmut_len() {
        let buf: BytesMut = "content-type: application/json\r\n".into();
        let header = OneHeader::from(buf);
        assert_eq!(header.len(), 32);
    }

    #[test]
    fn test_one_to_two_perfect() {
        let buf: BytesMut = "content-type: application/json\r\n".into();
        let one = OneHeader::from(buf);
        let two = TwoHeader::from(one);
        assert_eq!(two.key_as_ref(), b"content-type");
        assert_eq!(two.value_as_ref(), b"application/json");
    }

    #[test]
    fn test_one_to_two_no_space() {
        let buf: BytesMut = "content-type:application/json\r\n".into();
        let one = OneHeader::from(buf);
        let two = TwoHeader::from(one);
        assert_eq!(two.key_as_ref(), b"content-type");
        assert_eq!(two.value_as_ref(), b"application/json");
    }

    #[test]
    fn test_one_to_two_empty_value() {
        let buf: BytesMut = "content-type:\r\n".into();
        let one = OneHeader::from(buf);
        let two = TwoHeader::from(one);
        assert_eq!(two.key_as_ref(), b"content-type");
        assert_eq!(two.value_as_ref(), b"");
    }

    // key change
    #[test]
    fn test_change_header_key_same_len() {
        let input = BytesMut::from("Content-Length: 10");
        let input_range = input.as_ptr_range();
        let mut header = OneHeader::from(input);
        header.change_key(b"content-length");
        let result = header.into_bytes();
        let result_range = result.as_ptr_range();
        assert_eq!(result, "content-length: 10");
        assert_eq!(input_range, result_range);
    }

    #[test]
    fn test_change_header_key_reduced_len() {
        let input = BytesMut::from("Content-Length: 10");
        let input_range = input.as_ptr_range();
        let mut header = OneHeader::from(input);
        header.change_key(b"a");
        let result = header.into_bytes();
        let result_range = result.as_ptr_range();
        assert_eq!(result, "a: 10");
        assert!(input_range.contains(&result_range.start));
        assert!(input_range.contains(&result_range.end));
    }

    #[test]
    fn test_change_header_key_large_len() {
        let input = BytesMut::from("Small: 10");
        let input_range = input.as_ptr_range();
        let mut header = OneHeader::from(input);
        header.change_key(b"VeryLargeHeader");
        let result = header.into_bytes();
        let result_range = result.as_ptr_range();
        assert_eq!(result, "VeryLargeHeader: 10");
        assert_ne!(input_range, result_range);
    }

    #[test]
    fn test_change_header_key_same_len_no_ows() {
        let input = BytesMut::from("Content-Length:10");
        let input_range = input.as_ptr_range();
        let mut header = OneHeader::from(input);
        header.change_key(b"content-length");
        let result = header.into_bytes();
        let result_range = result.as_ptr_range();
        assert_eq!(result, "content-length:10");
        assert_eq!(input_range, result_range);
    }

    // value change
    #[test]
    fn test_change_header_value_same_len() {
        let input = BytesMut::from("Content-Length: 10\r\n");
        let input_range = input.as_ptr_range();
        let mut header = OneHeader::from(input);
        header.change_value(b"20");
        let result = header.into_bytes();
        let result_range = result.as_ptr_range();
        assert_eq!(result, "Content-Length: 20\r\n");
        assert_eq!(input_range, result_range);
    }

    #[test]
    fn test_change_header_value_reduced_len() {
        let input = BytesMut::from("Content-Length: 1000\r\n");
        let input_range = input.as_ptr_range();
        let mut header = OneHeader::from(input);
        header.change_value(b"1");
        let result = header.into_bytes();
        let result_range = result.as_ptr_range();
        assert_eq!(result, "Content-Length: 1\r\n");
        assert!(input_range.contains(&result_range.start));
        assert!(input_range.contains(&result_range.end));
    }

    #[test]
    fn test_change_header_value_large_len() {
        let input = BytesMut::from("Small: 10\r\n");
        let input_range = input.as_ptr_range();
        let mut header = OneHeader::from(input);
        header.change_value(b"10000");
        let result = header.into_bytes();
        let result_range = result.as_ptr_range();
        assert_eq!(result, "Small: 10000\r\n");
        assert_ne!(input_range, result_range);
    }

    /* TODO: fix
    #[test]
    fn test_change_header_value_multiple() {
        let input = BytesMut::from("Content-Encoding: gzip\r\n");
        let input_range = input.as_ptr_range();
        let mut header = OneHeader::from(input);
        let ce = [
            ContentEncoding::Gzip,
            ContentEncoding::Deflate,
            ContentEncoding::Brotli,
        ];
        let iter = ce.iter().map(|s| s.as_ref());
        header.change_value_multiple(iter);
        let result = header.into_bytes();
        let result_range = result.as_ptr_range();
        assert_eq!(result, "Content-Encoding: gzip, deflate, br\r\n");
        assert_ne!(input_range, result_range);
    }
    */
}
