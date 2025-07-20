use body_plz::variants::Body;
use header_plz::{Header, HeaderMap, body_headers::BodyHeader};

pub trait DecompressTrait {
    fn get_body(&mut self) -> Body;

    fn body_headers_as_mut(&mut self) -> &mut Option<BodyHeader>;

    fn header_map(&self) -> &HeaderMap;

    fn header_map_as_mut(&mut self) -> &mut HeaderMap;

    fn has_header_key(&self, key: &str) -> Option<usize> {
        self.header_map().header_key_position(key)
    }

    fn update_header_value_on_position(&mut self, pos: usize, value: &str) {
        self.header_map_as_mut().update_header_value_on_position(pos, value);
    }

    fn add_header(&mut self, key: &str, value: &str) {
        let header: Header = (key, value).into();
        self.header_map_as_mut().add_header(header);
    }

    fn set_body(&mut self, body: Body);

    //
    //
    //fn remove_header_on_position(&mut self, position: usize); // depends - header_map_as_mut
    //
    //fn update_header_value_on_position(
    //    &mut self,
    //    position: usize,
    //    value: &str,
    //); // depends - header_map_as_mut
}
