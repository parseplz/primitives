use buffer_plz::Cursor;
use bytes::{Buf, BytesMut};

use crate::{
    abnf::HEADER_DELIMITER,
    message_head::{
        header_map::{HMap, OneHeaderMap, one::OneHeader},
        info_line::one::InfoLine,
    },
};

mod try_from_bytes;

pub mod header_map;
pub(crate) mod info_line;

pub type OneMessageHead<I> = MessageHead<I, OneHeader>;

// Represent the Header region Infoline + HeaderMap.
#[derive(Clone, Eq, Debug, PartialEq)]
pub struct MessageHead<I, H> {
    info_line: I,
    header_map: HMap<H>,
}

impl<T> MessageHead<T, OneHeader>
where
    T: InfoLine,
{
    pub fn new(info_line: T, header_map: OneHeaderMap) -> Self {
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

    pub fn header_map(&self) -> &OneHeaderMap {
        &self.header_map
    }

    pub fn info_line(&self) -> &T {
        &self.info_line
    }

    pub fn info_line_mut(&mut self) -> &mut T {
        &mut self.info_line
    }

    pub fn header_map_mut(&mut self) -> &mut OneHeaderMap {
        &mut self.header_map
    }

    pub fn as_chain(&self) -> impl Buf {
        self.info_line().as_chain().chain(self.header_map.as_chain())
    }
}

impl<I, H> MessageHead<I, H> {
    pub fn into_parts(self) -> (I, HMap<H>) {
        (self.info_line, self.header_map)
    }
}

/* Steps:
 *      1. Find HEADER_DELIMITER ( 2 * CRLF ).
 *      2. If found, set buf position to index + 4 and return true.
 *      3. If not found, check if atlease buf len is 3 to consider atleast
 *         \r\n\r as received, set buf position to buf.len() - 3 and return
 *         false.
 */
impl MessageHead<(), ()> {
    pub fn is_complete(buf: &mut Cursor) -> bool {
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

    use crate::{OneRequestLine, OneResponseLine};

    use super::*;

    #[test]
    fn test_header_reader_read_success() {
        let req = "GET /echo HTTP/1.1\r\n\
                    Host: reqbin.com\r\n\r\n";
        let mut buf = BytesMut::from(req);
        let mut cur = Cursor::new(&mut buf);
        let verify = BytesMut::from(req);
        let status: bool = MessageHead::is_complete(&mut cur);
        assert!(status);
        assert_eq!(cur.position(), verify.len());
    }

    #[test]
    fn test_header_reader_read_fail() {
        let req = "GET /echo HTTP/1.1\r\n\
                    Host: reqbin.com\r\n";
        let mut buf = BytesMut::from(req);
        let mut cur = Cursor::new(&mut buf);
        let status: bool = MessageHead::is_complete(&mut cur);
        assert!(!status);
        assert_eq!(cur.position(), req.len() - 3);
    }

    #[test]
    fn test_header_reader_read_growth() {
        let req = "GET /echo HTTP/1.1\r\n";
        let mut buf = BytesMut::from(req);
        let mut cur = Cursor::new(&mut buf);
        let mut status = MessageHead::is_complete(&mut cur);
        assert!(!status);
        assert_eq!(cur.position(), req.len() - 3);
        let toadd = b"Host: reqbin.com\r";
        cur.as_mut().extend_from_slice(toadd);
        status = MessageHead::is_complete(&mut cur);
        assert!(!status);
        assert_eq!(cur.position(), cur.as_ref().len() - 3);
        let toadd = b"\n\r\n";
        cur.as_mut().extend_from_slice(toadd);
        status = MessageHead::is_complete(&mut cur);
        assert!(status);
        assert_eq!(cur.position(), cur.as_ref().len());
    }

    #[test]
    fn test_message_head_req_chain() {
        let input = "GET / HTTP/1.1\r\n\
                       Host: localhost\r\n\
                       Accept: text/html\r\n\
                       Accept-Language: en-US,en;q=0.5\r\n\
                       Accept-Encoding: gzip, deflate\r\n\
                       User-Agent: curl/7.29.0\r\n\
                       Connection: keep-alive\r\n\r\n";
        let buf = BytesMut::from(input);
        let msg_head =
            OneMessageHead::<OneRequestLine>::try_from(buf).unwrap();
        let mut chain = msg_head.as_chain();
        let verify = chain.copy_to_bytes(chain.remaining());
        assert_eq!(verify, input);
    }

    #[test]
    fn test_message_head_res_chain() {
        let input = "HTTP/1.1 200 OK\r\n\
                        Host: localhost\r\n\
                        Content-Type: text/plain\r\n\
                        Content-Length: 12\r\n\r\n";
        let buf = BytesMut::from(input);
        let msg_head =
            OneMessageHead::<OneResponseLine>::try_from(buf).unwrap();
        let mut chain = msg_head.as_chain();
        let verify = chain.copy_to_bytes(chain.remaining());
        assert_eq!(verify, input);
    }
}
