use bytes::BytesMut;

use crate::abnf::CRLF;

use super::{HeaderMap, header::Header};

/* Steps:
 *      1. Check if input is valid utf8 string.
 *      2. If not valid, convert to valid utf8 string using
 *         String::from_utf8_lossy() and convert to BytesMut and assign to
 *         input.
 *      3. Split the final CRLF.
 *      4. Create a new Vec<Header>
 *      ----- loop while input is not empty -----
 *      5. Find CRLF index.
 *      6. Split the line at crlf_index + 2.
 *      7. Create a new Header.
 *      8. Add the new Header to the new HeaderMap.
 */

impl From<BytesMut> for HeaderMap {
    fn from(mut input: BytesMut) -> Self {
        input = if std::str::from_utf8(&input).is_ok() {
            input
        } else {
            // 2. If not valid, convert to valid utf8 string
            String::from_utf8_lossy(&input).as_bytes().into()
        };
        let crlf = input.split_off(input.len() - 2);
        let mut header_vec = Vec::new();
        while !input.is_empty() {
            // safe to unwrap checked in step 1
            let header_str = str::from_utf8(&input).unwrap();
            let crlf_index = header_str.find(CRLF).unwrap_or(0);
            let header_bytes = input.split_to(crlf_index + 2);
            let h = Header::from(header_bytes);
            header_vec.push(h);
        }
        HeaderMap::new(header_vec, crlf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_map() {
        let buf: BytesMut = "content-type: application/json\r\n\
                    transfer-encoding: chunked\r\n\
                    content-encoding: gzip\r\n\
                    trailer: Some\r\n\
                    x-custom-header: somevalue\r\n\r\n"
            .into();
        let verify = buf.clone();
        let verify_ptr = buf.as_ptr_range();
        let header_map = HeaderMap::from(buf);
        let result = header_map.into_data();
        assert_eq!(verify, result);
        assert_eq!(verify_ptr, result.as_ptr_range());
    }

    #[test]
    fn test_header_map_crlf_only() {
        let buf: BytesMut = "\r\n".into();
        let verify = buf.clone();
        let header_map = HeaderMap::from(buf);
        assert_eq!(header_map.headers, vec![]);
        assert_eq!(header_map.crlf, verify);
    }
}
