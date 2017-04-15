use bytes::BytesMut;
use MarsError;
use damnpacket;
use damnpacket::Message;
use tokio_io::codec::{Decoder,Encoder};

#[derive(Debug)]
pub struct DamnCodec;

impl Decoder for DamnCodec {
    type Item = Message;
    type Error = MarsError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(i) = buf.iter().position(|&b| b == b'\0') {
            let line = buf.split_to(i + 1);
            match damnpacket::parse(&line[..]) {
                Ok(msg) => Ok(Some(msg)),
                Err(e) => Err(MarsError::from(e)),
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder for DamnCodec {
    type Item = Message;
    type Error = MarsError;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.extend(msg.as_bytes());
        Ok(())
    }
}
