use std::collections::VecDeque;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use futures::sync::mpsc::{self, UnboundedSender};
use futures::sync::oneshot::{self, Sender};
use futures::{Future, Sink, Stream};
use protobuf::RepeatedField;
use tokio;
use tokio::codec::Framed;
use tokio::net::TcpStream;

use codec::MsgCodec;
use protos::riemann::{Event, Msg};

#[derive(Debug)]
pub struct Client {

}
