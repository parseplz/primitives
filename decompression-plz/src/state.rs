use body_plz::variants::Body;
use bytes::BytesMut;
use header_plz::body_headers::encoding_info::EncodingInfo;

use crate::{
    decode_struct::DecodeStruct,
    decompress_trait::DecompressTrait,
    decompression::{
        multi::error::MultiDecompressErrorReason, state::decompression_runner,
    },
};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
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
    T: DecompressTrait + 'a + std::fmt::Debug,
{
    pub fn init(message: &'a mut T, buf: &'a mut BytesMut) -> Self {
        Self::Start(DecodeStruct::new(message, buf))
    }

    pub fn try_next(self) -> Result<Self, MultiDecompressErrorReason> {
        match self {
            DecodeState::Start(mut ds) => {
                let next_state = if ds.transfer_encoding_is_some() {
                    let encodings = ds.get_transfer_encoding();
                    Self::TransferEncoding(ds, encodings)
                } else if ds.content_encoding_is_some() {
                    let encodings = ds.get_content_encoding();
                    Self::ContentEncoding(ds, encodings)
                } else if ds.extra_body_is_some() {
                    Self::UpdateContentLength(ds)
                } else {
                    let body = ds.take_main_body();
                    ds.message.set_body(Body::Raw(body));
                    Self::End
                };
                Ok(next_state)
            }
            DecodeState::TransferEncoding(mut ds, mut encoding_infos) => {
                if ds.is_chunked_te() {
                    ds.chunked_to_raw();
                    // remove chunked TE
                    // Chunked TE must be the last
                    let last_info = encoding_infos.last_mut().unwrap();
                    last_info.encodings_as_mut().pop();

                    // if after removing TE it is empty, remove the header
                    if last_info.encodings().is_empty() {
                        ds.message
                            .remove_header_on_position(last_info.header_index);

                        // remove the last encoding_info
                        encoding_infos.pop();
                    }
                }
                // If only chunked was present then Vec<EncodingInfo> is empty
                let mut next_state = if encoding_infos.is_empty() {
                    if ds.content_encoding_is_some() {
                        let encodings = ds.get_content_encoding();
                        Self::ContentEncoding(ds, encodings)
                    } else {
                        Self::UpdateContentLength(ds)
                    }
                } else {
                    match apply_encoding(&mut ds, &mut encoding_infos) {
                        Ok(()) if ds.content_encoding_is_some() => {
                            let encodings = ds.get_content_encoding();
                            Self::ContentEncoding(ds, encodings)
                        }
                        Ok(()) => Self::UpdateContentLength(ds),
                        Err(e) => Self::UpdateContentLengthAndErr(ds, e),
                    }
                };
                next_state.set_transfer_encoding(encoding_infos);
                Ok(next_state)
            }
            DecodeState::ContentEncoding(mut ds, mut encoding_infos) => {
                let mut next_state =
                    match apply_encoding(&mut ds, &mut encoding_infos) {
                        Err(e) => Self::UpdateContentLengthAndErr(ds, e),
                        Ok(_) => Self::UpdateContentLength(ds),
                    };
                next_state.set_content_encoding(encoding_infos);
                Ok(next_state)
            }
            DecodeState::UpdateContentLength(mut ds) => {
                ds.add_body_and_update_cl();
                Ok(Self::End)
            }
            DecodeState::UpdateContentLengthAndErr(mut ds, e) => {
                ds.add_body_and_update_cl();
                Err(e)
            }
            DecodeState::End => Ok(DecodeState::End),
        }
    }

    pub fn is_ended(&self) -> bool {
        matches!(self, DecodeState::End)
    }

    pub fn decode_struct_as_mut(&mut self) -> &mut DecodeStruct<'a, T> {
        match self {
            DecodeState::Start(ds) => ds,
            DecodeState::TransferEncoding(ds, _) => ds,
            DecodeState::ContentEncoding(ds, _) => ds,
            DecodeState::UpdateContentLength(ds) => ds,
            DecodeState::UpdateContentLengthAndErr(ds, _) => ds,
            _ => unreachable!(),
        }
    }

    pub fn set_transfer_encoding(&mut self, te: Vec<EncodingInfo>) {
        let ds_mut = self.decode_struct_as_mut();
        ds_mut.set_transfer_encoding(te);
    }

    pub fn set_content_encoding(&mut self, ce: Vec<EncodingInfo>) {
        let ds_mut = self.decode_struct_as_mut();
        ds_mut.set_content_encoding(ce);
    }
}

fn apply_encoding<T>(
    decode_struct: &mut DecodeStruct<T>,
    encoding_info: &mut [EncodingInfo],
) -> Result<(), MultiDecompressErrorReason>
where
    T: DecompressTrait + std::fmt::Debug,
{
    match decompression_runner(
        &decode_struct.body,
        decode_struct.extra_body.as_deref(),
        encoding_info,
        decode_struct.buf,
    ) {
        Ok(state) => {
            let is_extra_raw = state.is_extra_raw();
            let (body, extra_body) = state.into();
            decode_struct.body = body;
            if !is_extra_raw {
                decode_struct.extra_body = extra_body;
            }
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
                is_extra_raw,
            } = e.reason
            {
                decode_struct.body = partial_body.split();
                if !is_extra_raw {
                    decode_struct.extra_body = None;
                }
                for (index, einfo) in encoding_info.iter().rev().enumerate() {
                    if index > header_index {
                        decode_struct
                            .message
                            .remove_header_on_position(einfo.header_index);
                    } else {
                        let iter = einfo
                            .encodings()
                            .iter()
                            .rev()
                            .skip(compression_index)
                            .rev()
                            .map(|e| e.as_ref());
                        decode_struct
                            .message
                            .header_map_as_mut()
                            .update_header_multiple_values_on_position(
                                einfo.header_index,
                                iter,
                            );
                        break;
                    }
                }
            }
            Err(e.reason)
        }
    }
}
