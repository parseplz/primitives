use crate::abnf::COLON;
use one::OneHeader;
use std::str::{self};
use two::Header;
pub mod one;
pub mod two;

use bytes::BytesMut;

use crate::abnf::CRLF;

pub trait Hmap {
    fn key_as_ref(&self) -> &[u8];

    fn value_as_ref(&self) -> &[u8];

    fn change_key(&mut self, key: &[u8]);

    fn change_value(&mut self, value: &[u8]);

    fn clear(&mut self);

    fn len(&self) -> usize;
}

pub type OneHeaderMap = HMap<OneHeader>;
pub type HeaderMap = HMap<Header>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HMap<T> {
    entries: Vec<T>,
    crlf: Option<BytesMut>,
}

impl<T> Default for HMap<T>
where
    T: Hmap + std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> HMap<T>
where
    T: Hmap + std::fmt::Debug,
{
    pub fn new() -> Self {
        HMap {
            entries: Vec::new(),
            crlf: None,
        }
    }

    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        T: From<(K, V)>,
    {
        self.entries.push(T::from((key, value)));
    }

    pub fn find_position_all<F>(
        &'a mut self,
        mut f: F,
    ) -> impl Iterator<Item = &'a mut T> + 'a
    where
        F: FnMut(&T) -> bool + 'a,
    {
        self.entries.iter_mut().filter_map(move |h| f(h).then_some(h))
    }

    pub fn find_position<F>(&self, f: F) -> Option<usize>
    where
        F: FnMut(&T) -> bool,
    {
        self.entries.iter().position(f)
    }

    // ---------- Entire header
    // ----- find
    pub fn header_position_all<K>(
        &'a mut self,
        to_find_hdr: K,
    ) -> impl Iterator<Item = &'a mut T> + 'a
    where
        T: From<K>,
    {
        let to_find: T = to_find_hdr.into();
        self.find_position_all(move |h| {
            h.key_as_ref().eq_ignore_ascii_case(to_find.key_as_ref())
                && h.value_as_ref()
                    .eq_ignore_ascii_case(to_find.value_as_ref())
        })
    }

    pub fn header_position<K>(&self, to_find_hdr: K) -> Option<usize>
    where
        T: From<K>,
    {
        let to_find: T = to_find_hdr.into();
        self.find_position(|h| {
            h.key_as_ref().eq_ignore_ascii_case(to_find.key_as_ref())
                && h.value_as_ref()
                    .eq_ignore_ascii_case(to_find.value_as_ref())
        })
    }

    // ----- update
    // old : Content-Length: 20
    // new : Content-Length: 10
    pub fn update_header_all<K>(&mut self, old: K, new: K) -> bool
    where
        T: From<K>,
    {
        let mut result = false;
        let new = T::from(new);

        for h in self.header_position_all(old) {
            result = true;
            h.change_key(new.key_as_ref());
            h.change_value(new.value_as_ref());
        }
        result
    }

    pub fn update_header<K>(&mut self, old: K, new: K) -> bool
    where
        T: From<K>,
    {
        let mut result = false;
        let new_hdr = T::from(new);
        if let Some(index) = self.header_position(old) {
            result = true;
            self.entries[index].change_key(new_hdr.key_as_ref());
            self.entries[index].change_value(new_hdr.value_as_ref());
        }
        result
    }

    // ----- remove
    pub fn remove_header_multiple_positions<U>(&mut self, positions: U) -> bool
    where
        U: Iterator<Item = usize>,
    {
        let mut result = false;
        for index in positions {
            result = true;
            self.entries[index].clear();
        }
        result
    }

    pub fn remove_header_all<K>(&mut self, to_remove: K) -> bool
    where
        T: From<K>,
    {
        let mut result = false;
        for entries in self.header_position_all(to_remove) {
            result = true;
            entries.clear()
        }
        result
    }

    pub fn remove_header<K>(&mut self, to_remove: K) -> bool
    where
        T: From<K>,
    {
        let mut result = false;
        if let Some(index) = self.header_position(to_remove) {
            result = true;
            self.entries[index].clear();
        }

        result
    }

    pub fn remove_header_on_position(&mut self, pos: usize) {
        self.entries[pos].clear();
    }

    // ---------- Key
    // ----- find
    pub fn header_key_position_all<K>(
        &'a mut self,
        key: K,
    ) -> impl Iterator<Item = &'a mut T> + 'a
    where
        K: AsRef<[u8]> + 'a,
    {
        self.find_position_all(move |h| {
            h.key_as_ref().eq_ignore_ascii_case(key.as_ref())
        })
    }

    pub fn header_key_position<K>(&self, key: K) -> Option<usize>
    where
        K: AsRef<[u8]> + 'a,
    {
        self.find_position(|h| {
            h.key_as_ref().eq_ignore_ascii_case(key.as_ref())
        })
    }

    pub fn contains_key<K>(&self, key: K) -> bool
    where
        K: AsRef<[u8]> + 'a,
    {
        self.header_key_position(key).is_some()
    }

    // ----- key -> value
    pub fn value_of_key<K>(&self, key: K) -> Option<&[u8]>
    where
        K: AsRef<[u8]> + 'a,
    {
        self.find_position(|h| {
            h.key_as_ref().eq_ignore_ascii_case(key.as_ref())
        })
        .map(|pos| self.entries[pos].value_as_ref())
    }

    // ----- update
    pub fn update_header_key_all<K>(&mut self, old_key: K, new_key: K) -> bool
    where
        K: AsRef<[u8]>,
    {
        let mut result = false;
        for entry in self.header_key_position_all(old_key) {
            result = true;
            entry.change_key(new_key.as_ref());
        }
        result
    }

    pub fn update_header_key<K>(&mut self, old_key: K, new_key: K) -> bool
    where
        K: AsRef<[u8]>,
    {
        let mut result = false;
        if let Some(pos) = self.header_key_position(old_key) {
            result = true;
            self.entries[pos].change_key(new_key.as_ref());
        }
        result
    }

    pub fn update_header_value_on_key_all<K>(
        &mut self,
        key: K,
        value: K,
    ) -> bool
    where
        K: AsRef<[u8]>,
    {
        let mut result = false;
        for entry in self.header_key_position_all(key) {
            result = true;
            entry.change_value(value.as_ref());
        }
        result
    }

    pub fn update_header_value_on_key<K>(&mut self, key: K, value: K) -> bool
    where
        K: AsRef<[u8]>,
    {
        let mut result = false;
        if let Some(pos) = self.header_key_position(key) {
            result = true;
            self.entries[pos].change_value(value.as_ref());
        }
        result
    }

    // remove
    pub fn remove_header_on_key_all<K>(&mut self, key: K) -> bool
    where
        K: AsRef<[u8]>,
    {
        let mut result = false;
        for entry in self.header_key_position_all(key) {
            result = true;
            entry.clear();
        }
        result
    }

    pub fn remove_header_on_key<K>(&mut self, key: K) -> bool
    where
        K: AsRef<[u8]>,
    {
        let mut result = false;
        if let Some(pos) = self.header_key_position(key) {
            result = true;
            self.entries[pos].clear();
        }
        result
    }

    // ---------- value
    // ------ update
    pub fn update_header_value_on_position<K>(&mut self, pos: usize, value: K)
    where
        K: AsRef<[u8]>,
    {
        self.entries[pos].change_value(value.as_ref());
    }

    pub fn build_multiple_value_key(
        values: impl Iterator<Item: AsRef<[u8]>>,
    ) -> BytesMut {
        let mut buf = BytesMut::new();
        let mut first = true;
        for value in values {
            if !first {
                buf.extend_from_slice(", ".as_bytes());
            }
            first = false;
            buf.extend_from_slice(value.as_ref());
        }
        buf
    }

    pub fn truncate_header_value_on_position() {
        todo!()
    }

    // ----- Misc
    pub fn len(&self) -> usize {
        self.entries.iter().fold(0, |total, entry| total + entry.len()) + 2
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/* Steps:
 *      1. Split the final CRLF.
 *      2. Create a new Vec<Header>
 *      ----- loop while input is not empty -----
 *      3. Find CRLF index.
 *      4. Split the line at crlf_index + 2.
 *      5. Create a new Header.
 *      6. Add the new Header to the new HeaderMap.
 */

impl From<BytesMut> for HMap<OneHeader> {
    fn from(mut input: BytesMut) -> Self {
        let crlf = input.split_off(input.len() - 2);
        let mut entries = Vec::new();
        while !input.is_empty() {
            let crlf_index = input
                .windows(2)
                .position(|b| b == CRLF.as_bytes())
                .unwrap_or(0);
            let header = input.split_to(crlf_index + 2);
            entries.push(OneHeader::from(header))
        }
        HMap {
            entries,
            crlf: Some(crlf),
        }
    }
}

impl HMap<OneHeader> {
    pub fn into_bytes(self) -> BytesMut {
        let mut buf = self.crlf.unwrap();
        for header in self.entries.into_iter().rev() {
            let mut data = header.into_bytes();
            data.unsplit(buf);
            buf = data;
        }
        buf
    }

    pub fn update_header_on_position_multiple_valuees(
        &mut self,
        pos: usize,
        values: impl Iterator<Item: AsRef<[u8]>>,
    ) {
        let mut buf = Self::build_multiple_value_key(values);
        buf.extend_from_slice(CRLF.as_bytes());
        self.entries[pos].change_value(buf.as_ref());
    }
}

impl HMap<Header> {
    pub fn update_header_on_position_multiple_values(
        &mut self,
        pos: usize,
        values: impl Iterator<Item: AsRef<[u8]>>,
    ) {
        let buf = Self::build_multiple_value_key(values);
        self.entries[pos].change_value(buf.as_ref());
    }
}

pub fn split_header(header: &str) -> (&str, &str) {
    header
        .split_once(COLON as char)
        .map(|(k, v)| (k, v.trim()))
        .unwrap_or_default()
}

impl From<HMap<OneHeader>> for HMap<Header> {
    fn from(one: HMap<OneHeader>) -> Self {
        let entries = one
            .entries
            .into_iter()
            .filter_map(|h| {
                if !h.is_empty() {
                    Some(Header::from(h))
                } else {
                    None
                }
            })
            .collect();
        HMap {
            entries,
            crlf: None,
        }
    }
}

impl From<HMap<Header>> for HMap<OneHeader> {
    fn from(two: HMap<Header>) -> Self {
        let entries = two
            .entries
            .into_iter()
            .filter_map(|h| {
                if !h.is_empty() {
                    Some(OneHeader::from(h))
                } else {
                    None
                }
            })
            .collect();
        HMap {
            entries,
            crlf: Some(CRLF.into()),
        }
    }
}

impl<'a, T> IntoIterator for &'a HMap<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

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

    fn build_from_two_verifier() -> HMap<OneHeader> {
        let input = "Host: localhost\r\n\
                     Content-Length: 20\r\n\
                     Content-type: application/json\r\n\
                     Transfer-encoding: chunked\r\n\
                     Content-Length: 20\r\n\
                     Content-Type: application/json\r\n\
                     Content-encoding: gzip\r\n\
                     Content-Length: 20\r\n\
                     Content-Type: application/json\r\n\
                     Trailer: Some\r\n\
                     Connection: keep-alive\r\n\
                     X-custom-header: somevalue\r\n\r\n";
        HMap::from(BytesMut::from(input))
    }

    fn build_test_one() -> HMap<OneHeader> {
        HMap::from(build_input())
    }

    fn build_test_two() -> HMap<Header> {
        build_test_one().into()
    }

    #[test]
    fn test_hmap_one_insert() {
        let mut map: HMap<OneHeader> = HMap::new();
        map.insert(BytesMut::from("key: "), BytesMut::from("value\r\n"));
        assert_eq!(map.entries.len(), 1);
        assert_eq!(map.entries[0].key_as_ref(), b"key");
        assert_eq!(map.entries[0].value_as_ref(), b"value");
    }

    #[test]
    fn test_hmap_two_insert() {
        let mut map: HMap<Header> = HMap::new();
        map.insert(Bytes::from("key"), Bytes::from("value"));
        assert_eq!(map.entries.len(), 1);
        assert_eq!(map.entries[0].key_as_ref(), b"key");
        assert_eq!(map.entries[0].value_as_ref(), b"value");
    }

    // ---------- Entire Header
    // ----- find
    #[test]
    fn test_hmap_find_header_pos_one() {
        let map = build_test_one();
        let _key = "Content-Length: 20";
        let result = map.header_position("content-length: 20");
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_hmap_find_header_pos_two() {
        let map = build_test_two();
        let _key = "Content-Length: 20";
        let result = map.header_position("content-length: 20");
        assert_eq!(result, Some(1));
    }

    // ----- update
    #[test]
    fn test_hmap_update_header_all_one() {
        let input = build_input();
        let input_range = input.as_ptr_range();
        let mut map = HMap::from(input);
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
        let result_range = result.as_ptr_range();
        assert_eq!(input_range, result_range);
    }

    #[test]
    fn test_hmap_update_header_one() {
        let input = build_input();
        let input_range = input.as_ptr_range();
        let mut map = HMap::from(input);
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

    #[test]
    fn test_hmap_update_header_all_two() {
        let mut map = build_test_two();
        let old = "Content-Length: 20";
        let new = "Content-Length: 10";
        let result = map.update_header_all(old, new);
        assert!(result);
        let mut verify = build_test_two();
        verify.entries[1].change_value(b"10");
        verify.entries[4].change_value(b"10");
        verify.entries[7].change_value(b"10");
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_update_header_two() {
        let mut map = build_test_two();
        let old = "Content-Length: 20";
        let new = "Content-Length: 10";
        let result = map.update_header(old, new);
        assert!(result);
        let mut verify = build_test_two();
        verify.entries[1].change_value(b"10");
        assert_eq!(map, verify);
    }

    // remove
    #[test]
    fn test_hmap_remove_header_all_one() {
        let mut map = build_test_one();
        let to_remove = "Content-Type: application/json";
        let result = map.remove_header_all(to_remove);
        assert!(result);
        let mut verify = build_test_one();
        verify.entries[2].clear();
        verify.entries[5].clear();
        verify.entries[8].clear();
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_remove_header_all_two() {
        let mut map = build_test_two();
        let to_remove = "Content-Type: application/json";
        let result = map.remove_header_all(to_remove);
        assert!(result);
        let mut verify = build_test_two();
        verify.entries[2].clear();
        verify.entries[5].clear();
        verify.entries[8].clear();
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_remove_header_one() {
        let mut map = build_test_one();
        let to_remove = "Content-Length: 20";
        let result = map.remove_header(to_remove);
        assert!(result);
        let mut verify = build_test_one();
        verify.entries[1].clear();
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_remove_header_two() {
        let mut map = build_test_two();
        let to_remove = "Content-Length: 20";
        let result = map.remove_header(to_remove);
        assert!(result);
        let mut verify = build_test_two();
        verify.entries[1].clear();
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_remove_header_on_position_one() {
        let mut map = build_test_one();
        map.remove_header_on_position(0);
        map.remove_header_on_position(2);
        map.remove_header_on_position(4);
        map.remove_header_on_position(6);
        map.remove_header_on_position(8);
        map.remove_header_on_position(10);
        let result = map.into_bytes();
        let verify = "Content-Length: 20\r\n\
                     Transfer-encoding: chunked\r\n\
                     Content-Type:application/json\r\n\
                     Content-Length:20\r\n\
                     Trailer: Some\r\n\
                     X-custom-header: somevalue\r\n\r\n";
        assert_eq!(result, verify);
    }

    #[test]
    fn test_hmap_remove_header_on_position_two() {
        let mut map = build_test_two();
        for i in (0..12).step_by(2) {
            map.remove_header_on_position(i);
        }
        let mut verify = build_test_two();
        for i in (0..12).step_by(2) {
            verify.entries[i].clear();
        }
        assert_eq!(map, verify);
    }

    // ---------- Key
    // ------ find
    #[test]
    fn test_hmap_header_key_position_one() {
        let map = build_test_one();
        let key = "Content-Type";
        let result = map.header_key_position(key);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_hmap_header_key_value_of_key_one() {
        let map = build_test_one();
        let result = map.value_of_key("Content-Type");
        let verify = "application/json";
        assert_eq!(result, Some(verify.as_bytes()));
    }

    #[test]
    fn test_hmap_header_key_position_two() {
        let map = build_test_two();
        let key = "Content-Type";
        let result = map.header_key_position(key);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_hmap_header_key_value_of_key_two() {
        let map = build_test_two();
        let result = map.value_of_key("Content-Type");
        let verify = "application/json";
        assert_eq!(result, Some(verify.as_bytes()));
    }

    // ----- update
    #[test]
    fn test_hmap_update_header_key_all_one() {
        let mut map = build_test_one();
        let old = "Content-Length";
        let new = "Updated-Content-Length";
        let result = map.update_header_key_all(old, new);
        assert!(result);

        let mut verify = build_test_one();
        verify.entries[1].change_key(new.as_bytes());
        verify.entries[4].change_key(new.as_bytes());
        verify.entries[7].change_key(new.as_bytes());
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_update_header_key_all_two() {
        let mut map = build_test_two();
        let old = "Content-Length";
        let new = "Updated-Content-Length";
        let result = map.update_header_key_all(old, new);
        assert!(result);
        let mut verify = build_test_two();
        verify.entries[1].change_key(new.as_bytes());
        verify.entries[4].change_key(new.as_bytes());
        verify.entries[7].change_key(new.as_bytes());
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_update_header_key_one() {
        let mut map = build_test_one();
        let old = "Content-Length";
        let new = "Updated-Content-Length";
        let result = map.update_header_key(old, new);
        assert!(result);
        let mut verify = build_test_one();
        verify.entries[1].change_key(new.as_bytes());
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_update_header_key_two() {
        let mut map = build_test_two();
        let old = "Content-Length";
        let new = "Updated-Content-Length";
        let result = map.update_header_key(old, new);
        assert!(result);
        let mut verify = build_test_two();
        verify.entries[1].change_key(new.as_bytes());
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_update_header_value_on_key_all_one() {
        let mut map = build_test_one();
        let key = "Content-Length";
        let new_val = "30";
        let result = map.update_header_value_on_key_all(key, new_val);
        assert!(result);
        let mut verify = build_test_one();
        verify.entries[1].change_value(new_val.as_bytes());
        verify.entries[4].change_value(new_val.as_bytes());
        verify.entries[7].change_value(new_val.as_bytes());
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_update_header_value_on_key_all_two() {
        let mut map = build_test_two();
        let key = "Content-Length";
        let new_val = "30";
        let result = map.update_header_value_on_key_all(key, new_val);
        assert!(result);
        let mut verify = build_test_two();
        verify.entries[1].change_value(new_val.as_bytes());
        verify.entries[4].change_value(new_val.as_bytes());
        verify.entries[7].change_value(new_val.as_bytes());
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_update_header_value_on_key_one() {
        let mut map = build_test_one();
        let key = "Content-Length";
        let new_val = "30";
        let result = map.update_header_value_on_key(key, new_val);
        assert!(result);
        let mut verify = build_test_one();
        verify.entries[1].change_value(new_val.as_bytes());
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_update_header_value_on_key_two() {
        let mut map = build_test_two();
        let key = "Content-Length";
        let new_val = "30";
        let result = map.update_header_value_on_key(key, new_val);
        assert!(result);
        let mut verify = build_test_two();
        verify.entries[1].change_value(new_val.as_bytes());
        assert_eq!(map, verify);
    }

    // ----- remove
    #[test]
    fn test_hmap_remove_header_on_key_all_one() {
        let mut map = build_test_one();
        let key = "Content-Length";
        let result = map.remove_header_on_key_all(key);
        assert!(result);
        let mut verify = build_test_one();
        verify.entries[1].clear();
        verify.entries[4].clear();
        verify.entries[7].clear();
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_remove_header_on_key_all_two() {
        let mut map = build_test_two();
        let key = "Content-Length";
        let result = map.remove_header_on_key_all(key);
        assert!(result);
        let mut verify = build_test_two();
        verify.entries[1].clear();
        verify.entries[4].clear();
        verify.entries[7].clear();
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_remove_header_on_key_one() {
        let mut map = build_test_one();
        let key = "Content-Length";
        let result = map.remove_header_on_key(key);
        assert!(result);
        let mut verify = build_test_one();
        verify.entries[1].clear();
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_remove_header_on_key_two() {
        let mut map = build_test_two();
        let key = "Content-Length";
        let result = map.remove_header_on_key(key);
        assert!(result);
        let mut verify = build_test_two();
        verify.entries[1].clear();
        assert_eq!(map, verify);
    }

    // ---------- value
    // ----- update
    #[test]
    fn test_hmap_update_header_value_on_pos_one() {
        let mut map = build_test_one();
        let pos = 1;
        let val = "30";
        map.update_header_value_on_position(pos, val);
        let mut verify = build_test_one();
        verify.entries[1].change_value(val.as_bytes());
        assert_eq!(map, verify);
    }

    #[test]
    fn test_hmap_update_header_value_on_pos_two() {
        let mut map = build_test_two();
        let pos = 1;
        let val = "30";
        map.update_header_value_on_position(pos, val);
        let mut verify = build_test_two();
        verify.entries[1].change_value(val.as_bytes());
        assert_eq!(map, verify);
    }

    #[ignore]
    #[test]
    fn test_update_update_header_multiple_values_on_position() {
        let _map = build_test_one();
    }

    // len
    #[test]
    fn test_hmap_len() {
        let map = build_test_one();
        assert_eq!(map.len(), 290);
        let map = build_test_two();
        assert_eq!(map.len(), 246);
    }

    #[test]
    fn test_header_map_len_small() {
        let data = "a: b\r\n\r\n";
        let buf = BytesMut::from(data);
        let header_map = HMap::from(buf);
        assert_eq!(header_map.len(), 8);

        let mut map: HMap<Header> = HMap::new();
        let one = build_test_one();
        for entry in one.entries.into_iter() {
            map.insert(
                Bytes::from(entry.key_as_ref().to_owned()),
                Bytes::from(entry.value_as_ref().to_owned()),
            );
        }
        assert_eq!(header_map.len(), 8);
    }

    // one to two
    #[test]
    fn test_from_one_to_two() {
        let one = build_test_one();
        let two = HMap::<Header>::from(one);
        assert_eq!(two, build_test_two());
    }

    #[test]
    fn test_from_one_to_two_remove_header() {
        let mut one = build_test_one();
        let to_remove = "Content-Length: 20";
        let result = one.remove_header(to_remove);
        assert!(result);
        let two = HMap::<Header>::from(one);
        let mut verify = build_test_two();
        verify.entries.remove(1);
        assert_eq!(two, verify);
    }

    #[test]
    fn test_from_one_to_two_remove_header_all() {
        let mut one = build_test_one();
        let to_remove = "Content-Type: application/json";
        let result = one.remove_header_all(to_remove);
        assert!(result);
        let two = HMap::<Header>::from(one);
        let mut verify = build_test_two();
        verify.entries.remove(2);
        verify.entries.remove(4);
        verify.entries.remove(6);
        assert_eq!(two, verify);
    }

    // two to one
    #[test]
    fn test_from_two_to_one() {
        let two = build_test_two();
        let one = HMap::<OneHeader>::from(two);
        assert_eq!(one, build_from_two_verifier());
    }

    #[test]
    fn test_from_two_to_one_remove_header() {
        let mut two = build_test_two();
        let to_remove = "Content-Length: 20";
        let result = two.remove_header(to_remove);
        assert!(result);
        let one = HMap::<OneHeader>::from(two);
        let mut verify = build_from_two_verifier();
        verify.entries.remove(1);
        assert_eq!(one, verify);
    }

    #[test]
    fn test_from_two_to_one_remove_header_all() {
        let mut two = build_test_two();
        let to_remove = "Content-Type: application/json";
        let result = two.remove_header_all(to_remove);
        assert!(result);
        let one = HMap::<OneHeader>::from(two);
        let mut verify = build_from_two_verifier();
        verify.entries.remove(2);
        verify.entries.remove(4);
        verify.entries.remove(6);
        assert_eq!(one, verify);
    }
}
