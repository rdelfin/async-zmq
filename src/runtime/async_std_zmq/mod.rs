//!
//! Async-std runtime Settings modules
//!
mod watcher;
pub(crate) use watcher::Watcher;

use crate::{
    runtime::{evented, InnerSocket, IntoSocket},
    socket::MessageBuf,
};

use futures::ready;
use std::task::{Context, Poll};
use zmq::Error;

pub(crate) type ZmqSocket = Watcher<evented::ZmqSocket>;

impl ZmqSocket {
    fn poll_event(&self, event: zmq::PollEvents) -> Poll<Result<(), Error>> {
        if self.into_socket().get_events()?.contains(event) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(Error::EAGAIN))
        }
    }
}

impl From<zmq::Socket> for ZmqSocket {
    fn from(socket: zmq::Socket) -> Self {
        Watcher::new(evented::ZmqSocket(socket))
    }
}

impl IntoSocket for ZmqSocket {
    fn into_socket(&self) -> &zmq::Socket {
        &self.get_ref().0
    }
}

impl InnerSocket for ZmqSocket {
    /// Error type
    type Error = zmq::Error;

    /// Messages
    type Item = MessageBuf;

    /// the future that send messages to a ZMQ socket
    type Request = Poll<Result<(), Self::Error>>;

    /// the future that receive messages from a ZMQ socket
    type Response = Poll<Result<Self::Item, Self::Error>>;

    ///
    /// Send buffer data
    ///
    fn send(&self, cx: &mut Context<'_>, buffer: &mut Self::Item) -> Self::Request {
        ready!(self.poll_write_ready(cx));
        ready!(self.poll_event(zmq::POLLOUT))?;

        while let Some(msg) = buffer.pop_front() {
            let mut flags = zmq::DONTWAIT;
            if !buffer.is_empty() {
                flags |= zmq::SNDMORE;
            }

            match self.into_socket().send(msg, flags) {
                Ok(_) => {}
                Err(zmq::Error::EAGAIN) => return Poll::Pending,
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }

        Poll::Ready(Ok(()))
    }

    ///
    /// Recive data
    ///
    fn recv(&self, cx: &mut Context<'_>) -> Self::Response {
        ready!(self.poll_read_ready(cx));
        ready!(self.poll_event(zmq::POLLIN))?;

        let mut buffer = MessageBuf::default();
        let mut more = true;

        while more {
            let mut msg = zmq::Message::new();
            match self.into_socket().recv(&mut msg, zmq::DONTWAIT) {
                Ok(_) => {
                    more = msg.get_more();
                    buffer.0.push_back(msg);
                }
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }

        Poll::Ready(Ok(buffer))
    }
}
