use crate::{header_map::HeaderMap, info_line::InfoLine};
use bytes::BytesMut;
mod try_from_bytes;

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
    pub fn into_data(self) -> BytesMut {
        let mut data = self.info_line.into_data();
        data.unsplit(self.header_map.into_data());
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
