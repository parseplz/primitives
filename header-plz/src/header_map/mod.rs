pub mod header;
use std::str;

use bytes::BytesMut;
use header::*;

use crate::abnf::{COLON, CRLF};

mod from_bytes;

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
    pub fn find_header_pos(&self, to_find: &str) -> Option<usize> {
        let (key, val) = to_find
            .split_once(COLON)
            .map(|(k, v)| (k, v.trim()))
            .unwrap_or_default();
        self.headers
            .iter()
            .position(|h| h.key_as_str() == key && h.value_as_str() == val)
    }

    // old : Content-Length: 20
    // new : Content-Length: 10
    pub fn change_header(&mut self, old: &str, new: &str) -> bool {
        if let Some(index) = self.find_header_pos(old) {
            let (new_key, new_val) = new
                .split_once(COLON)
                .map(|(k, v)| (k, v.trim()))
                .unwrap_or_default();
            self.headers[index].change_key(new_key);
            self.headers[index].change_value(new_val);
            return true;
        }
        false
    }

    pub fn remove_header(&mut self, to_remove: &str) -> bool {
        if let Some(index) = self.find_header_pos(to_remove) {
            self.headers.remove(index);
            return true;
        }
        false
    }

    pub fn add_header(&mut self, header: Header) {
        self.headers.push(header);
    }

    pub fn remove_header_on_pos(&mut self, pos: usize) {
        self.headers.remove(pos);
    }

    // Key
    pub fn has_header_key(&self, key: &str) -> Option<usize> {
        self.headers
            .iter()
            .position(|header| header.key_as_str().eq_ignore_ascii_case(key))
    }

    pub fn change_header_key(&mut self, old_key: &str, new_key: &str) -> bool {
        for h in self.headers.iter_mut() {
            if h.key_as_str().eq_ignore_ascii_case(old_key) {
                h.change_key(new_key);
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
                h.change_value(value);
                return true;
            }
        }
        false
    }

    pub fn change_header_value_on_pos(&mut self, pos: usize, value: &str) {
        self.headers[pos].change_value(value);
    }

    pub fn value_for_key(&self, key: &str) -> Option<&str> {
        for header in self.headers.iter() {
            if header.key_as_str().eq_ignore_ascii_case(key) {
                return Some(header.value_as_str());
            }
        }
        None
    }

    // general
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

    pub fn truncate_header_values<T, E>(&mut self, key: &str, remove: E)
    where
        T: AsRef<str>,
        E: IntoIterator<Item = T>,
    {
        let Some(pos) = self.has_header_key(key) else {
            return;
        };
        let value = self.headers[pos].value_as_mut();
        // Remove CRLF
        value.truncate(value.len() - 2);

        for e in remove.into_iter() {
            let mut to_reduce = value.len().saturating_sub(AsRef::<str>::as_ref(&e).len());
            while to_reduce > 0 {
                let b = value[to_reduce - 1];
                // If it's in a-z, A-Z, 0-9 => stop trimming
                if b.is_ascii_alphanumeric() {
                    break;
                }
                to_reduce -= 1;
            }
            value.truncate(to_reduce);
        }
        // Add CRLF
        value.extend_from_slice(CRLF.as_bytes());
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        body_headers::content_encoding::ContentEncoding,
        const_headers::{CONTENT_ENCODING, TRANSFER_ENCODING},
    };

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
        let input: BytesMut = "Content-Length: 20\r\n\r\n".into();
        let input_range = input.as_ptr_range();
        let mut map = HeaderMap::from(input);
        let old = "Content-Length: 20";
        let new = "Content-Length: 10";
        let result = map.change_header(old, new);
        assert!(result);
        let val = map.into_bytes();
        let verify = "Content-Length: 10\r\n\r\n";
        assert_eq!(val, verify);
        let result_range = val.as_ptr_range();
        assert_eq!(input_range, result_range);
    }

    #[test]
    fn test_header_map_remove_header_first() {
        let raw_header: BytesMut = "Content-Type: application/json\r\n\
                          Content-Length: 20\r\n\r\n"
            .into();
        let mut map = HeaderMap::from(raw_header);
        let to_remove = "Content-Length: 20";
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
        let to_remove = "Content-Type: application/json";
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

    #[test]
    fn test_header_map_truncate_header_values() {
        let data = "Header: a,  b,c\r\n\r\n";
        let buf = BytesMut::from(data);
        let mut header_map = HeaderMap::from(buf);
        let to_remove = "c";
        header_map.truncate_header_values("Header", [to_remove].iter());
        let result = header_map.into_bytes();
        assert_eq!(result, "Header: a,  b\r\n\r\n");

        let mut header_map = HeaderMap::from(result);
        let to_remove = "b";
        header_map.truncate_header_values("Header", [to_remove].iter());
        let result = header_map.into_bytes();
        assert_eq!(result, "Header: a\r\n\r\n");
    }

    #[test]
    fn test_header_map_truncate_header_values_ce() {
        let data = "Content-Encoding: gzip, deflate, br\r\n\r\n";
        let buf = BytesMut::from(data);
        let mut header_map = HeaderMap::from(buf);
        let applied_ce = [ContentEncoding::Deflate, ContentEncoding::Brotli];
        header_map.truncate_header_values(CONTENT_ENCODING, applied_ce.iter().rev());
        let result = header_map.into_bytes();
        assert_eq!(result, "Content-Encoding: gzip\r\n\r\n");
    }

    #[test]
    fn test_header_map_truncate_header_values_te() {
        let data = "Transfer-Encoding: gzip, deflate, br\r\n\r\n";
        let buf = BytesMut::from(data);
        let mut header_map = HeaderMap::from(buf);
        let applied_ce = [ContentEncoding::Deflate, ContentEncoding::Brotli];
        header_map.truncate_header_values(TRANSFER_ENCODING, applied_ce.iter().rev());
        let result = header_map.into_bytes();
        assert_eq!(result, "Transfer-Encoding: gzip\r\n\r\n");
    }
}
