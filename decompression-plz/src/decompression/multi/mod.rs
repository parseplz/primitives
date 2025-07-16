use bytes::{BytesMut, buf::Writer};
use header_plz::body_headers::encoding_info::EncodingInfo;

use crate::{
    decompression::single::decompress,
    error::{DecompressErrorStruct, Reason},
};

mod error;

pub fn decompress_multi(
    mut compressed: &[u8],
    mut writer: &mut Writer<&mut BytesMut>,
    encoding_info: &[EncodingInfo],
) -> Result<BytesMut, DecompressErrorStruct> {
    let mut input: &[u8] = compressed;
    let mut output: BytesMut = writer.get_mut().split();

    for (header_index, encoding_info) in encoding_info.iter().rev().enumerate() {
        for (compression_index, encoding) in encoding_info.encodings().iter().rev().enumerate() {
            let result = decompress(&mut input, &mut writer, encoding.clone());
            match result {
                Ok(_) => {
                    output = writer.get_mut().split();
                    input = &output[..];
                }
                Err(e) => {
                    let reason = if header_index == 0 && compression_index == 0 {
                        Reason::Corrupt
                    } else {
                        Reason::PartialCorrupt(header_index, compression_index)
                    };

                    let body = match reason {
                        Reason::Corrupt => None,
                        Reason::PartialCorrupt(_, _) => {
                            writer.get_mut().clear();
                            std::io::copy(&mut input, writer).unwrap();
                            output = writer.get_mut().split();
                            Some(output)
                        }
                    };

                    todo!()
                    //return Err(DecompressErrorStruct::new(output, None, e, reason));
                }
            }
        }
    }
    Ok(output)
}
