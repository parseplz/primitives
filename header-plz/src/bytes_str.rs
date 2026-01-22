use bytes::Bytes;

use std::str;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BytesStr(Bytes);

impl BytesStr {
    pub(crate) fn new() -> Self {
        Self(Bytes::new())
    }

    pub const fn from_static(value: &'static str) -> Self {
        BytesStr(Bytes::from_static(value.as_bytes()))
    }

    pub fn unchecked_from_slice(value: &[u8]) -> Self {
        BytesStr(Bytes::copy_from_slice(value))
    }

    pub(crate) fn as_str(&self) -> &str {
        // Safety: check valid utf-8 in constructor
        unsafe { std::str::from_utf8_unchecked(self.0.as_ref()) }
    }

    pub fn into_inner(self) -> Bytes {
        self.0
    }

    pub(crate) fn from_utf8(
        bytes: Bytes,
    ) -> Result<Self, std::str::Utf8Error> {
        str::from_utf8(&bytes)?;
        // Invariant: just checked is utf8
        Ok(BytesStr(bytes))
    }

    #[inline]
    /// ## Panics
    /// In a debug build this will panic if `bytes` is not valid UTF-8.
    ///
    /// ## Safety
    /// `bytes` must contain valid UTF-8. In a release build it is undefined
    /// behavior to call this with `bytes` that is not valid UTF-8.
    pub unsafe fn from_utf8_unchecked(bytes: Bytes) -> BytesStr {
        if cfg!(debug_assertions) {
            match str::from_utf8(&bytes) {
                Ok(_) => (),
                Err(err) => panic!(
                    "ByteStr::from_utf8_unchecked() with invalid bytes| error = {}, bytes = {:?}",
                    err, bytes
                ),
            }
        }
        // Invariant: assumed by the safety requirements of this function.
        BytesStr(bytes)
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

impl TryFrom<&[u8]> for BytesStr {
    type Error = str::Utf8Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(BytesStr::from(str::from_utf8(value)?))
    }
}

impl From<Bytes> for BytesStr {
    fn from(value: Bytes) -> Self {
        BytesStr(value)
    }
}
