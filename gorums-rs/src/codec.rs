extern crate bytes;

use bytes::{Buf, BufMut};
use prost::encoding::*;
use prost::DecodeError;
use prost::Message as ProstMessage;
use tonic::codec::*;
use tonic::{Code, Status};

use crate::proto::gorums::Metadata;
use crate::Message;

#[derive(Default)]
pub struct GorumsCodec {}

impl Codec for GorumsCodec {
    type Encode = Metadata;
    type Decode = Metadata;

    type Encoder = GorumsEncoder;
    type Decoder = GorumsDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        GorumsEncoder {}
    }

    fn decoder(&mut self) -> Self::Decoder {
        GorumsDecoder {}
    }
}

#[derive(Debug, Clone, Default)]
pub struct GorumsEncoder {}

impl Encoder for GorumsEncoder {
    type Item = Metadata;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        let md = item.get_metadata();
        encode_varint(md.encoded_len() as u64, buf);
        md.encode(buf)
            .expect("Message only errors if not enough space");

        let msg = item.get_message();
        encode_varint(msg.encoded_len() as u64, buf);
        buf.put_slice(msg);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct GorumsDecoder {}

impl Decoder for GorumsDecoder {
    type Item = Metadata;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let md_len = decode_varint(src).map_err(from_decode_error)?;
        let md_buf = src.take(md_len as usize);
        let mut md = Metadata::decode(md_buf).map_err(from_decode_error)?;

        let msg_len = decode_varint(src).map_err(from_decode_error)?;
        let msg_buf = src.take(msg_len as usize);
        md.message = msg_buf.chunk().to_vec(); // will be decoded by handler later

        Ok(Some(md))
    }
}

fn from_decode_error(error: DecodeError) -> Status {
    Status::new(Code::Internal, error.to_string())
}
