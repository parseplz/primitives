pub mod header;
use std::str::{self};

use bytes::BytesMut;
use header::*;

use crate::abnf::{COMMA, CRLF, SP};

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

    pub fn add_header(&mut self, header: Header) {
        self.headers.push(header);
    }

    // Finders
    pub fn find_pos_all<F>(&self, mut f: F) -> Option<Vec<usize>>
    where
        F: FnMut(&Header) -> bool,
    {
        let pos: Vec<usize> = self
            .headers
            .iter()
            .enumerate()
            .filter_map(|(i, h)| f(h).then_some(i))
            .collect();
        Some(pos).filter(|v| !v.is_empty())
    }

    pub fn find_pos<F>(&self, f: F) -> Option<usize>
    where
        F: FnMut(&Header) -> bool,
    {
        self.headers.iter().position(f)
    }

    // ---------- Entire header
    // ----- find
    pub fn header_position_all(&self, to_find_hdr: &str) -> Option<Vec<usize>> {
        let (key, val) = Header::split_header(to_find_hdr);
        self.find_pos_all(|h| {
            h.key_as_str().eq_ignore_ascii_case(key) && h.value_as_str().eq_ignore_ascii_case(val)
        })
    }

    pub fn header_position(&self, to_find_hdr: &str) -> Option<usize> {
        let (key, val) = Header::split_header(to_find_hdr);
        self.find_pos(|h| {
            h.key_as_str().eq_ignore_ascii_case(key) && h.value_as_str().eq_ignore_ascii_case(val)
        })
    }

    // ----- update
    // old : Content-Length: 20
    // new : Content-Length: 10
    pub fn update_header_all(&mut self, old: &str, new: &str) -> bool {
        let mut result = false;
        if let Some(positions) = self.header_position_all(old) {
            result = true;
            let (new_key, new_val) = Header::split_header(new);
            for index in positions {
                self.headers[index].change_key(new_key);
                self.headers[index].change_value(new_val);
            }
        }
        result
    }

    pub fn update_header(&mut self, old: &str, new: &str) -> bool {
        let mut result = false;
        if let Some(position) = self.header_position(old) {
            result = true;
            let (new_key, new_val) = Header::split_header(new);
            self.headers[position].change_key(new_key);
            self.headers[position].change_value(new_val);
        }
        result
    }

    // ----- remove
    pub fn remove_header_all_pos(&mut self, positions: Vec<usize>) {
        for index in positions.into_iter().rev() {
            self.headers.remove(index);
        }
    }

    // Content-Length: 10
    pub fn remove_header_all(&mut self, to_remove: &str) -> bool {
        let mut result = false;
        if let Some(positions) = self.header_position_all(to_remove) {
            result = true;
            self.remove_header_all_pos(positions);
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

    pub fn remove_header_on_position(&mut self, pos: usize) {
        self.headers.remove(pos);
    }

    // ---------- Key
    // ----- find
    pub fn header_key_position_all(&self, key: &str) -> Option<Vec<usize>> {
        self.find_pos_all(|h| h.key_as_str().eq_ignore_ascii_case(key))
    }

    pub fn header_key_position(&self, key: &str) -> Option<usize> {
        self.find_pos(|h| h.key_as_str().eq_ignore_ascii_case(key))
    }

    // ----- key -> value
    pub fn value_of_key(&self, key: &str) -> Option<&str> {
        self.find_pos(|h| h.key_as_str().eq_ignore_ascii_case(key))
            .map(|pos| self.headers[pos].value_as_str())
    }

    pub fn is_empty_value(&self, index: usize) -> bool {
        self.headers[index].value_as_str().is_empty()
    }

    // ----- update
    pub fn update_header_key_all(&mut self, old_key: &str, new_key: &str) -> bool {
        let mut result = false;
        if let Some(positions) = self.header_key_position_all(old_key) {
            result = true;
            for index in positions {
                self.headers[index].change_key(new_key);
            }
        }
        result
    }

    pub fn update_header_key(&mut self, old_key: &str, new_key: &str) -> bool {
        let mut result = false;
        if let Some(position) = self.header_key_position(old_key) {
            result = true;
            self.headers[position].change_key(new_key);
        }
        result
    }

    pub fn update_header_value_on_key_all(&mut self, key: &str, value: &str) -> bool {
        let mut result = false;
        if let Some(positions) = self.header_key_position_all(key) {
            result = true;
            for index in positions {
                self.headers[index].change_value(value);
            }
        }
        result
    }

    pub fn update_header_value_on_key(&mut self, key: &str, value: &str) -> bool {
        let mut result = false;
        if let Some(position) = self.header_key_position(key) {
            result = true;
            self.headers[position].change_value(value);
        }
        result
    }

    pub fn truncate_header_values_on_key<T>(&mut self, key: &str, remove: T)
    where
        T: AsRef<str>,
    {
        let Some(pos) = self.header_key_position(key) else {
            return;
        };
        let value = self.headers[pos].value_as_str();
        let Some(mut index) = value.find(remove.as_ref()) else {
            return;
        };

        for (i, c) in value[..index].char_indices().rev() {
            if c == SP || c == COMMA {
                index = i;
            } else {
                break;
            }
        }

        self.headers[pos].value_as_mut().truncate(index);
        self.headers[pos]
            .value_as_mut()
            .extend_from_slice(CRLF.as_bytes());
    }

    // ----- remove
    pub fn remove_header_on_key_all(&mut self, key: &str) -> bool {
        let mut result = false;
        if let Some(positions) = self.header_key_position_all(key) {
            result = true;
            self.remove_header_all_pos(positions);
        }
        result
    }

    pub fn remove_header_on_key(&mut self, key: &str) -> bool {
        let mut result = false;
        if let Some(position) = self.header_key_position(key) {
            result = true;
            self.headers.remove(position);
        }
        result
    }

    // ---------- value
    // ------ update
    pub fn update_header_value_on_pos(&mut self, pos: usize, value: &str) {
        self.headers[pos].change_value(value);
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

    use crate::{body_headers::content_encoding::ContentEncoding, const_headers::CONTENT_ENCODING};

    use super::*;

    fn build_input() -> BytesMut {
        let input = "Host: localhost\r\n\
                     Content-Length: 20\r\n\
                     Content-type: application/json\r\n\
                     Transfer-encoding: chunked\r\n\
                     Content-Length:20\r\n\
                     Content-Type:application/json\r\n\
                     Content-encoding: gzip\r\n\
                     Content-Length:20\r\n\
                     Content-Type:application/json\r\n\
                     Trailer: Some\r\n\
                     Connection: keep-alive\r\n\
                     X-custom-header: somevalue\r\n\r\n";
        BytesMut::from(input)
    }

    fn build_header_map() -> HeaderMap {
        let input = build_input();
        HeaderMap::from(input)
    }

    // ---------- Entire Header
    // ----- find
    #[test]
    fn test_header_map_find_header_pos_all() {
        let map = build_header_map();
        let key = "Content-Length: 20";
        let result = map.header_position_all(key);
        assert_eq!(result, Some(vec![1, 4, 7]));
    }

    #[test]
    fn test_header_map_find_header_pos() {
        let map = build_header_map();
        let key = "Content-Length: 20";
        let result = map.header_position(key);
        assert_eq!(result, Some(1));
    }

    // ----- update
    #[test]
    fn test_header_map_update_header_all() {
        let mut map = build_header_map();
        let old = "Content-Length: 20";
        let new = "Content-Length: 10";
        let result = map.update_header_all(old, new);
        assert!(result);
        let result = map.into_bytes();
        let verify = "Host: localhost\r\n\
                     Content-Length: 10\r\n\
                     Content-type: application/json\r\n\
                     Transfer-encoding: chunked\r\n\
                     Content-Length:10\r\n\
                     Content-Type:application/json\r\n\
                     Content-encoding: gzip\r\n\
                     Content-Length:10\r\n\
                     Content-Type:application/json\r\n\
                     Trailer: Some\r\n\
                     Connection: keep-alive\r\n\
                     X-custom-header: somevalue\r\n\r\n";
        assert_eq!(result, verify);
    }

    #[test]
    fn test_header_map_update_header() {
        let input = build_input();
        let input_range = input.as_ptr_range();
        let mut map = HeaderMap::from(input);
        let old = "Content-Length: 20";
        let new = "Content-Length: 10";
        let result = map.update_header(old, new);
        assert!(result);
        let result = map.into_bytes();
        let verify = "Host: localhost\r\n\
                     Content-Length: 10\r\n\
                     Content-type: application/json\r\n\
                     Transfer-encoding: chunked\r\n\
                     Content-Length:20\r\n\
                     Content-Type:application/json\r\n\
                     Content-encoding: gzip\r\n\
                     Content-Length:20\r\n\
                     Content-Type:application/json\r\n\
                     Trailer: Some\r\n\
                     Connection: keep-alive\r\n\
                     X-custom-header: somevalue\r\n\r\n";
        assert_eq!(result, verify);
        let result_range = result.as_ptr_range();
        assert_eq!(input_range, result_range);
    }

    // ----- remove
    #[test]
    fn test_header_map_remove_header_all() {
        let mut map = build_header_map();
        let to_remove = "Content-Type: application/json";
        let result = map.remove_header_all(to_remove);
        assert!(result);
        let result = map.into_bytes();
        let verify = "Host: localhost\r\n\
                      Content-Length: 20\r\n\
                      Transfer-encoding: chunked\r\n\
                      Content-Length:20\r\n\
                      Content-encoding: gzip\r\n\
                      Content-Length:20\r\n\
                      Trailer: Some\r\n\
                      Connection: keep-alive\r\n\
                      X-custom-header: somevalue\r\n\r\n";
        assert_eq!(result, verify);
    }

    #[test]
    fn test_header_map_remove_header() {
        let mut map = build_header_map();
        let to_remove = "Content-Length: 20";
        let result = map.remove_header(to_remove);
        assert!(result);
        let result = map.into_bytes();
        let verify = "Host: localhost\r\n\
                     Content-type: application/json\r\n\
                     Transfer-encoding: chunked\r\n\
                     Content-Length:20\r\n\
                     Content-Type:application/json\r\n\
                     Content-encoding: gzip\r\n\
                     Content-Length:20\r\n\
                     Content-Type:application/json\r\n\
                     Trailer: Some\r\n\
                     Connection: keep-alive\r\n\
                     X-custom-header: somevalue\r\n\r\n";
        assert_eq!(result, verify);
    }

    // ---------- Key
    // ------ find
    #[test]
    fn test_header_map_header_key_position_all() {
        let map = build_header_map();
        let key = "Content-Length";
        let result = map.header_key_position_all(key);
        assert_eq!(result, Some(vec![1, 4, 7]));
    }

    #[test]
    fn test_header_map_header_key_position() {
        let map = build_header_map();
        let key = "Content-Type";
        let result = map.header_key_position(key);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_header_map_header_key_value_of_key() {
        let map = build_header_map();
        let result = map.value_of_key("Content-Type");
        let verify = Some("application/json");
        assert_eq!(result, verify);
    }

    // ----- update
    #[test]
    fn test_header_map_update_header_key_all() {
        let mut map = build_header_map();
        let old = "Content-Length";
        let new = "Updated-Content-Length";
        let result = map.update_header_key_all(old, new);
        assert!(result);
        let result = map.into_bytes();
        let verify = "Host: localhost\r\n\
                      Updated-Content-Length: 20\r\n\
                      Content-type: application/json\r\n\
                      Transfer-encoding: chunked\r\n\
                      Updated-Content-Length:20\r\n\
                      Content-Type:application/json\r\n\
                      Content-encoding: gzip\r\n\
                      Updated-Content-Length:20\r\n\
                      Content-Type:application/json\r\n\
                      Trailer: Some\r\n\
                      Connection: keep-alive\r\n\
                      X-custom-header: somevalue\r\n\r\n";
        assert_eq!(result, verify);
    }

    #[test]
    fn test_header_map_update_header_key() {
        let mut map = build_header_map();
        let old = "Content-Length";
        let new = "Updated-Content-Length";
        let result = map.update_header_key(old, new);
        assert!(result);
        let result = map.into_bytes();
        let verify = "Host: localhost\r\n\
                      Updated-Content-Length: 20\r\n\
                      Content-type: application/json\r\n\
                      Transfer-encoding: chunked\r\n\
                      Content-Length:20\r\n\
                      Content-Type:application/json\r\n\
                      Content-encoding: gzip\r\n\
                      Content-Length:20\r\n\
                      Content-Type:application/json\r\n\
                      Trailer: Some\r\n\
                      Connection: keep-alive\r\n\
                      X-custom-header: somevalue\r\n\r\n";
        assert_eq!(result, verify);
    }

    #[test]
    fn test_header_map_update_header_value_on_key_all() {
        let mut map = build_header_map();
        let key = "Content-Length";
        let new_val = "30";
        let result = map.update_header_value_on_key_all(key, new_val);
        assert!(result);
        let result = map.into_bytes();
        let verify = "Host: localhost\r\n\
                     Content-Length: 30\r\n\
                     Content-type: application/json\r\n\
                     Transfer-encoding: chunked\r\n\
                     Content-Length:30\r\n\
                     Content-Type:application/json\r\n\
                     Content-encoding: gzip\r\n\
                     Content-Length:30\r\n\
                     Content-Type:application/json\r\n\
                     Trailer: Some\r\n\
                     Connection: keep-alive\r\n\
                     X-custom-header: somevalue\r\n\r\n";
        assert_eq!(result, verify);
    }

    #[test]
    fn test_header_map_update_header_value_on_key() {
        let mut map = build_header_map();
        let key = "Content-Length";
        let new_val = "30";
        let result = map.update_header_value_on_key(key, new_val);
        assert!(result);
        let result = map.into_bytes();
        let verify = "Host: localhost\r\n\
                     Content-Length: 30\r\n\
                     Content-type: application/json\r\n\
                     Transfer-encoding: chunked\r\n\
                     Content-Length:20\r\n\
                     Content-Type:application/json\r\n\
                     Content-encoding: gzip\r\n\
                     Content-Length:20\r\n\
                     Content-Type:application/json\r\n\
                     Trailer: Some\r\n\
                     Connection: keep-alive\r\n\
                     X-custom-header: somevalue\r\n\r\n";
        assert_eq!(result, verify);
    }

    // ----- remove
    #[test]
    fn test_header_map_remove_header_on_key_all() {
        let mut map = build_header_map();
        let key = "Content-Length";
        let result = map.remove_header_on_key_all(key);
        assert!(result);
        let result = map.into_bytes();
        let verify = "Host: localhost\r\n\
                      Content-type: application/json\r\n\
                      Transfer-encoding: chunked\r\n\
                      Content-Type:application/json\r\n\
                      Content-encoding: gzip\r\n\
                      Content-Type:application/json\r\n\
                      Trailer: Some\r\n\
                      Connection: keep-alive\r\n\
                      X-custom-header: somevalue\r\n\r\n";
        assert_eq!(result, verify);
    }

    #[test]
    fn test_header_map_remove_header_on_key() {
        let mut map = build_header_map();
        let key = "Content-Length";
        let result = map.remove_header_on_key(key);
        assert!(result);
        let result = map.into_bytes();
        let verify = "Host: localhost\r\n\
                      Content-type: application/json\r\n\
                      Transfer-encoding: chunked\r\n\
                      Content-Length:20\r\n\
                      Content-Type:application/json\r\n\
                      Content-encoding: gzip\r\n\
                      Content-Length:20\r\n\
                      Content-Type:application/json\r\n\
                      Trailer: Some\r\n\
                      Connection: keep-alive\r\n\
                      X-custom-header: somevalue\r\n\r\n";
        assert_eq!(result, verify);
    }

    // ---------- value
    // ----- update
    #[test]
    fn test_update_header_value_on_pos() {
        let mut map = build_header_map();
        let pos = 1;
        let val = "30";
        map.update_header_value_on_pos(pos, val);
        let result = map.into_bytes();
        let verify = "Host: localhost\r\n\
                      Content-Length: 30\r\n\
                      Content-type: application/json\r\n\
                      Transfer-encoding: chunked\r\n\
                      Content-Length:20\r\n\
                      Content-Type:application/json\r\n\
                      Content-encoding: gzip\r\n\
                      Content-Length:20\r\n\
                      Content-Type:application/json\r\n\
                      Trailer: Some\r\n\
                      Connection: keep-alive\r\n\
                      X-custom-header: somevalue\r\n\r\n";
        assert_eq!(result, verify);
    }

    // ------ len
    #[test]
    fn test_header_map_len_small() {
        let data = "a: b\r\n\r\n";
        let buf = BytesMut::from(data);
        let header_map = HeaderMap::from(buf);
        assert_eq!(header_map.len(), 8);
    }

    #[test]
    fn test_header_map_len_large() {
        let map = build_header_map();
        assert_eq!(map.len(), 290);
    }

    #[test]
    fn test_header_map_truncate_header_values() {
        let data = "Header: a,  b,c\r\n\r\n";
        let buf = BytesMut::from(data);
        let mut header_map = HeaderMap::from(buf);
        header_map.truncate_header_values_on_key("Header", "c");
        let result = header_map.into_bytes();
        assert_eq!(result, "Header: a,  b\r\n\r\n");

        let mut header_map = HeaderMap::from(result);
        header_map.truncate_header_values_on_key("Header", "b");
        let result = header_map.into_bytes();
        assert_eq!(result, "Header: a\r\n\r\n");
    }

    #[test]
    fn test_header_map_truncate_header_values_middle() {
        let input = "Content-Encoding: gzip, deflate, br\r\n\r\n";
        let mut header_map = HeaderMap::from(BytesMut::from(input));
        header_map.truncate_header_values_on_key(CONTENT_ENCODING, ContentEncoding::Deflate);
        let result = header_map.into_bytes();
        assert_eq!(result, "Content-Encoding: gzip\r\n\r\n");
    }

    #[test]
    fn test_header_map_truncate_header_values_all() {
        let input = "Content-Encoding: gzip, deflate, br\r\n\r\n";
        let mut header_map = HeaderMap::from(BytesMut::from(input));
        header_map.truncate_header_values_on_key(CONTENT_ENCODING, ContentEncoding::Gzip);
        let result = header_map.into_bytes();
        assert_eq!(result, "Content-Encoding: \r\n\r\n");
    }
}
