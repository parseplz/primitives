use body_plz::variants::Body;
use header_plz::{Header, HeaderMap, body_headers::BodyHeader};

pub trait DecompressTrait {
    // Body
    fn get_body(&mut self) -> Body;
    fn set_body(&mut self, body: Body);

    fn body_headers_as_mut(&mut self) -> &mut Option<BodyHeader>;

    /// header
    fn header_map(&self) -> &HeaderMap;

    fn header_map_as_mut(&mut self) -> &mut HeaderMap;

    fn has_header_key(&self, key: &str) -> Option<usize> {
        self.header_map().header_key_position(key)
    }

    fn remove_header_on_position(&mut self, position: usize) {
        self.header_map_as_mut().remove_header_on_position(position);
    }

    fn update_header_value_on_position(&mut self, pos: usize, value: &str) {
        self.header_map_as_mut().update_header_value_on_position(pos, value);
    }

    fn add_header(&mut self, key: &str, value: &str) {
        let header: Header = (key, value).into();
        self.header_map_as_mut().add_header(header);
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
