use bytes::BytesMut;

use crate::abnf::HEADER_FS;

use super::Header;

/* Description:
 *      Contains atleast CRLF.
 *
 * Steps:
 *      1. Find ": " index.
 *      2. If no ": " found, split at index 1 as atleast CRLF if
 *         present.
 *      2. Split to key and value.
 *
 */
impl From<BytesMut> for Header {
    fn from(mut input: BytesMut) -> Self {
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
        Header::new(key, input)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_header_from_basic() {
        let buf = BytesMut::from("content-type: application/json\r\n");
        let verify_ptr = buf.as_ptr_range();
        let header = Header::from(buf);
        assert_eq!(header.key_as_str(), "content-type");
        assert_eq!(header.value_as_str(), "application/json");
        assert_eq!(verify_ptr, header.into_data().as_ptr_range());
    }

    #[test]
    fn test_header_from_fail_no_fs() {
        let buf = BytesMut::from("\r\n");
        let header = Header::from(buf);
        assert_eq!(header.key_as_str(), "\r");
        assert_eq!(header.value_as_str(), "\n");
    }

    #[test]
    fn test_header_from_len() {
        let buf: BytesMut = "content-type: application/json\r\n".into();
        let header = Header::from(buf);
        assert_eq!(header.len(), 32);
    }
}
