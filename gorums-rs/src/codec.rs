extern crate bytes;

use bytes::{Buf, BufMut};
use prost::encoding::*;
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

impl GorumsEncoder {
    fn encode_msg(
        &mut self,
        msg: Metadata,
        buf: &mut impl BufMut,
    ) -> Result<(), prost::EncodeError> {
        let md = msg.get_metadata();
        encode_varint(md.encoded_len() as u64, buf);
        md.encode(buf)
            .expect("Message only errors if not enough space");

        let msg = msg.get_message();
        encode_varint(msg.encoded_len() as u64, buf);
        buf.put_slice(msg);
        Ok(())
    }
}

impl Encoder for GorumsEncoder {
    type Item = Metadata;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        self.encode_msg(item, buf)
            .map_err(|err| Status::new(Code::Internal, err.to_string()))
    }
}

#[derive(Debug, Clone, Default)]
pub struct GorumsDecoder {}

impl GorumsDecoder {
    fn decode_msg(&mut self, buf: &mut impl Buf) -> Result<Metadata, prost::DecodeError> {
        let md_len = decode_varint(buf)?;
        let md_buf = buf.take(md_len as usize);
        let mut md = Metadata::decode(md_buf)?;

        let msg_len = decode_varint(buf)?;
        let msg_buf = buf.take(msg_len as usize);
        md.message = msg_buf.chunk().to_vec(); // will be decoded by handler later

        Ok(md)
    }
}

impl Decoder for GorumsDecoder {
    type Item = Metadata;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        self.decode_msg(src)
            .map(|msg| Some(msg))
            .map_err(|err| Status::new(Code::Internal, err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};
    use prost::Message;

    use super::*;
    use crate::proto::gorums::Metadata;

    #[test]
    fn encode() {
        let md = Metadata {
            message_id: 1,
            method: "foo".to_string(),
            message: "bar".as_bytes().to_vec(),
            status: None,
        };
        let mut buf = BytesMut::new();
        let mut codec = GorumsCodec::default();

        let result = codec.encoder().encode_msg(md.clone(), &mut buf);

        assert!(result.is_ok());
    }

    #[test]
    fn decode() {
        // message encoded by gorums: Metadata{id=1, method=foo}, message=Metadata{id=2, method=bar}
        let raw_msg: &[u8] = &[7, 8, 1, 18, 3, 102, 111, 111, 7, 8, 2, 18, 3, 98, 97, 114];
        let mut buf = Bytes::from(raw_msg);
        let mut codec = GorumsCodec::default();

        let md = codec.decoder().decode_msg(&mut buf).unwrap();
        // try to decode the payload message, which in this case happens to be a metadata message.
        let mut msg_buf = Bytes::from(md.message);
        let msg = Metadata::decode(&mut msg_buf).unwrap();

        assert_eq!(1, md.message_id);
        assert_eq!("foo", md.method);
        assert_eq!(2, msg.message_id);
        assert_eq!("bar", msg.method);
    }

    #[test]
    fn encode_and_decode() {
        let md = Metadata {
            message_id: 1,
            method: "foo".to_string(),
            message: "bar".as_bytes().to_vec(),
            status: None,
        };
        let mut buf = BytesMut::new();
        let mut codec = GorumsCodec::default();

        let encode_result = codec.encoder().encode_msg(md.clone(), &mut buf);
        assert!(encode_result.is_ok());
        let decode_result = codec.decoder().decode_msg(&mut buf);
        assert!(decode_result.is_ok());

        let decoded_msg = decode_result.unwrap();
        assert_eq!(md, decoded_msg);
    }
}
