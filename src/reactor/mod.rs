//! Socket type registered in async-std reactor
pub(crate) mod evented;
mod watcher;

use crate::socket::{Multipart, MultipartIter};
pub(crate) use watcher::Watcher;

use futures::ready;
use std::io::{self, ErrorKind};
use std::task::{Context, Poll};
use zmq::Error;

/// Trait to get the raw zmq socket.
pub trait AsRawSocket {
    /// Method to get the raw zmq socket reference if users need to use it directly.
    fn as_socket(&self) -> &zmq::Socket;
}

pub(crate) type ZmqSocket = Watcher<evented::ZmqSocket>;

impl ZmqSocket {
    fn poll_event(&self, event: zmq::PollEvents) -> Result<(), io::Error> {
        if self.as_socket().get_events()?.contains(event) {
            Ok(())
        } else {
            Err(io::Error::new(ErrorKind::WouldBlock, Error::EAGAIN))
        }
    }

    pub(crate) fn send<I: Iterator<Item = T>, T: Into<zmq::Message>>(
        &self,
        cx: &mut Context<'_>,
        buffer: &mut MultipartIter<I, T>,
    ) -> Poll<Result<(), Error>> {
        let _ = ready!(self.poll_write_with(cx, |_| { self.poll_event(zmq::POLLOUT) }));
        //ready!()?;

        let mut buffer = buffer.0.by_ref().peekable();
        while let Some(msg) = buffer.next() {
            let mut flags = zmq::DONTWAIT;
            if buffer.peek().is_some() {
                flags |= zmq::SNDMORE;
            }

            match self.as_socket().send(msg, flags) {
                Ok(_) => {}
                Err(Error::EAGAIN) => return Poll::Pending,
                Err(e) => return Poll::Ready(Err(e)),
            }
        }

        Poll::Ready(Ok(()))
    }

    pub(crate) fn recv(&self, cx: &mut Context<'_>) -> Poll<Result<Multipart, Error>> {
        let _ = ready!(self.poll_read_with(cx, |_| { self.poll_event(zmq::POLLIN) }));

        let mut buffer = Vec::new();
        let mut more = true;

        while more {
            let mut msg = zmq::Message::new();
            match self.as_socket().recv(&mut msg, zmq::DONTWAIT) {
                Ok(_) => {
                    more = msg.get_more();
                    buffer.push(msg);
                }
                Err(e) => return Poll::Ready(Err(e)),
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
