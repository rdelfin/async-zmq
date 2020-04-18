use std::convert::Into;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{reactor::ZmqSocket, Message, Sink, Stream};
use futures::ready;
use zmq::Error;

/// Alias type for Multipart Iterator.
///
/// This is a slice of element with [`Into<MessageBuf>`] implemented.
///
/// [`Into<MessageBuf>`]: https://doc.rust-lang.org/std/convert/trait.Into.html
pub struct MultipartIter<I: Iterator<Item=T>, T: Into<Message>>(pub I);

impl <T: Into<Message>> From<Vec<T>> for MultipartIter<std::vec::IntoIter<T>, T> {
    fn from(vec: Vec<T>) -> Self {
        MultipartIter(vec.into_iter())
    }
}

impl <T: Into<Message>> From<T> for MultipartIter<std::vec::IntoIter<T>, T> {
    fn from(m: T) -> Self {
        MultipartIter(vec![m].into_iter())
    }
}

/// Alias type for Multipart.
pub type Multipart = Vec<Message>;

/// ZMQ socket builder. It lets user to either bind or connect the socket of their choice.
pub struct SocketBuilder<'a, T> {
    pub(crate) context: Option<&'a zmq::Context>,
    pub(crate) socket_type: zmq::SocketType,
    pub(crate) endpoint: &'a str,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T> SocketBuilder<'a, T>
where
    T: From<zmq::Socket>,
{
    pub(crate) fn new(socket_type: zmq::SocketType, endpoint: &'a str) -> Self {
        Self {
            context: None,
            socket_type,
            endpoint,
            _phantom: Default::default(),
        }
    }

    /// Get the zmq context to share with
    pub fn get_context(&self) -> Option<&zmq::Context> {
        self.context
    }

    /// Create the zmq socket with given context
    pub fn with_context(self, context: &'a zmq::Context) -> Self {
        Self {
            context: Some(context),
            socket_type: self.socket_type,
            endpoint: self.endpoint,
            _phantom: Default::default(),
        }
    }

    /// Connect to the ZMQ endpoint based on given URI
    pub fn connect(self) -> Result<T, Error> {
        let socket = match self.context {
            Some(cx) => cx.socket(self.socket_type)?,
            None => zmq::Context::new().socket(self.socket_type)?,
        };

        socket.connect(self.endpoint)?;
        Ok(T::from(socket))
    }

    /// Bind to the ZMQ endpoint based on given URI
    pub fn bind(self) -> Result<T, Error> {
        let socket = match self.context {
            Some(cx) => cx.socket(self.socket_type)?,
            None => zmq::Context::new().socket(self.socket_type)?,
        };

        socket.bind(self.endpoint)?;
        Ok(T::from(socket))
    }
}

pub(crate) struct Sender<I: Iterator<Item=T> + Unpin, T: Into<Message>> {
    pub(crate) socket: ZmqSocket,
    pub(crate) buffer: Option<MultipartIter<I, T>>,
}

impl<I: Iterator<Item=T> + Unpin, T: Into<Message>> Sink<MultipartIter<I, T>> for Sender<I, T> {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::poll_flush(self, cx)
    }

    fn start_send(self: Pin<&mut Self>, item: MultipartIter<I, T>) -> Result<(), Self::Error> {
        self.get_mut().buffer = Some(item.into());
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let Self { socket, buffer } = self.get_mut();
        if let Some(buffer) = buffer.as_mut() {
            socket.send(cx, buffer)
        } else {
            Poll::Ready(Ok(()))
        }
        
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::poll_flush(self, cx)
    }
}

pub(crate) struct Receiver {
    pub(crate) socket: ZmqSocket,
}

impl Stream for Receiver {
    type Item = Result<Multipart, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(Some(Ok(ready!(self.socket.recv(cx))?)))
    }
}

pub(crate) struct Broker<I: Iterator<Item=T> + Unpin, T: Into<Message>> {
    pub(crate) socket: ZmqSocket,
    pub(crate) buffer: Option<MultipartIter<I, T>>,
}

impl<I: Iterator<Item=T> + Unpin, T: Into<Message>> Sink<MultipartIter<I, T>> for Broker<I, T> {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::poll_flush(self, cx)
    }

    fn start_send(self: Pin<&mut Self>, item: MultipartIter<I, T>) -> Result<(), Self::Error> {
        self.get_mut().buffer = Some(item);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let Self { socket, buffer} = self.get_mut();
        if let Some(buffer) = buffer.as_mut() {
            socket.send(cx, buffer)
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::poll_flush(self, cx)
    }
}

impl<I: Iterator<Item=T> + Unpin, T: Into<Message>> Stream for Broker<I, T> {
    type Item = Result<Multipart, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(Some(Ok(ready!(self.socket.recv(cx))?)))
    }
}
