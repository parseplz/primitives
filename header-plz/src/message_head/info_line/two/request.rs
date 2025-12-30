use crate::{methods::Method, uri::Uri};

#[derive(Debug, Default)]
pub struct RequestLine {
    pub method: Method,
    pub uri: Uri,
}
