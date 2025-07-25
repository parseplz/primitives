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
    UpdateContentLengthAndErr(DecodeStruct<'a, T>, MultiDecompressErrorReason),
    End,
}

impl<'a, T> DecodeState<'a, T>
where
    T: DecompressTrait + 'a,
{
    pub fn init(message: &'a mut T, buf: &'a mut BytesMut) -> Self {
        let decode_struct = DecodeStruct::new(message, buf);
        Self::Start(decode_struct)
    }

    pub fn try_next(self) -> Result<Self, MultiDecompressErrorReason> {
        match self {
            DecodeState::Start(mut decode_struct) => {
                let next_state = if decode_struct.transfer_encoding_is_some() {
                    let encodings = decode_struct.transfer_encoding();
                    Self::TransferEncoding(decode_struct, encodings)
                } else if decode_struct.content_encoding_is_some() {
                    let encodings = decode_struct.content_encoding();
                    Self::ContentEncoding(decode_struct, encodings)
                } else if decode_struct.extra_body_is_some() {
                    Self::UpdateContentLength(decode_struct)
                } else {
                    Self::End
                };
                Ok(next_state)
            }
            DecodeState::TransferEncoding(
                mut decode_struct,
                mut encoding_infos,
            ) => {
                // Convert chunked to raw
                // http/1 only
                if decode_struct.is_chunked_te() {
                    decode_struct.chunked_to_raw();
                }
                // TODO: check if only te is chunked
                let next_state = match apply_encoding(
                    &mut decode_struct,
                    &mut encoding_infos,
                ) {
                    Ok(()) if decode_struct.content_encoding_is_some() => {
                        let encodings = decode_struct.content_encoding();
                        Self::ContentEncoding(decode_struct, encodings)
                    }
                    Ok(()) => Self::UpdateContentLength(decode_struct),
                    Err(e) => {
                        // TODO: maybe remove chunked TE
                        if e.is_partial() {
                            Self::UpdateContentLengthAndErr(decode_struct, e)
                        } else {
                            return Err(e);
                        }
                    }
                };
                Ok(next_state)
            }
            DecodeState::ContentEncoding(
                mut decode_struct,
                mut encoding_infos,
            ) => {
                let next_state = if let Err(e) =
                    apply_encoding(&mut decode_struct, &mut encoding_infos)
                {
                    if e.is_partial() {
                        Self::UpdateContentLengthAndErr(decode_struct, e)
                    } else {
                        return Err(e);
                    }
                } else {
                    Self::UpdateContentLength(decode_struct)
                };
                Ok(next_state)
            }
            DecodeState::UpdateContentLength(mut decode_struct) => {
                decode_struct.add_body_and_update_cl();
                Ok(Self::End)
            }
            DecodeState::UpdateContentLengthAndErr(mut decode_struct, e) => {
                decode_struct.add_body_and_update_cl();
                Err(e)
            }
            DecodeState::End => Ok(Self::End),
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
            // whatever the error clear the buf
            decode_struct.buf.clear();
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

#[cfg(test)]
mod tests {

    use header_plz::InfoLine;
    use header_plz::Request;
    use header_plz::body_headers::parse::ParseBodyHeaders;
    use header_plz::message_head::MessageHead;
    use oneone_plz::oneone::OneOne;

    use super::*;

    fn build_oneone<T>(input: &str) -> OneOne<T>
    where
        T: InfoLine,
        MessageHead<T>: ParseBodyHeaders,
    {
        OneOne::try_from(BytesMut::from(input)).unwrap()
    }

    #[test]
    fn test_decode_state_init() {
        let input = "POST / HTTP/1.1\r\n\
                   Content-Length: 11\r\n\r\n\
                   hello world";
        let req: OneOne<Request> = build_oneone(input);
    }
}
