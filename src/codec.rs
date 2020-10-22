use prost::bytes::{Buf, BufMut, BytesMut};
use prost::Message;
use tokio_util::codec::{Decoder, Encoder};

use std::io;
use std::usize;

use crate::protos::riemann::Msg;

#[derive(Debug, Default)]
pub struct MsgCodec {
    len: Option<usize>,
}

impl Encoder<Msg> for MsgCodec {
    type Error = io::Error;

    fn encode(&mut self, msg: Msg, buf: &mut BytesMut) -> io::Result<()> {
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
        if let Some(msg_len) = self.len {
            if buf.remaining() >= msg_len {
                let msg = Msg::decode(buf.split_to(msg_len)).map_err(io::Error::from)?;
                self.len = None;
                Ok(Some(msg))
            } else {
                Ok(None)
            }
        } else if buf.remaining() > 4 {
            let msg_len = buf.split_to(4).get_u32() as usize;
            self.len = Some(msg_len);
            self.decode(buf)
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
