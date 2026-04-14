pub mod error;
pub mod request;
pub mod response;
use crate::Version;
use bytes::{Buf, BytesMut};
use error::*;

// Trait for parsing info line of request and response.
pub trait InfoLine {
    fn try_build_infoline(raw: BytesMut) -> Result<Self, InfoLineError>
    where
        Self: Sized;

    fn into_bytes(self) -> BytesMut;

    fn as_chain(&self) -> impl Buf;

    fn version(&self) -> Option<Version>;
}
