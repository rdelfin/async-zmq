use std::collections::VecDeque;
use std::convert::Into;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_std::task::ready;
use zmq::Error;

use crate::evented;
use crate::watcher::Watcher;
use crate::{Message, Sink, Stream};

/// Alias type for Message queue.
///
/// This is a [`VecDeque`] to easier popping front [`Message`](struct.Message.html).
/// Users are free to use any type to queue their message as long as it satisfied trait boud [`Into<MessageBuf>`].
///
/// [`VecDeque`]: https://doc.rust-lang.org/std/collections/struct.VecDeque.html
/// [`Into<MessageBuf>`]: https://doc.rust-lang.org/std/convert/trait.Into.html
#[derive(Debug, Default, PartialEq, Eq)]
pub struct MessageBuf(pub VecDeque<Message>);

impl From<Message> for MessageBuf {
    fn from(message: Message) -> Self {
        let mut buf = VecDeque::with_capacity(1);
        buf.push_back(message);
        Self(buf)
    }
}

impl<T: Into<Message>> From<Vec<T>> for MessageBuf {
    fn from(vec: Vec<T>) -> Self {
        Self(vec.into_iter().map(|i| i.into()).collect())
    }
}

impl std::iter::FromIterator<Message> for MessageBuf {
    fn from_iter<T: IntoIterator<Item = Message>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl std::ops::Deref for MessageBuf {
    type Target = VecDeque<Message>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MessageBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// ZMQ socket builder. It lets user to either bind or connect the socket of their choice.
pub struct SocketBuilder<'a, T> {
    pub(crate) socket: zmq::Socket,
    pub(crate) endpoint: &'a str,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T> SocketBuilder<'a, T>
where
    T: From<zmq::Socket>,
{
    pub(crate) fn new(socket: zmq::Socket, endpoint: &'a str) -> Self {
        Self {
            socket,
            endpoint,
            _phantom: Default::default(),
        }
    }
    /// Connect to the ZMQ endpoint based on given URI
    pub fn connect(self) -> Result<T, Error> {
        self.socket.connect(self.endpoint)?;
        Ok(T::from(self.socket))
    }

    /// Bind to the ZMQ endpoint based on given URI
    pub fn bind(self) -> Result<T, Error> {
        self.socket.bind(self.endpoint)?;
        Ok(T::from(self.socket))
    }
}

pub(crate) type ZmqSocket = Watcher<evented::ZmqSocket>;

impl ZmqSocket {
    pub(crate) fn as_raw_socket(&self) -> &zmq::Socket {
        &self.get_ref().0
    }
}

impl From<zmq::Socket> for ZmqSocket {
    fn from(socket: zmq::Socket) -> Self {
        Watcher::new(evented::ZmqSocket(socket))
    }
}

impl ZmqSocket {
    pub(crate) fn send(
        &self,
        cx: &mut Context<'_>,
        buffer: &mut MessageBuf,
    ) -> Poll<Result<(), Error>> {
        ready!(self.poll_write_ready(cx));
        ready!(self.poll_event(zmq::POLLOUT))?;

        while let Some(msg) = buffer.pop_front() {
            let mut flags = zmq::DONTWAIT;
            if !buffer.is_empty() {
                flags |= zmq::SNDMORE;
            }

            match self.as_raw_socket().send(msg, flags) {
                Ok(_) => {}
                Err(zmq::Error::EAGAIN) => return Poll::Pending,
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }

        Poll::Ready(Ok(()))
    }

    pub(crate) fn recv(&self, cx: &mut Context<'_>) -> Poll<Result<MessageBuf, Error>> {
        ready!(self.poll_read_ready(cx));
        ready!(self.poll_event(zmq::POLLIN))?;

        let mut buffer = MessageBuf::default();
        let mut more = true;

        while more {
            let mut msg = zmq::Message::new();
            match self.as_raw_socket().recv(&mut msg, zmq::DONTWAIT) {
                Ok(_) => {
                    more = msg.get_more();
                    buffer.0.push_back(msg);
                }
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }

        Poll::Ready(Ok(buffer))
    }

    fn poll_event(&self, event: zmq::PollEvents) -> Poll<Result<(), Error>> {
        if self.as_raw_socket().get_events()?.contains(event) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(Error::EAGAIN))
        }
    }
}

pub(crate) struct Sender {
    pub(crate) socket: ZmqSocket,
    pub(crate) buffer: MessageBuf,
}

impl<T: Into<MessageBuf>> Sink<T> for Sender {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<T>::poll_flush(self, cx)
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.get_mut().buffer = item.into();
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let Self { socket, buffer } = self.get_mut();
        socket.send(cx, buffer)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<T>::poll_flush(self, cx)
    }
}

pub(crate) struct Reciever {
    pub(crate) socket: ZmqSocket,
}

impl Stream for Reciever {
    type Item = Result<MessageBuf, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(Some(Ok(ready!(self.socket.recv(cx))?)))
    }
}

pub(crate) struct Broker {
    pub(crate) socket: ZmqSocket,
    pub(crate) buffer: MessageBuf,
}

impl<T: Into<MessageBuf>> Sink<T> for Broker {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<T>::poll_flush(self, cx)
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.get_mut().buffer = item.into();
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let Self { socket, buffer } = self.get_mut();
        socket.send(cx, buffer)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<T>::poll_flush(self, cx)
    }
}

impl Stream for Broker {
    type Item = Result<MessageBuf, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(Some(Ok(ready!(self.socket.recv(cx))?)))
    }
}
