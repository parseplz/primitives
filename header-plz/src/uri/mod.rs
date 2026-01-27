use crate::{
    bytes_str::BytesStr,
    uri::{builder::Builder, path::PathAndQuery, scheme::Scheme},
};
use std::convert::Infallible;
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
    InvalidAuthority,
    Authority,
    Empty,
}

impl From<Infallible> for InvalidUri {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Debug, Clone)]
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
