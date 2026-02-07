use bytes::Bytes;

use crate::{
    abnf::{FORWARD_SLASH, FRAGMENT, QMARK},
    bytes_str::BytesStr,
    uri::{builder::Builder, path::PathAndQuery, scheme::Scheme},
};
use std::{convert::Infallible, str::FromStr};
mod builder;
pub mod path;
pub mod scheme;

/*

abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
|-|   |-------------------------------||--------| |-------------------| |-----|
 |                  |                       |               |              |
scheme          authority                 path            query         fragment

*/

#[derive(Debug)]
pub enum InvalidUri {
    InvalidScheme,
    InvalidPath,
    InvalidFormat,
    Authority,
    Empty,
}

impl From<Infallible> for InvalidUri {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Clone, Debug)]
pub struct Uri {
    pub(crate) scheme: Scheme,
    pub(crate) authority: BytesStr,
    pub(crate) path_and_query: PathAndQuery,
}

impl Default for Uri {
    #[inline]
    fn default() -> Uri {
        Uri {
            scheme: Scheme::empty(),
            authority: BytesStr::new(),
            path_and_query: PathAndQuery::slash(),
        }
    }
}

impl Uri {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn scheme(&self) -> Option<&Scheme> {
        if self.scheme.is_none() {
            None
        } else {
            Some(&self.scheme)
        }
    }

    pub fn path_and_query(&self) -> &PathAndQuery {
        &self.path_and_query
    }

    pub fn path(&self) -> &str {
        self.path_and_query.path()
    }

    pub fn query(&self) -> Option<&str> {
        self.path_and_query.query()
    }

    pub fn authority(&self) -> Option<&str> {
        if self.authority.is_empty() {
            None
        } else {
            Some(&self.authority)
        }
    }

    pub fn has_path(&self) -> bool {
        !self.path_and_query.data.is_empty() || !self.scheme.is_none()
    }

    pub fn into_parts(self) -> (Scheme, BytesStr, PathAndQuery) {
        (self.scheme, self.authority, self.path_and_query)
    }

    pub fn from_shared(s: Bytes) -> Result<Uri, InvalidUri> {
        use InvalidUri::*;

        match s.len() {
            0 => {
                return Err(Empty);
            }
            1 => match s[0] {
                b'/' => {
                    return Ok(Uri {
                        scheme: Scheme::empty(),
                        authority: BytesStr::new(),
                        path_and_query: PathAndQuery::slash(),
                    });
                }
                b'*' => {
                    return Ok(Uri {
                        scheme: Scheme::empty(),
                        authority: BytesStr::new(),
                        path_and_query: PathAndQuery::star(),
                    });
                }
                _ => {
                    let authority =
                        unsafe { BytesStr::from_utf8_unchecked(s) };
                    return Ok(Uri {
                        scheme: Scheme::empty(),
                        authority,
                        path_and_query: PathAndQuery::empty(),
                    });
                }
            },
            _ => {}
        }

        if s[0] == b'/' {
            return Ok(Uri {
                scheme: Scheme::empty(),
                authority: BytesStr::new(),
                path_and_query: PathAndQuery::from_shared(s)?,
            });
        }

        parse_full(s)
    }
}

fn parse_full(mut s: Bytes) -> Result<Uri, InvalidUri> {
    // Parse the scheme
    let scheme = match Scheme::parse(&s[..]) {
        Scheme::None => Scheme::None,
        Scheme::Standard(p) => {
            // TODO: use truncate
            let _ = s.split_to(p.len() + 3);
            Scheme::Standard(p)
        }
        Scheme::Other(n) => {
            // Grab the protocol
            let mut scheme = s.split_to(n + 3);

            // Strip ://, TODO: truncate
            let _ = scheme.split_off(n);

            // Allocate the ByteStr
            let val = unsafe { BytesStr::from_utf8_unchecked(scheme) };

            Scheme::Other(Box::new(val))
        }
    };

    // Find the end of the authority. The scheme will already have been
    // extracted.

    let authority_end = s
        .as_ref()
        .iter()
        .position(|&b| b == FORWARD_SLASH || b == QMARK || b == FRAGMENT)
        .unwrap_or(s.len());

    if scheme.is_none() {
        if authority_end != s.len() {
            return Err(InvalidUri::InvalidFormat);
        }

        let authority = unsafe { BytesStr::from_utf8_unchecked(s) };

        return Ok(Uri {
            scheme,
            authority,
            path_and_query: PathAndQuery::empty(),
        });
    }

    // Authority is required when absolute
    if authority_end == 0 {
        return Err(InvalidUri::InvalidFormat);
    }

    let authority = s.split_to(authority_end);
    let authority = unsafe { BytesStr::from_utf8_unchecked(authority) };

    Ok(Uri {
        scheme,
        authority,
        path_and_query: PathAndQuery::from_shared(s)?,
    })
}

impl PartialEq for Uri {
    fn eq(&self, other: &Uri) -> bool {
        if self.scheme() != other.scheme() {
            return false;
        }

        if self.authority() != other.authority() {
            return false;
        }

        if self.path() != other.path() {
            return false;
        }

        if self.query() != other.query() {
            return false;
        }

        true
    }
}

impl FromStr for Uri {
    type Err = InvalidUri;

    #[inline]
    fn from_str(s: &str) -> Result<Uri, InvalidUri> {
        Uri::try_from(s.as_bytes())
    }
}

impl<'a> TryFrom<&'a [u8]> for Uri {
    type Error = InvalidUri;

    #[inline]
    fn try_from(t: &'a [u8]) -> Result<Self, Self::Error> {
        Uri::from_shared(Bytes::copy_from_slice(t))
    }
}

impl<'a> TryFrom<&'a str> for Uri {
    type Error = InvalidUri;

    #[inline]
    fn try_from(t: &'a str) -> Result<Self, Self::Error> {
        t.parse()
    }
}

impl<'a> TryFrom<&'a String> for Uri {
    type Error = InvalidUri;

    #[inline]
    fn try_from(t: &'a String) -> Result<Self, Self::Error> {
        t.parse()
    }
}

impl TryFrom<String> for Uri {
    type Error = InvalidUri;

    #[inline]
    fn try_from(t: String) -> Result<Self, Self::Error> {
        Uri::from_shared(Bytes::from(t))
    }
}

impl TryFrom<Vec<u8>> for Uri {
    type Error = InvalidUri;

    #[inline]
    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        Uri::from_shared(Bytes::from(vec))
    }
}

impl PartialEq<str> for Uri {
    fn eq(&self, other: &str) -> bool {
        let mut other = other.as_bytes();
        let mut absolute = false;

        if let Some(scheme) = self.scheme() {
            let scheme = scheme.as_str().as_bytes();
            absolute = true;

            if other.len() < scheme.len() + 3 {
                return false;
            }

            if !scheme.eq_ignore_ascii_case(&other[..scheme.len()]) {
                return false;
            }

            other = &other[scheme.len()..];

            if &other[..3] != b"://" {
                return false;
            }

            other = &other[3..];
        }

        if let Some(auth) = self.authority() {
            let len = auth.len();
            absolute = true;

            if other.len() < len {
                return false;
            }

            if !auth.as_bytes().eq_ignore_ascii_case(&other[..len]) {
                return false;
            }

            other = &other[len..];
        }

        let path = self.path();

        if other.len() < path.len() || path.as_bytes() != &other[..path.len()]
        {
            if absolute && path == "/" {
                // PathAndQuery can be omitted, fall through
            } else {
                return false;
            }
        } else {
            other = &other[path.len()..];
        }

        if let Some(query) = self.query() {
            if other.is_empty() {
                return query.is_empty();
            }

            if other[0] != b'?' {
                return false;
            }

            other = &other[1..];

            if other.len() < query.len() {
                return false;
            }

            if query.as_bytes() != &other[..query.len()] {
                return false;
            }

            other = &other[query.len()..];
        }

        other.is_empty() || other[0] == b'#'
    }
}

impl PartialEq<Uri> for str {
    fn eq(&self, uri: &Uri) -> bool {
        uri == self
    }
}

impl<'a> PartialEq<&'a str> for Uri {
    fn eq(&self, other: &&'a str) -> bool {
        self == *other
    }
}

impl PartialEq<Uri> for &str {
    fn eq(&self, uri: &Uri) -> bool {
        uri == *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_uri_origin_form() {
        let uri = Uri::from_shared(Bytes::from("/path/to/resource")).unwrap();
        assert_eq!(uri.path(), "/path/to/resource");
        assert!(uri.scheme().is_none());
        assert!(uri.authority().is_none());
        assert!(uri.query().is_none());

        let uri = Uri::from_shared(Bytes::from("/path?query=1")).unwrap();
        assert_eq!(uri.path(), "/path");
        assert_eq!(uri.query(), Some("query=1"));

        let uri =
            Uri::from_shared(Bytes::from("http://example.com/path?query=1"))
                .unwrap();
        assert_eq!(uri.authority().unwrap(), "example.com");
        assert_eq!(uri.path(), "/path");
        assert_eq!(uri.query(), Some("query=1"));
    }

    #[test]
    fn test_uri_absolute_form_http() {
        let uri =
            Uri::from_shared(Bytes::from("http://example.com/foo")).unwrap();
        assert_eq!(uri.scheme().unwrap().as_str(), "http");
        assert_eq!(uri.authority().unwrap(), "example.com");
        assert_eq!(uri.path(), "/foo");
    }

    #[test]
    fn test_uri_absolute_form_https_with_port() {
        let uri = Uri::from_shared(Bytes::from("https://example.com:8080/"))
            .unwrap();
        assert_eq!(uri.scheme().unwrap().as_str(), "https");
        assert_eq!(uri.authority().unwrap(), "example.com:8080");
        assert_eq!(uri.path(), "/");
    }

    #[test]
    fn test_uri_absolute_form_no_path() {
        let uri = Uri::from_shared(Bytes::from("http://example.com")).unwrap();
        assert_eq!(uri.path(), "/");
        assert_eq!(uri.authority().unwrap(), "example.com");
    }

    #[test]
    fn test_uri_authority_form() {
        let uri = Uri::from_shared(Bytes::from("example.com:443")).unwrap();
        assert!(uri.scheme().is_none());
        assert_eq!(uri.authority().unwrap(), "example.com:443");
        assert!(uri.path_and_query.data.is_empty());
    }

    #[test]
    fn test_uri_asterisk_form() {
        let uri = Uri::from_shared(Bytes::from("*")).unwrap();
        assert!(uri.scheme().is_none());
        assert!(uri.authority().is_none());
        assert_eq!(uri.path(), "*");
    }

    #[test]
    fn test_uri_custom_scheme() {
        let uri = Uri::from_shared(Bytes::from("my-scheme://data")).unwrap();
        assert_eq!(uri.scheme().unwrap().as_str(), "my-scheme");
        assert_eq!(uri.authority().unwrap(), "data");
    }

    #[test]
    fn test_uri_eq_str() {
        let uri =
            Uri::from_shared(Bytes::from("http://example.com/foo")).unwrap();

        assert_eq!(uri, "http://example.com/foo");
        assert_eq!(uri, "HTTP://example.com/foo");
        assert_eq!(uri, "http://EXAMPLE.COM/foo");
        assert_ne!(uri, "http://example.com/FOO");
        assert_ne!(uri, "http://other.com/foo");
    }

    #[test]
    fn test_uri_try_from_strings() {
        let uri: Uri = "http://localhost:3000/api".try_into().unwrap();
        assert_eq!(uri.authority().unwrap(), "localhost:3000");

        let uri_string = String::from("/relative/path");
        let uri: Uri = uri_string.try_into().unwrap();
        assert_eq!(uri.path(), "/relative/path");
    }

    #[test]
    fn test_invalid_uris() {
        assert!(matches!(
            Uri::from_shared(Bytes::from("")),
            Err(InvalidUri::Empty)
        ));

        assert!(matches!(
            Uri::from_shared(Bytes::from("example.com/foo")),
            Err(InvalidUri::InvalidFormat)
        ));
    }

    #[test]
    fn test_parse_full_fallback() {
        let uri = Uri::from_shared(Bytes::from("a")).unwrap();
        assert_eq!(uri.authority().unwrap(), "a");
        assert!(uri.scheme().is_none());
    }

    #[test]
    fn test_uri_complex_authority() {
        let uri =
            Uri::from_shared(Bytes::from("http://user:pass@example.com/foo"))
                .unwrap();
        assert_eq!(uri.authority().unwrap(), "user:pass@example.com");

        let uri =
            Uri::from_shared(Bytes::from("http://[::1]:8080/foo")).unwrap();
        assert_eq!(uri.authority().unwrap(), "[::1]:8080");

        let uri = Uri::from_shared(Bytes::from("http://[::1]/")).unwrap();
        assert_eq!(uri.authority().unwrap(), "[::1]");
        assert_eq!(uri.path(), "/");

        let uri = Uri::from_shared(Bytes::from("[::1]")).unwrap();
        assert_eq!(uri.authority().unwrap(), "[::1]");
        assert!(uri.scheme().is_none());
    }

    #[test]
    fn test_uri_fragment_handling() {
        let uri =
            Uri::from_shared(Bytes::from("http://example.com#frag")).unwrap();
        assert_eq!(uri.authority().unwrap(), "example.com");
        assert_eq!(uri.path(), "/");
        let uri = Uri::from_shared(Bytes::from("/path#frag")).unwrap();
        assert_eq!(uri.path(), "/path");
    }

    #[test]
    #[ignore]
    fn test_uri_scheme_single_colon() {
        let uri =
            Uri::from_shared(Bytes::from("mailto:user@example.com")).unwrap();
        assert_eq!(uri.scheme().unwrap().as_str(), "mailto");
        assert!(uri.authority().unwrap().contains("user@example.com"));
    }
}
