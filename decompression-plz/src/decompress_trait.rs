use body_plz::variants::Body;
use bytes::BytesMut;
use header_plz::{
    body_headers::BodyHeader,
    message_head::header_map::{HMap, Hmap},
};

pub trait DecompressTrait {
    type HmapType: Hmap + std::fmt::Debug + for<'a> From<(&'a str, &'a str)>;

    // Body
    fn get_body(&mut self) -> Body;

    fn get_extra_body(&mut self) -> Option<BytesMut>;

    fn set_body(&mut self, body: Body);

    fn body_headers(&self) -> Option<&BodyHeader>;

    fn body_headers_as_mut(&mut self) -> Option<&mut BodyHeader>;

    /// header
    fn header_map(&self) -> &HMap<Self::HmapType>;

    fn header_map_as_mut(&mut self) -> &mut HMap<Self::HmapType>;

    fn insert_header(&mut self, key: &str, value: &str) {
        self.header_map_as_mut().insert(key, value);
    }

    fn extend<I>(&mut self, headers: I)
    where
        I: IntoIterator<Item = Self::HmapType>,
    {
        self.header_map_as_mut().extend(headers);
    }

    fn remove_header_on_position(&mut self, pos: usize) {
        self.header_map_as_mut().remove_header_on_position(pos);
    }

    // key
    fn has_header_key(&self, key: &str) -> Option<usize> {
        self.header_map().header_key_position(key)
    }

    fn remove_header_on_key(&mut self, key: &str) -> bool {
        self.header_map_as_mut().remove_header_on_key(key)
    }

    // value
    fn update_header_value_on_position(&mut self, pos: usize, value: &str) {
        self.header_map_as_mut().update_header_value_on_position(pos, value);
    }

    fn update_header_value_on_position_multiple_values(
        &mut self,
        pos: usize,
        value: impl Iterator<Item: AsRef<[u8]>>,
    ) {
        self.header_map_as_mut()
            .update_header_value_on_position_multiple_values(pos, value);
    }

    fn truncate_header_value_on_position<T>(
        &mut self,
        pos: usize,
        truncate_at: T,
    ) where
        T: AsRef<str>,
    {
        self.header_map_as_mut()
            .truncate_header_value_at_position(pos, truncate_at);
    }
}
