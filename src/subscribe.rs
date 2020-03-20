use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

use zmq::SocketType;

use crate::Stream;
use crate::socket::{AsRaw, MessageBuf, Reciever, ZmqSocket};

pub fn subscribe(endpoint: &str) -> Result<Subscribe, zmq::Error> {
    let socket = zmq::Context::new().socket(SocketType::SUB)?;

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

impl Stream for Subscribe {
    type Item = Result<MessageBuf, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0).poll_next(cx)
    }
}

impl Subscribe {
    pub fn set_subscribe(&self, topic: &str) -> Result<(), zmq::Error> {
        self.as_raw_socket().set_subscribe(topic.as_bytes())
    }

    pub fn set_unsubscribe(&self, topic: &str) -> Result<(), zmq::Error> {
        self.as_raw_socket().set_unsubscribe(topic.as_bytes())
    }
}
