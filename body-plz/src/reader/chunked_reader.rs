use std::num::ParseIntError;

use buffer_plz::Cursor;
use bytes::BytesMut;
use thiserror::Error;

use crate::body_struct::ChunkedBody;

use super::content_length_reader::read_content_length;
use header_plz::abnf::CRLF;
use header_plz::header_map::HeaderMap;
use header_plz::reader::read_header;

// Enum to represent chunked reader errors
#[cfg_attr(any(debug_assertions, test), derive(Eq, PartialEq))]
#[derive(Debug, Error)]
pub enum ChunkReaderError {
    #[error("UnabletoFindCRLF| {}", .0)]
    SplitExtension(String),
    #[error("SizeDecode| {}", .0)]
    Size(#[from] ParseIntError),
    #[error("LastChunkPoll")]
    LastChunkPoll,
}

// Enum to represent chunked reader state
#[cfg_attr(any(debug_assertions, test), derive(Eq, PartialEq))]
#[derive(Debug)]
pub enum ChunkReader {
    ReadSize,
    ReadChunk(usize),
    LastChunk,
    ReadTrailers,
    EndCRLF,
    End,
    Failed(ChunkReaderError),
}

/* Description:
 *      Method to read chunked body.
 *
 * Steps:
 *      1. ReadSize
 *          a. call mark_size_chunk() to mark the size chunk (hex_size + CRLF)
 *
 *          b. if size is marked, then call get_size() to convert hex_size to
 *          integer
 *              1. if size == 0, then state is LastChunk and return
 *                 ChunkedBody::LastChunk
 *
 *              2. else, ReadChunk(size + 2) i.e. data + CRLF, state is
 *                 ReadChunk and return ChunkedBody::Size
 *
 *          c. If mark_size_chunk() returns false, change state to Failed and
 *          return None
 *
 *      2. ReadChunk call content_length_read() with buf and size as args,
 *         if true, then trasition to ReadSize and return ChunkedBody::Chunk
 *
 *      3. LastChunk should not reach this state. If reach this state, then
 *         change state to Failed and return LastChunk
 *
 *      4. ReadTrailers
 *          a. If empty header,(only CRLF) then change state to End and return
 *          ChunkedBody::EndCRLF
 *
 *          b. else, call read_header() with buf as args, if true, then change
 *          state to End and build HeaderMap and return ChunkedBody::Trailers
 *          with the HeaderMap
 *
 *      5. EndCRLF If buf is CRLF, then change state to End and return
 *         ChunkedBody::EndCRLF
 *
 * Error:
 *      ChunkReaderError::LastChunkPoll [3]
 */

impl ChunkReader {
    pub fn next(&mut self, buf: &mut Cursor) -> Option<ChunkedBody> {
        match self {
            // 1. Read Size
            Self::ReadSize => {
                // 1.a. call mark_size_chunk()
                if ChunkReader::mark_size_chunk(buf) {
                    match Self::get_size(buf) {
                        Ok(size) => {
                            // 1.b.1. If size == 0, then LastChunk
                            if size == 0 {
                                *self = Self::LastChunk;
                                return Some(ChunkedBody::LastChunk(buf.split_at_current_pos()));
                                // 1.b.2. else, ReadChunk(size + 2)
                            } else {
                                *self = Self::ReadChunk(size + 2);
                            }
                            return Some(ChunkedBody::Size(buf.split_at_current_pos()));
                        }
                        // 1.c. If get_size() returns error, Failed State
                        Err(e) => {
                            *self = Self::Failed(e);
                            return None;
                        }
                    }
                }
                None
            }

            &mut Self::ReadChunk(ref mut size) => {
                if read_content_length(buf, size) {
                    *self = Self::ReadSize;
                    return Some(ChunkedBody::Chunk(buf.split_at_current_pos()));
                }
                None
            }
            Self::LastChunk => {
                *self = Self::Failed(ChunkReaderError::LastChunkPoll);
                // Temproary return Should panic at this state
                Some(ChunkedBody::LastChunk(BytesMut::new()))
            }
            Self::ReadTrailers => {
                // 4.a. If Empty Header
                if buf.remaining() == CRLF.as_bytes() {
                    buf.set_position(buf.position() + 2);
                    *self = Self::End;
                    return Some(ChunkedBody::EndCRLF(buf.split_at_current_pos()));
                }
                // 4.b. Actual Headers
                if read_header(buf) {
                    *self = Self::End;
                    let header_map = HeaderMap::new(buf.split_at_current_pos());
                    Some(ChunkedBody::Trailers(header_map))
                } else {
                    None
                }
            }
            Self::EndCRLF => {
                if let Some(index) = buf
                    .remaining()
                    .windows(2)
                    .position(|window| window == CRLF.as_bytes())
                {
                    buf.set_position(index + 2);
                    *self = Self::End;
                    Some(ChunkedBody::EndCRLF(buf.split_at_current_pos()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // find the position of CRLF in size chunk.
    fn mark_size_chunk(buf: &mut Cursor) -> bool {
        if let Some(index) = buf
            .remaining()
            .windows(2)
            .position(|window| window == CRLF.as_bytes())
        {
            // size_index
            buf.set_position(index);
            return true;
        }
        false
    }

    /* Description:
     *      Read chunk size given the position (CRLF) till the chunk_size is
     *      marked by calling mark_size_chunk.
     *
     * Format:
     *      chunk_size; chunk_extension \r\n.
     *
     * Steps:
     *      1. split buf at ";" , get first part (hex size).
     *      2. Convert hex size to integer.
     *      3. Move cursor pos to include CRLF.
     *
     *      NOTE: chunk_extension is ignored.
     *
     * Error:
     *      ChunkReaderError::NotValidUtf       [1]
     *      ChunkReaderError::ChunkedExtension  [2]
     *      ChunkReaderError::Size              [3]
     */

    fn get_size(buf: &mut Cursor) -> Result<usize, ChunkReaderError> {
        // 1. Convert the chunk to str, split ";" , get first part (hex size).
        let hex_size = &buf.as_ref()[0..buf.position()]
            .split(|c| *c == b';')
            .nth(0)
            .ok_or(ChunkReaderError::SplitExtension(
                String::from_utf8_lossy(&buf.as_ref()[0..buf.position()]).to_string(),
            ))?;
        // 2. Convert hex size to integer.
        let size = usize::from_str_radix(&String::from_utf8_lossy(hex_size), 16)?;
        // 3. Add CRLF
        buf.set_position(buf.position() + 2);
        Ok(size)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_chunked_reader_chunksize() {
        let data = "4\r\n";
        let mut buf = BytesMut::from(data.as_bytes());
        let mut cbuf = Cursor::new(&mut buf);
        let verify = cbuf.as_ref().into();
        let mut state = ChunkReader::ReadSize;
        let result = state.next(&mut cbuf);
        assert_eq!(result.unwrap(), ChunkedBody::Size(verify));
        assert_eq!(cbuf.position(), cbuf.len());
        assert_eq!(state, ChunkReader::ReadChunk(6));
    }

    #[test]
    fn test_chunked_reader_chunk() {
        let data = "mozilla\r\ngees";
        let mut buf = BytesMut::from(data.as_bytes());
        let mut cbuf = Cursor::new(&mut buf);

        let verify = "mozilla\r\n";
        let mut state = ChunkReader::ReadChunk(9);
        let result = state.next(&mut cbuf);
        assert_eq!(result.unwrap(), ChunkedBody::Chunk(verify.into()));
        assert_eq!(cbuf.position(), 0);
        assert_eq!(ChunkReader::ReadSize, state);
    }

    #[test]
    fn test_chunked_reader_lastchunk() {
        let data = "0\r\n";
        let mut buf = BytesMut::from(data.as_bytes());
        let mut cbuf = Cursor::new(&mut buf);
        let verify = cbuf.as_ref().into();
        let mut state = ChunkReader::ReadSize;
        let result = state.next(&mut cbuf);
        assert_eq!(result.unwrap(), ChunkedBody::LastChunk(verify));
        assert_eq!(cbuf.position(), cbuf.len());
        assert_eq!(ChunkReader::LastChunk, state);
    }

    #[test]
    fn test_chunked_reader_trailer() {
        let data = "key: value\r\n\r\n";
        let mut state = ChunkReader::ReadTrailers;
        let mut buf = BytesMut::from(data.as_bytes());
        let mut cbuf = Cursor::new(&mut buf);
        let result = state.next(&mut cbuf);
        assert_eq!(cbuf.position(), cbuf.len());
        assert_eq!(ChunkReader::End, state);
        assert!(matches!(result.unwrap(), ChunkedBody::Trailers(_)));
    }

    #[test]
    fn test_chunked_reader_trailer_incremental() {
        let data = "key: value";
        let mut buf = BytesMut::from(data.as_bytes());
        let mut cbuf = Cursor::new(&mut buf);
        let mut state = ChunkReader::ReadTrailers;
        let result = state.next(&mut cbuf);
        assert_eq!(result, None);
        cbuf.as_mut().put_slice(b"\r\n");
        let result = state.next(&mut cbuf);
        assert_eq!(result, None);
        cbuf.as_mut().put_slice(b"\r\n");
        let result = state.next(&mut cbuf);
        assert!(matches!(result.unwrap(), ChunkedBody::Trailers(_)));
        assert_eq!(cbuf.position(), cbuf.len());
        assert_eq!(ChunkReader::End, state);
    }

    #[test]
    fn test_chunked_reader_no_trailer() {
        let data = "\r\n";
        let mut buf = BytesMut::from(data.as_bytes());
        let mut cbuf = Cursor::new(&mut buf);
        let mut state = ChunkReader::ReadTrailers;
        let result = state.next(&mut cbuf);
        assert!(matches!(result.unwrap(), ChunkedBody::EndCRLF(_)));
        assert_eq!(cbuf.position(), cbuf.len());
        assert_eq!(ChunkReader::End, state);
    }

    #[test]
    fn test_chunked_reader_read_full() {
        let data = "7; hola amigo\r\n\
                   Mozilla\r\n\
                   9\r\n\
                   Developer\r\n\
                   7\r\n\
                   Network\r\n\
                   0\r\n\
                   a: b\r\n\
                   c: d\r\n\
                   \r\n";
        let mut buf = BytesMut::from(data.as_bytes());
        let mut cbuf = Cursor::new(&mut buf);
        let mut state = ChunkReader::ReadSize;
        // Poll 1 Read Chunk
        match state.next(&mut cbuf) {
            Some(ChunkedBody::Size(data)) => {
                assert_eq!(0, cbuf.position());
                assert_eq!(ChunkReader::ReadChunk(9), state);
                let verify = BytesMut::from("7; hola amigo\r\n");
                assert_eq!(data, verify);
            }
            _ => {
                panic!()
            }
        }
        //dbg!(String::from_utf8_lossy(&cbuf.remaining()));
        // Poll 2 Read Size
        match state.next(&mut cbuf) {
            Some(ChunkedBody::Chunk(data)) => {
                assert_eq!(0, cbuf.position());
                assert_eq!(ChunkReader::ReadSize, state);
                let verify = BytesMut::from("Mozilla\r\n");
                assert_eq!(data, verify);
            }
            _ => {
                panic!()
            }
        }
        //dbg!(String::from_utf8_lossy(&cbuf.remaining()));
        // Poll 3 Read Chunk
        match state.next(&mut cbuf) {
            Some(ChunkedBody::Size(data)) => {
                assert_eq!(0, cbuf.position());
                assert_eq!(ChunkReader::ReadChunk(11), state);
                let verify = BytesMut::from("9\r\n");
                assert_eq!(data, verify);
            }
            _ => {
                panic!()
            }
        }
        //dbg!(String::from_utf8_lossy(&cbuf.remaining()));
        // Poll 4
        match state.next(&mut cbuf) {
            Some(ChunkedBody::Chunk(data)) => {
                assert_eq!(0, cbuf.position());
                assert_eq!(ChunkReader::ReadSize, state);
                let verify = BytesMut::from("Developer\r\n");
                assert_eq!(data, verify);
            }
            _ => {
                panic!()
            }
        }
        // Poll 5 Read Size
        match state.next(&mut cbuf) {
            Some(ChunkedBody::Size(data)) => {
                assert_eq!(0, cbuf.position());
                assert_eq!(ChunkReader::ReadChunk(9), state);
                let verify = BytesMut::from("7\r\n");
                assert_eq!(data, verify);
            }
            _ => {
                panic!()
            }
        }
        // Poll 6
        match state.next(&mut cbuf) {
            Some(ChunkedBody::Chunk(data)) => {
                assert_eq!(0, cbuf.position());
                assert_eq!(ChunkReader::ReadSize, state);
                let verify = BytesMut::from("Network\r\n");
                assert_eq!(data, verify);
            }
            _ => {
                panic!()
            }
        }
        // Poll 7
        match state.next(&mut cbuf) {
            Some(ChunkedBody::LastChunk(data)) => {
                assert_eq!(0, cbuf.position());
                assert_eq!(ChunkReader::LastChunk, state);
                let verify = BytesMut::from("0\r\n");
                assert_eq!(data, verify);
            }
            _ => {
                panic!()
            }
        }
        // Poll 8 - If polled in Last Chunk State Error
        match state.next(&mut cbuf) {
            Some(ChunkedBody::LastChunk(data)) => {
                assert_eq!(0, cbuf.position());
                assert_eq!(ChunkReader::Failed(ChunkReaderError::LastChunkPoll), state);
                let verify = BytesMut::from("");
                assert_eq!(data, verify);
            }
            _ => {
                panic!()
            }
        }
        // Poll 9
        state = ChunkReader::ReadTrailers;
        match state.next(&mut cbuf) {
            Some(ChunkedBody::Trailers(data)) => {
                assert_eq!(0, cbuf.position());
                assert_eq!(ChunkReader::End, state);
                let buf = "a: b\r\n\
                       c: d\r\n\r\n";
                let verify = HeaderMap::new(BytesMut::from(buf));
                assert_eq!(data, verify);
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn test_chunked_reader_read_partial() {
        let data = "4\r\n\
                        Wiki\r\n\
                        6\r\n\
                        pedia \r\n\
                        E\r\n\
                        in";
        let remain = " \r\n\
                            \r\n\
                            chunks.\r\n\
                            0; hola\r\n\
                            \r\n";
        let mut buf = BytesMut::from(data);
        let mut cbuf = Cursor::new(&mut buf);
        let mut state = ChunkReader::ReadSize;
        loop {
            match state.next(&mut cbuf) {
                Some(_) => match state {
                    ChunkReader::LastChunk => break,
                    _ => continue,
                },
                None => {
                    assert_eq!(state, ChunkReader::ReadChunk(14));
                    cbuf.as_mut().put_slice(remain.as_bytes());
                    continue;
                }
            }
        }

        assert_eq!(state, ChunkReader::LastChunk);
        state = ChunkReader::EndCRLF;
        state.next(&mut cbuf);
        assert_eq!(state, ChunkReader::End);
        assert_eq!(cbuf.position(), cbuf.len());
    }

    #[test]
    fn test_chunked_reader_read_loop() {
        let data = "7; hola amigo\r\n\
                            Mozilla\r\n\
                            9\r\n\
                            Developer\r\n\
                            7\r\n\
                            Network\r\n\
                            0\r\n\
                            a: b\r\n\
                            c: d\r\n\
                            \r\n";
        let mut buf = BytesMut::from(data);
        let mut cbuf = Cursor::new(&mut buf);
        let mut state = ChunkReader::ReadSize;
        loop {
            match state.next(&mut cbuf) {
                Some(_) => match state {
                    ChunkReader::LastChunk => state = ChunkReader::ReadTrailers,
                    ChunkReader::End => {
                        break;
                    }
                    _ => continue,
                },
                None => panic!(),
            }
        }
        assert_eq!(cbuf.remaining().len(), 0);
        assert_eq!(state, ChunkReader::End);
    }

    #[test]
    fn test_get_size() {
        let data = "7\r\n";
        let mut buf = BytesMut::from(data);
        let mut cbuf = Cursor::new(&mut buf);
        let result = ChunkReader::mark_size_chunk(&mut cbuf);
        assert!(result);
        let size = ChunkReader::get_size(&mut cbuf).unwrap();
        assert_eq!(size, 7);
    }

    #[test]
    fn test_chunked_reader_extra_data() {
        let data = "\r\n\
                    extra data";
        let mut buf = BytesMut::from(data);
        let mut cbuf = Cursor::new(&mut buf);
        let mut state = ChunkReader::EndCRLF;
        let chunk = state.next(&mut cbuf).unwrap();
        assert!(matches!(chunk, ChunkedBody::EndCRLF(_)));
        assert_eq!(state, ChunkReader::End);
        assert_eq!(cbuf.remaining(), &b"extra data"[..]);
    }
}
