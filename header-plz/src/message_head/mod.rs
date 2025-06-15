use buffer_plz::Cursor;
use bytes::BytesMut;
use header_map::HeaderMap;
use info_line::InfoLine;

use crate::abnf::HEADER_DELIMITER;

mod try_from_bytes;

pub mod header_map;
pub mod info_line;

// Represent the Header region Infoline + HeaderMap.
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Eq))]
pub struct MessageHead<T> {
    info_line: T,
    header_map: HeaderMap,
}

impl<T> MessageHead<T>
where
    T: InfoLine,
{
    pub fn new(info_line: T, header_map: HeaderMap) -> Self {
        MessageHead {
            info_line,
            header_map,
        }
    }

    // Convert into Data
    pub fn into_bytes(self) -> BytesMut {
        let mut data = self.info_line.into_bytes();
        data.unsplit(self.header_map.into_bytes());
        data
    }

    pub fn header_map(&self) -> &HeaderMap {
        &self.header_map
    }

    pub fn infoline(&self) -> &T {
        &self.info_line
    }

    pub fn infoline_as_mut(&mut self) -> &mut T {
        &mut self.info_line
    }

    pub fn header_map_as_mut(&mut self) -> &mut HeaderMap {
        &mut self.header_map
    }
}

/* Steps:
 *      1. Find HEADER_DELIMITER ( 2 * CRLF ).
 *      2. If found, set buf position to index + 4 and return true.
 *      3. If not found, check if atlease buf len is 3 to consider atleast
 *         \r\n\r as received, set buf position to buf.len() - 3 and return
 *         false.
 */
impl MessageHead<()> {
    pub fn is_ended(buf: &mut Cursor) -> bool {
        if let Some(index) = buf
            .as_ref()
            .windows(4)
            .position(|window| window == HEADER_DELIMITER)
        {
            // 2. Found
            buf.set_position(index + 4);
            return true;
        }
        // 3. Considering \r\n\r as received
        if buf.len() > 3 {
            buf.set_position(buf.len() - 3);
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_header_reader_read_success() {
        let req = "GET /echo HTTP/1.1\r\n\
                    Host: reqbin.com\r\n\r\n";
        let mut buf = BytesMut::from(req);
        let mut cur = Cursor::new(&mut buf);
        let verify = BytesMut::from(req);
        let status: bool = MessageHead::is_ended(&mut cur);
        assert!(status);
        assert_eq!(cur.position(), verify.len());
    }

    #[test]
    fn test_header_reader_read_fail() {
        let req = "GET /echo HTTP/1.1\r\n\
                    Host: reqbin.com\r\n";
        let mut buf = BytesMut::from(req);
        let mut cur = Cursor::new(&mut buf);
        let status: bool = MessageHead::is_ended(&mut cur);
        assert!(!status);
        assert_eq!(cur.position(), req.len() - 3);
    }

    #[test]
    fn test_header_reader_read_growth() {
        let req = "GET /echo HTTP/1.1\r\n";
        let mut buf = BytesMut::from(req);
        let mut cur = Cursor::new(&mut buf);
        let mut status = MessageHead::is_ended(&mut cur);
        assert!(!status);
        assert_eq!(cur.position(), req.len() - 3);
        let toadd = b"Host: reqbin.com\r";
        cur.as_mut().extend_from_slice(toadd);
        status = MessageHead::is_ended(&mut cur);
        assert!(!status);
        assert_eq!(cur.position(), cur.as_ref().len() - 3);
        let toadd = b"\n\r\n";
        cur.as_mut().extend_from_slice(toadd);
        status = MessageHead::is_ended(&mut cur);
        assert!(status);
        assert_eq!(cur.position(), cur.as_ref().len());
    }
}
