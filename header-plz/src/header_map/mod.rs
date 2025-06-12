pub mod header;
use std::str::{self};

use bytes::BytesMut;
use header::*;

use crate::abnf::{COLON, COMMA, CRLF, SP};

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
    pub fn header_position(&self, to_find_hdr: &str) -> Option<usize> {
        let (key, val) = to_find_hdr
            .split_once(COLON)
            .map(|(k, v)| (k, v.trim()))
            .unwrap_or_default();
        self.headers
            .iter()
            .position(|h| h.key_as_str() == key && h.value_as_str() == val)
    }

    pub fn header_position_all(&self, to_find_hdr: &str) -> Option<Vec<usize>> {
        let (key, val) = to_find_hdr
            .split_once(COLON)
            .map(|(k, v)| (k, v.trim()))
            .unwrap_or_default();
        let pos: Vec<usize> = self
            .headers
            .iter()
            .enumerate()
            .filter_map(|(i, h)| {
                if h.key_as_str() == key && h.value_as_str() == val {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        Some(pos).filter(|v| !v.is_empty())
    }

    // old : Content-Length: 20
    // new : Content-Length: 10
    pub fn change_header(&mut self, old: &str, new: &str) -> bool {
        let mut result = false;
        if let Some(position) = self.header_position(old) {
            result = true;
            let (new_key, new_val) = new
                .split_once(COLON)
                .map(|(k, v)| (k, v.trim()))
                .unwrap_or_default();
            self.headers[position].change_key(new_key);
            self.headers[position].change_value(new_val);
        }
        result
    }

    pub fn change_header_all(&mut self, old: &str, new: &str) -> bool {
        let mut result = false;
        if let Some(positions) = self.header_position_all(old) {
            result = true;
            let (new_key, new_val) = new
                .split_once(COLON)
                .map(|(k, v)| (k, v.trim()))
                .unwrap_or_default();
            for index in positions {
                self.headers[index].change_key(new_key);
                self.headers[index].change_value(new_val);
            }
        }
        result
    }

    // Content-Length: 10
    pub fn remove_header_all(&mut self, to_remove: &str) -> bool {
        let mut result = false;
        if let Some(positions) = self.header_position_all(to_remove) {
            result = true;
            for index in positions.into_iter().rev() {
                self.headers.remove(index);
            }
        }
        result
    }

    pub fn remove_header(&mut self, to_remove: &str) -> bool {
        let mut result = false;
        if let Some(position) = self.header_position(to_remove) {
            result = true;
            self.headers.remove(position);
        }
        result
    }

    pub fn add_header(&mut self, header: Header) {
        self.headers.push(header);
    }

    pub fn remove_header_on_position(&mut self, pos: usize) {
        self.headers.remove(pos);
    }

    // Key
    pub fn header_key_position(&self, key: &str) -> Option<usize> {
        self.headers
            .iter()
            .position(|header| header.key_as_str().eq_ignore_ascii_case(key))
    }

    pub fn header_key_position_all(&self, key: &str) -> Option<Vec<usize>> {
        let pos: Vec<usize> = self
            .headers
            .iter()
            .enumerate()
            .filter_map(|(i, h)| if h.key_as_str() == key { Some(i) } else { None })
            .collect();
        Some(pos).filter(|v| !v.is_empty())
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

    pub fn change_header_key_all(&mut self, old_key: &str, new_key: &str) -> bool {
        let mut result = false;
        if let Some(positions) = self.header_key_position_all(old_key) {
            result = true;
            for index in positions {
                self.headers[index].change_key(new_key);
            }
        }
        result
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

    pub fn remove_header_on_key_all(&mut self, key: &str) -> bool {
        let mut result = false;
        if let Some(positions) = self.header_key_position_all(key) {
            result = true;
            for index in positions.into_iter().rev() {
                self.headers.remove(index);
            }
        }
        result
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

    pub fn truncate_header_values<T, E>(&mut self, key: &str, remove: E)
    where
        T: AsRef<str>,
        E: IntoIterator<Item = T>,
    {
        let Some(pos) = self.header_key_position(key) else {
            return;
        };

        let value = self.headers[pos].value_as_str();
        let mut index = value.len();

        for e in remove.into_iter() {
            if let Some(curr) = value.find(e.as_ref()) {
                index = index.min(curr);
            }
        }

        if index == 0 {
            return;
        }

        loop {
            let mut chars = value.chars();
            match chars.nth(index.saturating_sub(1)).unwrap() {
                SP | COMMA => index = index.saturating_sub(1),
                _ => break,
            };
        }

        self.headers[pos].value_as_mut().truncate(index);
        self.headers[pos]
            .value_as_mut()
            .extend_from_slice(CRLF.as_bytes());
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

    use crate::{
        body_headers::content_encoding::ContentEncoding,
        const_headers::{CONTENT_ENCODING, TRANSFER_ENCODING},
    };

    use super::*;

    //..........Entire Header
    // find_header_pos
    #[test]
    fn test_header_map_find_header_pos_single() {
        let input = "Content-Length: 20\r\n\r\n";
        let map = HeaderMap::from(BytesMut::from(input));
        let key = "Content-Length: 20";
        let result = map.header_position_all(key);
        assert_eq!(result, Some(vec![0]));
    }

    #[test]
    fn test_header_map_find_header_pos_multiple() {
        let input = "Content-Length: 20\r\n\
                     Content-Type: application/json\r\n\
                     Content-Length: 20\r\n\r\n";
        let map = HeaderMap::from(BytesMut::from(input));
        let key = "Content-Length: 20";
        let result = map.header_position_all(key);
        assert_eq!(result, Some(vec![0, 2]));
    }

    // change_header
    #[test]
    fn test_header_map_change_header_single() {
        let input: BytesMut = "Content-Length: 20\r\n\r\n".into();
        let input_range = input.as_ptr_range();
        let mut map = HeaderMap::from(input);
        let old = "Content-Length: 20";
        let new = "Content-Length: 10";
        let result = map.change_header_all(old, new);
        assert!(result);
        let val = map.into_bytes();
        let verify = "Content-Length: 10\r\n\r\n";
        assert_eq!(val, verify);
        let result_range = val.as_ptr_range();
        assert_eq!(input_range, result_range);
    }

    #[test]
    fn test_header_map_change_header_multiple() {
        let input = "Content-Length: 20\r\n\
                     Content-Type: application/json\r\n\
                     Content-Length: 20\r\n\r\n";
        let mut map = HeaderMap::from(BytesMut::from(input));
        let old = "Content-Length: 20";
        let new = "Content-Length: 10";
        let result = map.change_header_all(old, new);
        assert!(result);
        let val = map.into_bytes();
        let verify = "Content-Length: 10\r\n\
                      Content-Type: application/json\r\n\
                      Content-Length: 10\r\n\r\n";
        assert_eq!(val, verify);
    }

    // remove header
    #[test]
    fn test_header_map_remove_header_first() {
        let raw_header: BytesMut = "Content-Type: application/json\r\n\
                                    Content-Length: 20\r\n\r\n"
            .into();
        let mut map = HeaderMap::from(raw_header);
        let to_remove = "Content-Length: 20";
        let result = map.remove_header_all(to_remove);
        assert!(result);
        let val = map.into_bytes();
        let verify = "Content-Type: application/json\r\n\r\n";
        assert_eq!(val, verify);
    }

    #[test]
    fn test_header_map_remove_header_second() {
        let input = "Content-Type: application/json\r\n\
                     Content-Length: 20\r\n\r\n";
        let mut map = HeaderMap::from(BytesMut::from(input));
        let to_remove = "Content-Type: application/json";
        let result = map.remove_header_all(to_remove);
        assert!(result);
        let val = map.into_bytes();
        let verify = "Content-Length: 20\r\n\r\n";
        assert_eq!(val, verify);
    }

    #[test]
    fn test_header_map_remove_header_multiple() {
        let input = "Content-Type: application/json\r\n\
                     Content-Length: 20\r\n\
                     Content-Type: application/json\r\n\r\n";
        let mut map = HeaderMap::from(BytesMut::from(input));
        let to_remove = "Content-Type: application/json";
        let result = map.remove_header_all(to_remove);
        assert!(result);
        let val = map.into_bytes();
        let verify = "Content-Length: 20\r\n\r\n";
        assert_eq!(val, verify);
    }

    #[test]
    fn test_header_map_has_header_key() {
        let raw_header: BytesMut = "Content-Length: 20\r\n\r\n".into();
        let map = HeaderMap::from(raw_header);
        let key = "Content-Length";
        let result = map.header_key_position(key);
        assert_eq!(result, Some(0));
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
