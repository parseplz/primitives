use bytes::Bytes;

use crate::{methods::Method, uri::Uri};

#[derive(Debug, Default)]
pub struct RequestLine {
    method: Method,
    uri: Uri,
    extension: Option<Box<Bytes>>,
}

impl RequestLine {
    pub fn into_parts(self) -> (Method, Uri) {
        (self.method, self.uri)
    }

    pub fn extension(&self) -> Option<&Bytes> {
        self.extension.as_deref()
    }
}
