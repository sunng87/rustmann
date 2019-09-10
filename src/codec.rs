use bytes::{BigEndian, BufMut, ByteOrder, BytesMut};
use protobuf::{parse_from_carllerche_bytes, Message};
use tokio::codec::{Decoder, Encoder};

use std::io;
use std::usize;

use crate::protos::riemann::Msg;

#[derive(Debug)]
pub struct MsgCodec;

impl Encoder for MsgCodec {
    type Item = Msg;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        let data = msg.write_to_bytes().map_err(io::Error::from)?;

        buf.reserve(4 + data.len());

        buf.put_u32_be(data.len() as u32);
        buf.put(&data);
        Ok(())
    }
}

impl Decoder for MsgCodec {
    type Item = Msg;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        if buf.len() > 4 {
            let msg_len = BigEndian::read_u32(&buf.split_to(4)) as usize;

            if buf.len() >= msg_len {
                let msg = parse_from_carllerche_bytes::<Msg>(&buf.split_to(msg_len).into())
                    .map_err(io::Error::from)?;
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

    let data = msg.write_to_bytes().map_err(io::Error::from)?;
    buf.put(&data);

    Ok(buf)
}
