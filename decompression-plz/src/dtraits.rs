use header_plz::{HeaderMap, body_headers::BodyHeader};

pub trait DecompressTrait {
    fn body_headers(&mut self) -> &Option<BodyHeader>;

    fn header_map_as_mut(&mut self) -> &mut HeaderMap;

    fn remove_header_on_position(&mut self, position: usize); // depends - header_map_as_mut

    fn update_header_value_on_position(&mut self, position: usize, value: &str); // depends - header_map_as_mut
}
