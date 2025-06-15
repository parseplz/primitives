use super::Header;
use crate::abnf::{COLON, SP};
use bytes::BytesMut;

pub fn find_header_fs(input: &str) -> usize {
    if let Some(index) = input.find(COLON) {
        // check if index + 1 == Space i.e. ": "
        if input.chars().nth(index + 1).unwrap() == SP {
            index + 2
            // only ":"
        } else {
            index + 1
        }
    } else {
        0
    }
}

/* Description:
 *      Contains atleast CRLF.
 */
impl From<BytesMut> for Header {
    fn from(mut input: BytesMut) -> Self {
        // utf8 already checked in HeaderMap::new()
        // safe to unwrap
        let input_str = str::from_utf8(&input).unwrap();
        let fs_index = find_header_fs(input_str);

        // 2. If no ": " found, split at index 1 as atleast CRLF if present.
        let key = if fs_index == 0 {
            input.split()
        } else {
            input.split_to(fs_index)
        };
        Header::new(key, input)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header_from_bytesmut_basic() {
        let buf = BytesMut::from("content-type: application/json\r\n");
        let verify_ptr = buf.as_ptr_range();
        let header = Header::from(buf);
        assert_eq!(header.key_as_str(), "content-type");
        assert_eq!(header.value_as_str(), "application/json");
        assert_eq!(verify_ptr, header.into_bytes().as_ptr_range());
    }

    #[test]
    fn test_header_from_bytesmut_no_space() {
        let buf = BytesMut::from("content-type:application/json\r\n");
        let verify_ptr = buf.as_ptr_range();
        let header = Header::from(buf);
        assert_eq!(header.key_as_str(), "content-type");
        assert_eq!(header.value_as_str(), "application/json");
        assert_eq!(verify_ptr, header.into_bytes().as_ptr_range());
    }

    #[test]
    fn test_header_from_bytesmut_fail_no_fs() {
        let buf = BytesMut::from("\r\n");
        let header = Header::from(buf);
        assert_eq!(header.key_as_str(), "\r\n");
        assert_eq!(header.value_as_str(), "");
    }

    #[test]
    fn test_header_from_bytesmut_len() {
        let buf: BytesMut = "content-type: application/json\r\n".into();
        let header = Header::from(buf);
        assert_eq!(header.len(), 32);
    }
}
