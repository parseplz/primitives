#![allow(warnings, unused)]

use bytes::BytesMut;

use crate::state::DecodeState;
pub mod chunked;
pub mod content_length;
pub use decompression::multi::error::MultiDecompressErrorReason;
pub mod decode_struct;
mod decompress_trait;
mod decompression;
pub use decompress_trait::DecompressTrait;
pub mod state;

pub fn decompress<T>(
    message: &mut T,
    buf: &mut BytesMut,
) -> Result<(), MultiDecompressErrorReason>
where
    T: DecompressTrait + std::fmt::Debug,
{
    let mut state = DecodeState::init(message, buf);
    loop {
        state = state.try_next()?;
        if state.is_ended() {
            break;
        }
    }

    Ok(())
}
