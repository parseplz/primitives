use super::*;
use crate::{
    bytes_str::BytesStr,
    uri::{path::PathAndQuery, scheme::Scheme},
};

#[derive(Debug, Default, Clone)]
struct UriParts {
    pub scheme: Option<Scheme>,
    pub authority: Option<BytesStr>,
    pub path_and_query: Option<PathAndQuery>,
}

impl From<Uri> for UriParts {
    fn from(src: Uri) -> Self {
        let path_and_query = if src.has_path() {
            Some(src.path_and_query)
        } else {
            None
        };

        let scheme = match src.scheme {
            Scheme::None => None,
            _ => Some(src.scheme),
        };

        let authority = if src.authority.is_empty() {
            None
        } else {
            Some(src.authority)
        };

        UriParts {
            scheme,
            authority,
            path_and_query,
        }
    }
}

#[derive(Debug)]
pub struct Builder(Result<UriParts, InvalidUri>);

impl Builder {
    #[inline]
    pub fn new() -> Builder {
        Builder(Ok(UriParts::default()))
    }

    pub fn scheme<T>(self, scheme: T) -> Self
    where
        T: TryInto<Scheme>,
        <T as TryInto<Scheme>>::Error: Into<InvalidUri>,
    {
        self.map(move |mut uri| {
            uri.scheme = scheme.try_into().map_err(Into::into)?.into();
            Ok(uri)
        })
    }

    pub fn path_and_query<T>(self, scheme: T) -> Self
    where
        T: TryInto<PathAndQuery>,
        <T as TryInto<PathAndQuery>>::Error: Into<InvalidUri>,
    {
        self.map(move |mut uri| {
            uri.path_and_query = scheme.try_into().map_err(Into::into)?.into();
            Ok(uri)
        })
    }

    pub fn authority<T>(self, authority: T) -> Self
    where
        T: TryInto<BytesStr>,
    {
        self.map(move |mut uri| {
            uri.authority = authority
                .try_into()
                .map_err(|_| InvalidUri::Authority)?
                .into();
            Ok(uri)
        })
    }

    fn map<F>(self, func: F) -> Self
    where
        F: FnOnce(UriParts) -> Result<UriParts, InvalidUri>,
    {
        Builder(self.0.and_then(func))
    }

    fn build(self) -> Result<Uri, InvalidUri> {
        let parts = self.0?;
        let scheme = match parts.scheme {
            Some(scheme) => scheme,
            None => Scheme::None,
        };

        let authority = match parts.authority {
            Some(authority) => authority,
            None => BytesStr::new(),
        };

        let path_and_query = match parts.path_and_query {
            Some(path_and_query) => path_and_query,
            None => PathAndQuery::empty(),
        };
        Ok(Uri {
            scheme,
            authority,
            path_and_query,
        })
    }
}

impl From<Uri> for Builder {
    fn from(uri: Uri) -> Self {
        Self(Ok(uri.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_from_str() {
        let uri = Builder::new()
            .scheme(Scheme::HTTP)
            .authority("hyper.rs")
            .path_and_query("/foo?a=1#23")
            .build()
            .unwrap();
        assert_eq!(uri.scheme(), Some(&Scheme::HTTP));
        assert_eq!(uri.authority().unwrap(), "hyper.rs");
        assert_eq!(uri.path(), "/foo");
        assert_eq!(uri.query(), Some("a=1"));
    }

    #[test]
    fn build_from_string() {
        for i in 1..10 {
            let uri = Builder::new()
                .path_and_query(format!("/foo?a={}#i", i))
                .build()
                .unwrap();
            let expected_query = format!("a={}", i);
            assert_eq!(uri.path(), "/foo");
            assert_eq!(uri.query(), Some(expected_query.as_str()));
        }
    }

    #[test]
    fn build_from_string_ref() {
        for i in 1..10 {
            let p_a_q = format!("/foo?a={}", i);
            let uri = Builder::new().path_and_query(&p_a_q).build().unwrap();
            let expected_query = format!("a={}", i);
            assert_eq!(uri.path(), "/foo");
            assert_eq!(uri.query(), Some(expected_query.as_str()));
        }
    }

    #[test]
    fn build_from_uri() {
        let original_uri = Uri::default();
        let uri = Builder::from(original_uri.clone()).build().unwrap();
        assert_eq!(original_uri, uri);
    }
}
