use bytes::{Buf, BufMut, BytesMut};
use prost::Message;
use tokio_util::codec::{Decoder, Encoder};

use std::io;
use std::usize;

use crate::protos::riemann::Msg;

#[derive(Debug)]
pub struct MsgCodec;

impl Encoder for MsgCodec {
    type Item = Msg;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        let size = msg.encoded_len();
        buf.put_u32(size as u32);

        msg.encode(buf).map_err(io::Error::from)?;
        Ok(())
    }
}

impl Decoder for MsgCodec {
    type Item = Msg;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        if buf.len() > 4 {
            let msg_len = buf.split_to(4).get_u32() as usize;

            if buf.len() >= msg_len {
                let msg = Msg::decode(buf.split_to(msg_len)).map_err(io::Error::from)?;
                Ok(Some(msg))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

pub(crate) fn encode_for_udp(msg: &Msg) -> Result<BytesMut, io::Error> {
    let mut buf = BytesMut::new();

    msg.encode(&mut buf).map_err(io::Error::from)?;
    Ok(buf)
}
