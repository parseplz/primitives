use bytes::Bytes;

use crate::{method::Method, uri::Uri};

#[derive(Debug, Default, PartialEq)]
pub struct RequestLine {
    method: Method,
    uri: Uri,
    extension: Option<Box<Bytes>>,
}

impl RequestLine {
    pub fn new(method: Method, uri: Uri) -> Self {
        Self {
            method,
            uri,
            ..Default::default()
        }
    }

    pub fn into_parts(self) -> (Method, Uri, Option<Box<Bytes>>) {
        (self.method, self.uri, self.extension)
    }

    // getters
    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    pub fn extension(&self) -> Option<&Bytes> {
        self.extension.as_deref()
    }

    // setters
    pub fn set_method(&mut self, method: Method) {
        self.method = method
    }

    pub fn set_uri(&mut self, uri: Uri) {
        self.uri = uri
    }

    pub fn set_extension(&mut self, ext: Bytes) {
        self.extension = Some(Box::new(ext));
    }
}
