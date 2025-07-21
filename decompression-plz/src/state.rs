use bytes::BytesMut;
use header_plz::body_headers::encoding_info::EncodingInfo;

use crate::{
    content_length::add_body_and_update_cl,
    decode_struct::DecodeStruct,
    decompression::{multi::error::MultiDecompressError, state::runner},
    dtraits::DecompressTrait,
    encoding_type::EncodingType,
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
    fn init(
        mut message: T,
        mut extra_body: Option<BytesMut>,
        buf: &'a mut BytesMut,
    ) -> Self {
        let mut decode_struct = DecodeStruct::new(message, extra_body, buf);
        Self::Start(decode_struct)
    }

    fn try_next(self) -> Self {
        match self {
            DecodeState::Start(mut decode_struct) => {
                if decode_struct.transfer_encoding_is_some() {
                    let encodings = decode_struct.transfer_encoding();
                    Self::TransferEncoding(decode_struct, encodings)
                } else if decode_struct.content_encoding_is_some() {
                    let encodings = decode_struct.content_encoding();
                    Self::ContentEncoding(decode_struct, encodings)
                } else {
                    if decode_struct.extra_body_is_some() {
                        Self::UpdateContentLength(decode_struct)
                    } else {
                        Self::End
                    }
                }
            }
            DecodeState::TransferEncoding(
                mut decode_struct,
                mut encoding_infos,
            ) => {
                let result =
                    apply_encoding(&mut decode_struct, &mut encoding_infos);
                if decode_struct.content_encoding_is_some() {
                    let encodings = decode_struct.content_encoding();
                    Self::ContentEncoding(decode_struct, encodings)
                } else {
                    Self::UpdateContentLength(decode_struct)
                }
            }
            DecodeState::ContentEncoding(
                mut decode_struct,
                mut encoding_infos,
            ) => {
                let result =
                    apply_encoding(&mut decode_struct, &mut encoding_infos);

                Self::UpdateContentLength(decode_struct)
            }
            DecodeState::UpdateContentLength(mut decode_struct) => {
                let mut body = decode_struct.take_main_body();
                if let Some(extra) = decode_struct.take_extra_body() {
                    body.unsplit(extra);
                }
                add_body_and_update_cl(&mut decode_struct.message, body);
                Self::End
            }
            DecodeState::End => Self::End,
        }
    }

    fn is_ended(&self) -> bool {
        matches!(self, DecodeState::End)
    }
}

fn apply_encoding<T>(
    decode_struct: &mut DecodeStruct<T>,
    encoding_info: &mut Vec<EncodingInfo>,
) -> Result<(), MultiDecompressError>
where
    T: DecompressTrait,
{
    match runner(
        &decode_struct.body,
        decode_struct.extra_body.as_deref(),
        encoding_info,
        decode_struct.buf,
    ) {
        Ok(state) => {
            let (main, extra) = state.into();
            decode_struct.body = main;
            decode_struct.extra_body = extra;
            Ok(())
        }
        Err(e) => todo!(),
    }
}
