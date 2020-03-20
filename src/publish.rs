use zmq::{Context, Result, SocketType};

use crate::socket::{AsRaw, MessageBuf, Sender, ZmqSocket};

pub fn publish(endpoint: &str) -> Result<Publish> {
    let socket = Context::new().socket(SocketType::PUB)?;

    socket.bind(endpoint)?;

    Ok(Publish::from(socket))
}

pub struct Publish(Sender);

impl AsRaw for Publish {
    fn as_raw_socket(&self) -> &zmq::Socket {
        &self.0.socket.get_ref().0
    }
}

impl From<zmq::Socket> for Publish {
    fn from(socket: zmq::Socket) -> Self {
        Self(Sender {
            socket: ZmqSocket::from(socket),
            buffer: MessageBuf::default(),
        })
    }
}
