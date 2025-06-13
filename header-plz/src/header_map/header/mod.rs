use std::str::{self};

use bytes::BytesMut;

use crate::abnf::{COLON, CRLF, HEADER_FS};
mod from_bytes;
mod from_str;

// Struct for single Header
#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    key: BytesMut,   // Key + ": "
    value: BytesMut, // Value + "\r\n"
}

impl Header {
    pub fn new(key: BytesMut, value: BytesMut) -> Self {
        Header { key, value }
    }

    pub fn into_bytes(mut self) -> BytesMut {
        self.key.unsplit(self.value);
        self.key
    }

    pub fn change_key(&mut self, key: &str) {
        reuse_or_swap(key.len() + 2, &mut self.key, key);
        self.key.extend_from_slice(HEADER_FS.as_bytes());
    }

    pub fn change_value(&mut self, value: &str) {
        reuse_or_swap(value.len() + 2, &mut self.value, value);
        self.value.extend_from_slice(CRLF.as_bytes());
    }

    // new() method checked whether it is a valid str
    // safe to unwrap
    pub fn key_as_str(&self) -> &str {
        str::from_utf8(&self.key)
            .unwrap()
            .split(COLON)
            .nth(0)
            .unwrap()
    }

    pub fn value_as_str(&self) -> &str {
        str::from_utf8(&self.value)
            .unwrap()
            .split(CRLF)
            .nth(0)
            .unwrap()
    }

    pub fn len(&self) -> usize {
        self.key.len() + self.value.len()
    }

    pub fn key(&self) -> &BytesMut {
        &self.key
    }

    pub fn value(&self) -> &BytesMut {
        &self.value
    }

    pub fn key_as_mut(&mut self) -> &mut BytesMut {
        &mut self.key
    }

    pub fn value_as_mut(&mut self) -> &mut BytesMut {
        &mut self.value
    }

    pub fn split_header(header: &str) -> (&str, &str) {
        header
            .split_once(COLON)
            .map(|(k, v)| (k, v.trim()))
            .unwrap_or_default()
    }
}

fn reuse_or_swap(len: usize, target: &mut BytesMut, incoming: &str) {
    if target.capacity() >= len {
        clear_and_write(target, incoming.as_bytes());
    } else {
        *target = BytesMut::from(incoming.as_bytes());
    }
}

fn clear_and_write(buf: &mut BytesMut, data: &[u8]) {
    buf.clear();
    buf.extend_from_slice(data);
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::Header;

    // key change
    #[test]
    fn test_change_header_key_same_len() {
        let input = BytesMut::from("Content-Length: 10");
        let input_range = input.as_ptr_range();
        let mut header = Header::from(input);
        header.change_key("content-length");
        let result = header.into_bytes();
        let result_range = result.as_ptr_range();
        assert_eq!(result, "content-length: 10");
        assert_eq!(input_range, result_range);
    }

    #[test]
    fn test_change_header_key_reduced_len() {
        let input = BytesMut::from("Content-Length: 10");
        let input_range = input.as_ptr_range();
        let mut header = Header::from(input);
        header.change_key("a");
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
        let mut header = Header::from(input);
        header.change_key("VeryLargeHeader");
        let result = header.into_bytes();
        let result_range = result.as_ptr_range();
        assert_eq!(result, "VeryLargeHeader: 10");
        assert_ne!(input_range, result_range);
    }

    // value change
    #[test]
    fn test_change_header_value_same_len() {
        let input = BytesMut::from("Content-Length: 10\r\n");
        let input_range = input.as_ptr_range();
        let mut header = Header::from(input);
        header.change_value("20");
        let result = header.into_bytes();
        let result_range = result.as_ptr_range();
        assert_eq!(result, "Content-Length: 20\r\n");
        assert_eq!(input_range, result_range);
    }

    #[test]
    fn test_change_header_value_reduced_len() {
        let input = BytesMut::from("Content-Length: 1000\r\n");
        let input_range = input.as_ptr_range();
        let mut header = Header::from(input);
        header.change_value("1");
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
        let mut header = Header::from(input);
        header.change_value("10000");
        let result = header.into_bytes();
        let result_range = result.as_ptr_range();
        assert_eq!(result, "Small: 10000\r\n");
        assert_ne!(input_range, result_range);
    }
}
