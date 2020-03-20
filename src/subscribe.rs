use zmq::{Context, Result, SocketType};

use crate::socket::{AsRaw, Reciever, ZmqSocket};

pub fn subscribe(endpoint: &str) -> Result<Subscribe> {
    let socket = Context::new().socket(SocketType::SUB)?;

    socket.connect(endpoint)?;

    Ok(Subscribe::from(socket))
}

pub struct Subscribe(Reciever);

impl AsRaw for Subscribe {
    fn as_raw_socket(&self) -> &zmq::Socket {
        &self.0.socket.get_ref().0
    }
}

impl From<zmq::Socket> for Subscribe {
    fn from(socket: zmq::Socket) -> Self {
        Self(Reciever {
            socket: ZmqSocket::from(socket),
        })
    }
}

impl Subscribe {
    pub fn subscribe(&self, topic: &str) -> Result<()> {
        self.as_raw_socket().set_subscribe(topic.as_bytes())
    }

    pub fn unsubscribe(&self, topic: &str) -> Result<()> {
        self.as_raw_socket().set_unsubscribe(topic.as_bytes())
    }
}
