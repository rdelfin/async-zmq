//! Socket type registered in async-std reactor
pub(crate) mod evented;
mod watcher;

pub(crate) use watcher::Watcher;
use crate::socket::{Multipart, MultipartIter};

use futures::ready;
use std::task::{Context, Poll};
use zmq::Error;

/// Trait to get the raw zmq socket.
pub trait AsRawSocket {
    /// Method to get the raw zmq socket reference if users need to use it directly.
    fn as_socket(&self) -> &zmq::Socket;
}

pub(crate) type ZmqSocket = Watcher<evented::ZmqSocket>;

impl ZmqSocket {
    fn poll_event(&self, event: zmq::PollEvents) -> Poll<Result<(), Error>> {
        if self.as_socket().get_events()?.contains(event) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(Error::EAGAIN))
        }
    }

    pub(crate) fn send<I: Iterator<Item = T>, T: Into<zmq::Message>>(
        &self,
        cx: &mut Context<'_>,
        buffer: &mut MultipartIter<I, T>,
    ) -> Poll<Result<(), Error>> {
        ready!(self.poll_write_ready(cx));
        ready!(self.poll_event(zmq::POLLOUT))?;

        let mut buffer = buffer.0.by_ref().peekable();
        while let Some(msg) = buffer.next() {
            let mut flags = zmq::DONTWAIT;
            if let Some(_) = buffer.peek() {
                flags |= zmq::SNDMORE;
            }

            match self.as_socket().send(msg, flags) {
                Ok(_) => {}
                Err(Error::EAGAIN) => return Poll::Pending,
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }

        Poll::Ready(Ok(()))
    }

    pub(crate) fn recv(&self, cx: &mut Context<'_>) -> Poll<Result<Multipart, Error>> {
        ready!(self.poll_read_ready(cx));
        ready!(self.poll_event(zmq::POLLIN))?;

        let mut buffer = Vec::new();
        let mut more = true;

        while more {
            let mut msg = zmq::Message::new();
            match self.as_socket().recv(&mut msg, zmq::DONTWAIT) {
                Ok(_) => {
                    more = msg.get_more();
                    buffer.push(msg);
                }
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }

        Poll::Ready(Ok(buffer))
    }
}

impl From<zmq::Socket> for ZmqSocket {
    fn from(socket: zmq::Socket) -> Self {
        Watcher::new(evented::ZmqSocket(socket))
    }
}

impl AsRawSocket for ZmqSocket {
    fn as_socket(&self) -> &zmq::Socket {
        &self.get_ref().0
    }
}
