pub mod request;
pub mod response;
use bytes::{Buf, BytesMut};
pub mod error;
use error::*;

// Trait for parsing info line of request and response.
pub trait InfoLine {
    fn try_build_infoline(raw: BytesMut) -> Result<Self, InfoLineError>
    where
        Self: Sized;

    fn into_bytes(self) -> BytesMut;

    fn as_chain(&self) -> impl Buf;
}
