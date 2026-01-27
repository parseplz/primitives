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

    pub fn try_set_scheme<T>(&mut self, scheme: T) -> Result<(), InvalidUri>
    where
        T: TryInto<Scheme>,
        <T as TryInto<Scheme>>::Error: Into<InvalidUri>,
    {
        self.uri.scheme = scheme.try_into().map_err(Into::into)?;
        Ok(())
    }

    pub fn try_set_path<T>(&mut self, path: T) -> Result<(), InvalidUri>
    where
        T: TryInto<PathAndQuery>,
        <T as TryInto<PathAndQuery>>::Error: Into<InvalidUri>,
    {
        self.uri.path_and_query = path.try_into().map_err(Into::into)?;
        Ok(())
    }

    pub fn try_set_authority<T>(
        &mut self,
        authority: T,
    ) -> Result<(), InvalidUri>
    where
        T: TryInto<BytesStr>,
    {
        self.uri.authority =
            authority.try_into().map_err(|_| InvalidUri::Authority)?;
        Ok(())
    }

    pub fn set_extension(&mut self, ext: Bytes) {
        self.extension = Some(Box::new(ext));
    }
}
