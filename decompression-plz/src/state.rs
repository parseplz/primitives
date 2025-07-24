use std::cmp::Ordering;

use bytes::BytesMut;
use header_plz::body_headers::encoding_info::EncodingInfo;
use tracing::error;

use crate::{
    content_length::add_body_and_update_cl,
    decode_struct::DecodeStruct,
    decompress_trait::DecompressTrait,
    decompression::{
        multi::error::MultiDecompressErrorReason, state::decompression_runner,
    },
};

pub enum DecodeState<'a, T> {
    Start(DecodeStruct<'a, T>),
    TransferEncoding(DecodeStruct<'a, T>, Vec<EncodingInfo>),
    ContentEncoding(DecodeStruct<'a, T>, Vec<EncodingInfo>),
    UpdateContentLength(DecodeStruct<'a, T>),
    End,
}

impl<'a, T> DecodeState<'a, T>
where
    T: DecompressTrait + 'a,
{
    pub fn init(
        message: &'a mut T,
        extra_body: Option<BytesMut>,
        buf: &'a mut BytesMut,
    ) -> Self {
        let decode_struct = DecodeStruct::new(message, extra_body, buf);
        Self::Start(decode_struct)
    }

    pub fn try_next(self) -> Self {
        match self {
            DecodeState::Start(mut decode_struct) => {
                if decode_struct.transfer_encoding_is_some() {
                    let encodings = decode_struct.transfer_encoding();
                    Self::TransferEncoding(decode_struct, encodings)
                } else if decode_struct.content_encoding_is_some() {
                    let encodings = decode_struct.content_encoding();
                    Self::ContentEncoding(decode_struct, encodings)
                } else if decode_struct.extra_body_is_some() {
                    Self::UpdateContentLength(decode_struct)
                } else {
                    Self::End
                }
            }
            DecodeState::TransferEncoding(
                mut decode_struct,
                mut encoding_infos,
            ) => {
                match apply_encoding(&mut decode_struct, &mut encoding_infos) {
                    Ok(()) if decode_struct.content_encoding_is_some() => {
                        let encodings = decode_struct.content_encoding();
                        Self::ContentEncoding(decode_struct, encodings)
                    }
                    Ok(()) => Self::UpdateContentLength(decode_struct),
                    Err(e) => {
                        // TODO: remove chunked TE
                        error!("{}", e);
                        if e.is_partial() {
                            Self::UpdateContentLength(decode_struct)
                        } else {
                            Self::End
                        }
                    }
                }
            }
            DecodeState::ContentEncoding(
                mut decode_struct,
                mut encoding_infos,
            ) => {
                match apply_encoding(&mut decode_struct, &mut encoding_infos) {
                    Err(e) if !e.is_partial() => {
                        error!("{}", e);
                        return Self::End;
                    }
                    _ => {}
                }
                Self::UpdateContentLength(decode_struct)
            }
            DecodeState::UpdateContentLength(mut decode_struct) => {
                let mut body = decode_struct.take_main_body();
                if let Some(extra) = decode_struct.take_extra_body() {
                    body.unsplit(extra);
                }
                add_body_and_update_cl(decode_struct.message, body);
                Self::End
            }
            DecodeState::End => Self::End,
        }
    }

    pub fn is_ended(&self) -> bool {
        matches!(self, DecodeState::End)
    }
}

fn apply_encoding<T>(
    decode_struct: &mut DecodeStruct<T>,
    encoding_info: &mut [EncodingInfo],
) -> Result<(), MultiDecompressErrorReason>
where
    T: DecompressTrait,
{
    match decompression_runner(
        &decode_struct.body,
        decode_struct.extra_body.as_deref(),
        encoding_info,
        decode_struct.buf,
    ) {
        Ok(state) => {
            (decode_struct.body, decode_struct.extra_body) = state.into();
            let iter = encoding_info.iter().map(|einfo| einfo.header_index);
            // remove applied headers
            decode_struct
                .message
                .header_map_as_mut()
                .remove_header_multiple_positions(iter);
            Ok(())
        }
        Err(mut e) => {
            if let MultiDecompressErrorReason::Partial {
                ref mut partial_body,
                header_index,
                compression_index,
            } = e.reason
            {
                decode_struct.body = partial_body.split();
                decode_struct.extra_body = None;
                for einfo in encoding_info.iter().rev() {
                    match einfo.header_index.cmp(&header_index) {
                        Ordering::Less | Ordering::Equal => {
                            let last_failed = einfo
                                .encodings()
                                .iter()
                                .rev()
                                .nth(compression_index)
                                .unwrap();
                            decode_struct
                                .message
                                .truncate_header_value_on_position(
                                    einfo.header_index,
                                    last_failed,
                                );
                            break;
                        }
                        Ordering::Greater => {
                            decode_struct
                                .message
                                .remove_header_on_position(einfo.header_index);
                        }
                    }
                }
            }
            Err(e.reason)
        }
    }
}
