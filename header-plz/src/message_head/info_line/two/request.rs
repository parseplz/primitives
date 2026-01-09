use crate::{methods::Method, uri::Uri};

#[derive(Debug, Default)]
pub struct RequestLine {
    pub method: Method,
    pub uri: Uri,
    extension: Option<Box<Bytes>>,
}
    pub fn extension(&self) -> Option<&Bytes> {
        self.extension.as_deref()
    }
