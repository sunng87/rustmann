use tokio::codec::{Encoder, Decoder};
use bytes::{BufMut, BytesMut, ByteOrder, BigEndian};
use protobuf::{Message, parse_from_carllerche_bytes};

use std::io;

use protos::riemann::{Event, Msg};

impl Encoder for Msg {
    type Item = Self;
    type Error = io::Error;

    fn encode(&mut self, msg: Self, buf: &mut BytesMut) -> io::Result<()> {
        let data = msg.write_to_bytes().map_err(io::Error::from)?;
        BigEndian::write_u32(buf, data.len() as u32);
        buf.put(&data);
        Ok(())
    }
}

impl Decoder for Msg {
    type Item = Self;
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
