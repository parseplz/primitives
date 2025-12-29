// Require the scheme to not be too long in order to enable further
// optimizations later.
// const MAX_SCHEME_LEN: usize = 64;

use std::{fmt, str::FromStr};

use crate::{bytes_str::BytesStr, uri::InvalidUri};

#[derive(Debug, Clone)]
pub enum Scheme<T = Box<BytesStr>> {
    None,
    Standard(Protocol),
    Other(T),
}

impl Scheme {
    pub const HTTP: Scheme = Scheme::Standard(Protocol::Http);
    pub const HTTPS: Scheme = Scheme::Standard(Protocol::Https);
    pub const EMPTY: Scheme = Scheme::None;

    pub fn parse_exact(value: &[u8]) -> Self {
        match value {
            b"http" => Protocol::Http.into(),
            b"https" => Protocol::Https.into(),
            _ => {
                // TODO: needed ?
                //if s.len() > MAX_SCHEME_LEN {
                //    return Err(ErrorKind::SchemeTooLong.into());
                //}
                Scheme::Other(BytesStr::unchecked_from_slice(value).into())
            }
        }
    }

    pub fn as_str(&self) -> &str {
        use self::Protocol::*;
        use self::Scheme::*;

        match self {
            Standard(Http) => "http",
            Standard(Https) => "https",
            Other(v) => &v[..],
            None => "",
        }
    }

    pub(crate) fn empty() -> Self {
        Scheme::None
    }

    pub(crate) fn is_none(&self) -> bool {
        matches!(*self, Scheme::None)
    }
}

impl PartialEq for Scheme {
    fn eq(&self, other: &Scheme) -> bool {
        use self::Protocol::*;
        use self::Scheme::*;

        match (self, other) {
            (&Standard(Http), &Standard(Http)) => true,
            (&Standard(Https), &Standard(Https)) => true,
            (Other(a), Other(b)) => a == b,
            (&None, &None) => true,
            _ => false,
        }
    }
}

impl TryFrom<&[u8]> for Scheme {
    type Error = InvalidUri;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match str::from_utf8(value) {
            Ok(v) => Ok(Scheme::parse_exact(v.as_bytes())),
            Err(_) => Err(InvalidUri::InvalidScheme),
        }
    }
}

impl<'a> TryFrom<&'a str> for Scheme {
    type Error = InvalidUri;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        TryFrom::try_from(s.as_bytes())
    }
}

impl FromStr for Scheme {
    type Err = InvalidUri;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TryFrom::try_from(s)
    }
}

impl PartialEq<str> for Scheme {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq_ignore_ascii_case(other)
    }
}

/// Case-insensitive equality
impl PartialEq<Scheme> for str {
    fn eq(&self, other: &Scheme) -> bool {
        other == self
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Protocol {
    Http,
    Https,
}

impl Protocol {
    pub(super) fn len(&self) -> usize {
        match *self {
            Protocol::Http => 4,
            Protocol::Https => 5,
        }
    }
}

impl From<Protocol> for Scheme {
    fn from(src: Protocol) -> Self {
        Scheme::Standard(src)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scheme_eq_to_str() {
        assert_eq!(&scheme("http"), "http");
        assert_eq!(&scheme("https"), "https");
        assert_eq!(&scheme("ftp"), "ftp");
        assert_eq!(&scheme("my+funky+scheme"), "my+funky+scheme");
    }

    fn scheme(s: &str) -> Scheme {
        s.parse().expect(&format!("Invalid scheme: {}", s))
    }

    #[test]
    fn scheme_invalid_is_ok() {
        Scheme::try_from("my_funky_scheme").unwrap();
    }

    #[test]
    fn scheme_invalid_utf_is_err() {
        // Invalid UTF-8
        Scheme::try_from([0xC0].as_ref())
            .expect_err("Unexpectedly valid Scheme");
    }
}
