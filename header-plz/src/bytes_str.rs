use bytes::Bytes;

use std::{fmt, ops, str};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BytesStr(Bytes);

impl BytesStr {
    pub const fn from_static(value: &'static str) -> Self {
        BytesStr(Bytes::from_static(value.as_bytes()))
    }

    pub fn unchecked_from_slice(value: &[u8]) -> Self {
        BytesStr(Bytes::copy_from_slice(value))
    }

    pub fn try_from(bytes: Bytes) -> Result<Self, std::str::Utf8Error> {
        std::str::from_utf8(bytes.as_ref())?;
        Ok(BytesStr(bytes))
    }

    pub(crate) fn as_str(&self) -> &str {
        // Safety: check valid utf-8 in constructor
        unsafe { std::str::from_utf8_unchecked(self.0.as_ref()) }
    }

    pub(crate) fn into_inner(self) -> Bytes {
        self.0
    }
}

impl From<&str> for BytesStr {
    fn from(value: &str) -> Self {
        BytesStr(Bytes::copy_from_slice(value.as_bytes()))
    }
}

impl std::ops::Deref for BytesStr {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for BytesStr {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
