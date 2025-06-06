pub mod header;
use std::str;

use bytes::BytesMut;
use header::*;
mod from_bytes;

use crate::abnf::{CRLF, HEADER_FS};

// Vec<Header> + CRLF
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq))]
#[derive(Default)]
pub struct HeaderMap {
    headers: Vec<Header>,
    crlf: BytesMut, // Final Crlf
}

impl HeaderMap {
    pub fn new(headers: Vec<Header>, crlf: BytesMut) -> Self {
        HeaderMap { headers, crlf }
    }

    pub fn into_bytes(mut self) -> BytesMut {
        for header in self.headers.into_iter().rev() {
            let mut data = header.into_bytes();
            data.unsplit(self.crlf);
            self.crlf = data;
        }
        self.crlf
    }

    pub fn headers(&self) -> &Vec<Header> {
        &self.headers
    }

    pub fn headers_as_mut(&mut self) -> &mut Vec<Header> {
        &mut self.headers
    }

    pub fn into_header_vec(self) -> Vec<Header> {
        self.headers
    }

    // Entire header
    pub fn change_header(&mut self, old: Header, new: Header) -> bool {
        for (index, header) in self.headers.iter().enumerate() {
            if *header == old {
                self.headers[index] = new;
                return true;
            }
        }
        false
    }

    pub fn remove_header(&mut self, toremove: Header) -> bool {
        for (index, h) in self.headers.iter_mut().enumerate() {
            if *h == toremove {
                self.headers.remove(index);
                return true;
            }
        }
        false
    }

    pub fn add_header(&mut self, header: Header) {
        self.headers.push(header);
    }

    pub fn change_header_on_key(&mut self, key: &str, new_header: Header) -> bool {
        for h in self.headers.iter_mut() {
            if h.key_as_str().eq_ignore_ascii_case(key) {
                *h = new_header;
                return true;
            }
        }
        false
    }

    pub fn remove_header_on_pos(&mut self, pos: usize) {
        self.headers.remove(pos);
    }

    // Header Key
    pub fn has_header_key(&self, key: &str) -> Option<usize> {
        self.headers
            .iter()
            .position(|header| header.key_as_str().eq_ignore_ascii_case(key))
    }

    pub fn change_header_key(&mut self, old_key: &str, new_key: &str) -> bool {
        for h in self.headers.iter_mut() {
            if h.key_as_str().eq_ignore_ascii_case(old_key) {
                let mut a = new_key.to_string();
                a.push_str(HEADER_FS);
                h.change_key(a.as_bytes().into());
                return true;
            }
        }
        false
    }

    pub fn remove_header_on_key(&mut self, key: &str) -> bool {
        for (index, h) in self.headers.iter().enumerate() {
            if h.key_as_str().eq_ignore_ascii_case(key) {
                self.headers.remove(index);
                return true;
            }
        }
        false
    }

    // Value
    pub fn change_header_value_on_key(&mut self, key: &str, value: &str) -> bool {
        for h in self.headers.iter_mut() {
            if h.key_as_str().eq_ignore_ascii_case(key) {
                let mut a = value.to_string();
                a.push_str(CRLF);
                h.change_value(a.as_bytes().into());
                return true;
            }
        }
        false
    }

    pub fn change_header_value_on_pos(&mut self, pos: usize, value: &str) {
        let mut buf = BytesMut::with_capacity(value.len() + 2);
        buf.extend_from_slice(value.as_bytes());
        buf.extend_from_slice(CRLF.as_bytes());
        self.headers[pos].change_value(buf);
    }

    pub fn value_for_key(&self, key: &str) -> Option<&str> {
        for header in self.headers.iter() {
            if header.key_as_str().eq_ignore_ascii_case(key) {
                return Some(header.value_as_str());
            }
        }
        None
    }

    // common
    pub fn has_key_and_value(&self, key: &str, value: &str) -> Option<usize> {
        self.headers.iter().position(|header| {
            header.key_as_str().eq_ignore_ascii_case(key)
                && header.value_as_str().eq_ignore_ascii_case(value)
        })
    }

    pub fn len(&self) -> usize {
        self.headers
            .iter()
            .fold(0, |total, entry| total + entry.len())
            + 2
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_header_map_has_header_key() {
        let raw_header: BytesMut = "Content-Length: 20\r\n\r\n".into();
        let map = HeaderMap::from(raw_header);
        let key = "Content-Length";
        let result = map.has_header_key(key);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_header_map_change_header() {
        let raw_header: BytesMut = "Content-Length: 20\r\n\r\n".into();
        let mut map = HeaderMap::from(raw_header);
        let old = ("Content-Length", "20").into();
        let new = ("Content-Type", "application/json").into();
        let result = map.change_header(old, new);
        assert!(result);
        let val = map.into_bytes();
        let verify = "Content-Type: application/json\r\n\r\n";
        assert_eq!(val, verify);
    }

    #[test]
    fn test_header_map_remove_header_first() {
        let raw_header: BytesMut = "Content-Type: application/json\r\n\
                          Content-Length: 20\r\n\r\n"
            .into();
        let mut map = HeaderMap::from(raw_header);
        let to_remove = ("Content-Length", "20").into();
        let result = map.remove_header(to_remove);
        assert!(result);
        let val = map.into_bytes();
        let verify = "Content-Type: application/json\r\n\r\n";
        assert_eq!(val, verify);
    }

    #[test]
    fn test_header_map_remove_header_second() {
        let raw_header: BytesMut = "Content-Type: application/json\r\n\
                          Content-Length: 20\r\n\r\n"
            .into();
        let mut map = HeaderMap::from(raw_header);
        let to_remove = ("Content-Type", "application/json").into();
        let result = map.remove_header(to_remove);
        assert!(result);
        let val = map.into_bytes();
        let verify = "Content-Length: 20\r\n\r\n";
        assert_eq!(val, verify);
    }

    #[test]
    fn test_header_map_change_header_key() {
        let raw_header: BytesMut = "Content-Length: 20\r\n\r\n".into();
        let mut map = HeaderMap::from(raw_header);
        let old = "Content-Length";
        let new = "Content-Type";
        let result = map.change_header_key(old, new);
        assert!(result);
        let val = map.into_bytes();
        let verify = "Content-Type: 20\r\n\r\n";
        assert_eq!(val, verify);
    }

    #[test]
    fn test_header_map_change_header_value() {
        let raw_header: BytesMut = "Content-Length: 20\r\n\r\n".into();
        let mut map = HeaderMap::from(raw_header);
        let key = "Content-Length";
        let new_val = "30";
        let result = map.change_header_value_on_key(key, new_val);
        assert!(result);
        let val = map.into_bytes();
        let verify = "Content-Length: 30\r\n\r\n";
        assert_eq!(val, verify);
    }

    #[test]
    fn test_header_map_remove_header_on_key() {
        let raw_header: BytesMut = "Content-Length: 20\r\n\r\n".into();
        let mut map = HeaderMap::from(raw_header);
        let key = "Content-Length";
        let result = map.remove_header_on_key(key);
        assert!(result);
        let val = map.into_bytes();
        let verify = "\r\n";
        assert_eq!(val, verify);
    }

    #[test]
    fn test_header_map_value_for_key() {
        let data = "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/png,image/svg+xml,*/*;q=0.8\r\n\r\n";

        let buf = BytesMut::from(data);
        let header_map = HeaderMap::from(buf);
        let result = header_map.value_for_key("Accept");
        let verify = Some(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/png,image/svg+xml,*/*;q=0.8",
        );
        assert_eq!(result, verify);
    }

    #[test]
    fn test_change_header_value_on_pos() {
        let raw_header: BytesMut = "Content-Length: 20\r\n\r\n".into();
        let mut map = HeaderMap::from(raw_header);
        let pos = 0;
        let new_val = "30";
        map.change_header_value_on_pos(pos, new_val);
        let val = map.into_bytes();
        let verify = "Content-Length: 30\r\n\r\n";
        assert_eq!(val, verify);
    }

    #[test]
    fn test_header_map_len_small() {
        let data = "a: b\r\n\r\n";
        let buf = BytesMut::from(data);
        let header_map = HeaderMap::from(buf);
        assert_eq!(header_map.len(), 8);
    }

    #[test]
    fn test_header_map_len_large() {
        let data = "content-type: application/json\r\n\
                    transfer-encoding: chunked\r\n\
                    content-encoding: gzip\r\n\
                    trailer: Some\r\n\
                    x-custom-header: somevalue\r\n\r\n";
        let buf = BytesMut::from(data);
        let header_map = HeaderMap::from(buf);
        // 32 + 28 + 24 + 15 + 28 + 2
        assert_eq!(header_map.len(), 129);
    }
}
