pub mod request;
pub mod response;
use bytes::BytesMut;
pub mod error;
use error::*;

/* Description:
 *      Trait for parsing info line of request and response.
 *
 * Implemented in:
 *      request.rs
 *      response.rs
 */

pub trait InfoLine {
    fn build_infoline(raw: BytesMut) -> Result<Self, InfoLineError>
    where
        Self: Sized;
    fn into_data(self) -> BytesMut;
}
