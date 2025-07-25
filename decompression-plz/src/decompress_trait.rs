use body_plz::variants::Body;
use bytes::BytesMut;
use header_plz::{Header, HeaderMap, body_headers::BodyHeader};

pub trait DecompressTrait {
    // Body
    fn get_body(&mut self) -> Body;

    fn get_extra_body(&mut self) -> Option<BytesMut>;

    fn set_body(&mut self, body: Body);

    fn body_headers_as_mut(&mut self) -> &mut Option<BodyHeader>;

    /// header
    fn header_map(&self) -> &HeaderMap;

    fn header_map_as_mut(&mut self) -> &mut HeaderMap;

    // entire header
    fn add_header(&mut self, key: &str, value: &str) {
        self.header_map_as_mut().add_header(Header::from((key, value)));
    }

    fn add_multi_headers(&mut self, mut headers: Vec<Header>) {
        self.header_map_as_mut().headers_as_mut().append(&mut headers);
    }
    fn remove_header_on_position(&mut self, position: usize) {
        self.header_map_as_mut().remove_header_on_position(position);
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

    fn truncate_header_value_on_position<T>(
        &mut self,
        pos: usize,
        truncate_at: T,
    ) where
        T: AsRef<str>,
    {
        self.header_map_as_mut()
            .truncate_header_value_on_position(pos, truncate_at);
    }

    //fn update_header_value_on_position(
    //    &mut self,
    //    position: usize,
    //    value: &str,
    //); // depends - header_map_as_mut
}
