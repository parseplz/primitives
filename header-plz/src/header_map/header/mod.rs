use std::str::{self};

use bytes::BytesMut;

use crate::abnf::{CRLF, HEADER_FS};
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
