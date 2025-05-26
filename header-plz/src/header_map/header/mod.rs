use std::str::{self};

use bytes::BytesMut;

use crate::abnf::{CRLF, HEADER_FS};
mod from_str;

// Struct for single Header
#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    key: BytesMut,   // Key + ": "
    value: BytesMut, // Value + "\r\n"
}

impl Header {
    /* Description:
     *      Associated method to build Header.
     *      Contains atleast CRLF.
     *
     * Steps:
     *      1. Find ": " index.
     *      2. If no ": " found, split at index 1 as atleast CRLF if
     *         present.
     *      2. Split to key and value.
     *
     */

    pub fn new(mut input: BytesMut) -> Self {
        // utf8 already checked in HeaderMap::new()
        // safe to unwrap
        let data = str::from_utf8(&input).unwrap();
        let fs_index = data.find(HEADER_FS).unwrap_or(0);

        // 2. If no ": " found, split at index 1 as atleast CRLF if present.
        let key = if fs_index == 0 {
            input.split_to(1)
        } else {
            input.split_to(fs_index + 2)
        };
        Header { key, value: input }
    }

    pub fn into_data(mut self) -> BytesMut {
        self.key.unsplit(self.value);
        self.key
    }

    pub fn change_key(&mut self, key: BytesMut) {
        self.key = key
    }

    pub fn change_value(&mut self, value: BytesMut) {
        self.value = value
    }

    // new() method checked whether it is a valid str
    // safe to unwrap
    pub fn key_as_str(&self) -> &str {
        str::from_utf8(&self.key)
            .unwrap()
            .split(HEADER_FS)
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header_basic() {
        let buf = BytesMut::from("content-type: application/json\r\n");
        let verify_ptr = buf.as_ptr_range();
        let header = Header::new(buf);
        assert_eq!(header.key_as_str(), "content-type");
        assert_eq!(header.value_as_str(), "application/json");
        assert_eq!(verify_ptr, header.into_data().as_ptr_range());
    }

    #[test]
    fn test_header_fail_no_fs() {
        let buf = BytesMut::from("\r\n");
        let header = Header::new(buf);
        assert_eq!(header.key_as_str(), "\r");
        assert_eq!(header.value_as_str(), "\n");
    }

    #[test]
    fn test_header_len() {
        let buf: BytesMut = "content-type: application/json\r\n".into();
        let header = Header::new(buf);
        assert_eq!(header.len(), 32);
    }
}
