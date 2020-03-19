use std::collections::VecDeque;
use std::task::{Context, Poll};
use std::pin::Pin;

use async_std::task::ready;
use async_std::io::Error;
use zmq::Message;

use crate::evented;
use crate::watcher::Watcher;

pub type MessageBuf = VecDeque<Message>;
pub(crate) type ZmqSocket = Watcher<evented::ZmqSocket>;

impl From<zmq::Socket> for ZmqSocket {
    fn from(socket: zmq::Socket) -> Self {
        Watcher::new(evented::ZmqSocket(socket))
    }
}

impl ZmqSocket {
    pub fn send(&self, cx: &mut Context<'_>, buffer: &mut MessageBuf) -> Poll<Result<(), Error>> {
        ready!(self.poll_write_ready(cx));
        ready!(self.poll_event(zmq::POLLIN))?;

        while let Some(msg) = buffer.pop_front() {
            let mut flags = zmq::DONTWAIT;
            if !buffer.is_empty() {
                flags |= zmq::SNDMORE;
            }

            match self.get_ref().0.send(msg, flags) {
                Ok(_) => {}
                Err(zmq::Error::EAGAIN) => return Poll::Pending,
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }

        Poll::Ready(Ok(()))
    }

    pub fn recv(&self, cx: &mut Context<'_>) -> Poll<Result<MessageBuf, Error>> {
        ready!(self.poll_read_ready(cx));
        ready!(self.poll_event(zmq::POLLOUT))?;

        let mut buffer = MessageBuf::new();
        let mut more = true;

        while more {
            let mut msg = zmq::Message::new();
            match self.get_ref().0.recv(&mut msg, zmq::DONTWAIT) {
                Ok(_) => {
                    more = msg.get_more();
                    buffer.push_back(msg);
                }
                Err(zmq::Error::EAGAIN) => return Poll::Pending,
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }

        Poll::Ready(Ok(buffer))
    }

    fn poll_event(&self, event: zmq::PollEvents) -> Poll<Result<(), Error>> {
        if self.get_ref().0.get_events()?.contains(event) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}

#[must_use = "sinks do nothing unless polled"]
pub trait Sink<Item> {
    type Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error>;

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
}

pub struct Sender {
    pub(crate) socket: ZmqSocket,
    pub(crate) buffer: MessageBuf,
}

impl Sink<MessageBuf> for Sender {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: MessageBuf) -> Result<(), Self::Error> {
        self.get_mut().buffer = item;
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        let Self {
            socket,
            buffer,
        } = self.get_mut();

        socket.send(cx, buffer)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        self.poll_flush(cx)
    }
}

pub struct Reciever {
    pub(crate) socket: ZmqSocket,
}

impl async_std::stream::Stream for Reciever {
    type Item = Result<MessageBuf, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        Poll::Ready(Some(Ok(ready!(self.get_mut().socket.recv(cx))?)))
    }
}
