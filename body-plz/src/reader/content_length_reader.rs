use std::cmp::Ordering;

use buffer::Cursor;

/* Steps:
 *      1. Compare remaining length with size.
 *
 *      2. If remaining length is less than size, reduce size by remaining
 *         length and set buf position to current position + remaining length.
 *
 *      3. If remaining length is greater or equal, set buf position to current
 *         position + size.
 */

pub fn read_content_length(buf: &mut Cursor, size: &mut usize) -> bool {
    match buf.remaining().len().cmp(size) {
        Ordering::Less => {
            *size -= buf.remaining().len();
            buf.set_position(buf.position() + buf.remaining().len());
            false
        }
        Ordering::Greater | Ordering::Equal => {
            buf.set_position(buf.position() + *size);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_content_length_reader_read_success() {
        let mut data = BytesMut::zeroed(2000);
        let verify = data.clone();
        let mut buffer = Cursor::new(&mut data);
        let status = read_content_length(&mut buffer, &mut 2000);
        assert!(status);
        assert_eq!(buffer.position(), 2000);
        assert_eq!(buffer.split_at_current_pos(), verify);
    }

    #[test]
    fn test_content_length_reader_partial_growth() {
        let mut data = BytesMut::new();
        let mut buf = Cursor::new(&mut data);
        let mut size = 2000;
        for i in 1..5 {
            buf.as_mut().put_bytes(b'0', 500);
            let status = read_content_length(&mut buf, &mut size);
            if i == 4 {
                assert!(status);
                break;
            }
            assert_eq!(buf.position(), 500 * i);
            assert!(!status);
        }
    }

    #[test]
    fn test_content_length_reader_read_fail() {
        let mut data = BytesMut::zeroed(1999);
        let mut buffer = Cursor::new(&mut data);
        let status = read_content_length(&mut buffer, &mut 2000);
        assert!(!status);
        assert_eq!(buffer.position(), 1999);
    }

    #[test]
    fn test_content_length_reader_read_excess_data() {
        let mut data = BytesMut::zeroed(2100);
        let mut buffer = Cursor::new(&mut data);
        let status = read_content_length(&mut buffer, &mut 2000);
        assert!(status);
        assert_eq!(buffer.split_at_current_pos(), BytesMut::zeroed(2000));
        assert_eq!(buffer.into_inner(), BytesMut::zeroed(100));
    }

    #[test]
    fn test_content_length_reader_read_zero() {
        let mut data = BytesMut::new();
        let mut buffer = Cursor::new(&mut data);
        let status = read_content_length(&mut buffer, &mut 0);
        assert!(status);
        assert_eq!(buffer.position(), 0);
    }
}
