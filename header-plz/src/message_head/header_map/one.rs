use bytes::Buf;
use bytes::buf::Chain;
use bytes::{BufMut, Bytes, BytesMut};

use crate::{
    abnf::*,
    message_head::header_map::{HeaderStr, HeaderVersion, Hmap, two::Header},
    version::Version,
};

#[derive(Clone, Eq, Debug, PartialEq)]
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

    pub fn as_chain(&self) -> Chain<&[u8], &[u8]> {
        self.key[..].chain(&self.value[..])
    }
}

impl HeaderStr for OneHeader {
    fn key_as_str(&self) -> Option<&str> {
        str::from_utf8(&self.key)
            .ok()
            .and_then(|s| s.split(COLON as char).nth(0))
    }

    fn value_as_str(&self) -> Option<&str> {
        str::from_utf8(&self.value).ok().and_then(|s| {
            s.split(str::from_utf8(CRLF).unwrap_or_default()).nth(0)
        })
    }
}

impl HeaderVersion for OneHeader {
    fn version(&self) -> crate::version::Version {
        Version::H11
    }

    fn is_one_one(&self) -> bool {
        true
    }

    fn is_two(&self) -> bool {
        false
    }
}

impl<T, E> From<(T, E)> for OneHeader
where
    T: AsRef<[u8]>,
    E: AsRef<[u8]>,
{
    fn from((key, value): (T, E)) -> Self {
        let mut key = BytesMut::from(key.as_ref());
        if !key.ends_with(HEADER_FS) {
            key.extend_from_slice(HEADER_FS);
        }
        let mut value = BytesMut::from(value.as_ref());
        if !value.ends_with(CRLF) {
            value.extend_from_slice(CRLF);
        }
        OneHeader {
            key,
            value,
        }
    }
}

impl From<&[u8]> for OneHeader {
    fn from(input: &[u8]) -> Self {
        let fs_index = find_header_fs(input);
        let (key, value) = if fs_index == 0 {
            // key
            (input, "".as_bytes())
        } else {
            // key: val
            input.split_at(fs_index)
        };
        OneHeader::from((key, value))
    }
}

// Content-Type: application/json
impl From<&str> for OneHeader {
    fn from(input: &str) -> Self {
        OneHeader::from(input.as_bytes())
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
        OneHeader::new(key, input)
    }
}

// TODO: utf-8 check ?
impl From<OneHeader> for Header {
    fn from(mut one: OneHeader) -> Self {
        let key = match one.key.iter().position(|b| b == &COLON) {
            Some(0) | None => Bytes::new(),
            Some(fs_index) => {
                let mut key = one.key.split_to(fs_index);
                key.make_ascii_lowercase();
                key.freeze()
            }
        };

        // if only CRLF or no value
        let value = if one.value_len() < 3 {
            Bytes::new()
        } else {
            one.value.split_to(one.value_len() - 2).freeze()
        };
        Header::from((key, value))
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
            self.key.extend_from_slice(HEADER_FS);
        } else {
            self.key.put_u8(COLON);
        }
    }

    fn change_value(&mut self, value: &[u8]) {
        reuse_or_swap(value.len() + 2, &mut self.value, value);
        self.value.extend_from_slice(CRLF);
    }

    fn clear(&mut self) {
        self.key.clear();
        self.value.clear();
    }

    fn len(&self) -> usize {
        self.key.len() + self.value.len()
    }

    fn truncate_value(&mut self, pos: usize) {
        self.value.truncate(pos);
        self.value.extend_from_slice(CRLF);
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
    use crate::const_headers::CONTENT_TYPE;

    use super::*;

    // from
    #[test]
    fn test_one_header_from_tuple_mixed() {
        let value = "application/json";
        let header: OneHeader = (CONTENT_TYPE, value).into();
        let expected = OneHeader {
            key: BytesMut::from("content-type: "),
            value: BytesMut::from("application/json\r\n"),
        };
        assert_eq!(header, expected);
        let mut chain = expected.as_chain();
        let verify = chain.copy_to_bytes(chain.remaining());
        assert_eq!(verify, expected.into_bytes());
    }

    #[test]
    fn test_one_header_from_tuple_slice() {
        let value = "application/json";
        let header: OneHeader = (CONTENT_TYPE, value.as_bytes()).into();
        let expected = OneHeader {
            key: BytesMut::from("content-type: "),
            value: BytesMut::from("application/json\r\n"),
        };
        assert_eq!(header, expected);
        let mut chain = expected.as_chain();
        let verify = chain.copy_to_bytes(chain.remaining());
        assert_eq!(verify, expected.into_bytes());
    }

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
        let mut chain = expected.as_chain();
        let verify = chain.copy_to_bytes(chain.remaining());
        assert_eq!(verify, expected.into_bytes());
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
        let mut chain = expected.as_chain();
        let verify = chain.copy_to_bytes(chain.remaining());
        assert_eq!(verify, expected.into_bytes());
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
        let mut chain = expected.as_chain();
        let verify = chain.copy_to_bytes(chain.remaining());
        assert_eq!(verify, expected.into_bytes());
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
        let mut chain = expected.as_chain();
        let verify = chain.copy_to_bytes(chain.remaining());
        assert_eq!(verify, expected.into_bytes());
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
        let mut chain = header.as_chain();
        let verify = chain.copy_to_bytes(chain.remaining());
        let toverify = header.into_bytes();
        assert_eq!(verify, toverify);
        assert_eq!(verify_ptr, toverify.as_ptr_range());
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
        let mut chain = header.as_chain();
        let verify = chain.copy_to_bytes(chain.remaining());
        let toverify = header.into_bytes();
        assert_eq!(verify, toverify);
        assert_eq!(verify_ptr, toverify.as_ptr_range());
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
        let mut chain = header.as_chain();
        let verify = chain.copy_to_bytes(chain.remaining());
        let toverify = header.into_bytes();
        assert_eq!(verify, toverify);
        assert_eq!(verify_ptr, toverify.as_ptr_range());
    }

    #[test]
    fn test_one_header_from_bytesmut_len() {
        let buf: BytesMut = "content-type: application/json\r\n".into();
        let header = OneHeader::from(buf);
        assert_eq!(header.len(), 32);
    }

    #[test]
    fn test_one_to_two_perfect() {
        let buf: BytesMut = "Content-type: application/json\r\n".into();
        let one = OneHeader::from(buf);
        let two = Header::from(one);
        assert_eq!(two.key_as_ref(), b"content-type");
        assert_eq!(two.value_as_ref(), b"application/json");
    }

    #[test]
    fn test_one_to_two_no_space() {
        let buf: BytesMut = "Content-type:application/json\r\n".into();
        let one = OneHeader::from(buf);
        let two = Header::from(one);
        assert_eq!(two.key_as_ref(), b"content-type");
        assert_eq!(two.value_as_ref(), b"application/json");
    }

    #[test]
    fn test_one_to_two_empty_value() {
        let buf: BytesMut = "Content-type:\r\n".into();
        let one = OneHeader::from(buf);
        let two = Header::from(one);
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

    #[test]
    fn test_truncate_value() {
        let mut input = OneHeader::from(("key", "hola, que, tal"));
        input.truncate_value(9);
        let verify = OneHeader::from(("key", "hola, que"));
        assert_eq!(input, verify);
    }
}
