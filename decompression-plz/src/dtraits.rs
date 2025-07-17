use body_plz::variants::Body;
use header_plz::{HeaderMap, body_headers::BodyHeader};

pub trait DecompressTrait {
    fn get_body(&mut self) -> Body;
    fn body_headers_as_mut(&mut self) -> &mut Option<BodyHeader>;

    //
    fn header_map_as_mut(&mut self) -> &mut HeaderMap;

    fn remove_header_on_position(&mut self, position: usize); // depends - header_map_as_mut

    fn update_header_value_on_position(
        &mut self,
        position: usize,
        value: &str,
    ); // depends - header_map_as_mut
}
