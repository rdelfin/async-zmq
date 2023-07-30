//! ROUTER socket module of Request-reply pattern in ZMQ
//!
//! Use the [`router`] function to instantiate a router socket and use methods
//! from the [`Sink`]/[`SinkExt`] and [`Stream`]/[`StreamExt`] traits.
//!
//! A router socket must be paired with a [`dealer`], [`request`] or another
//! router socket.
//!
//! # Example
//!
//! ```no_run
//! ```
//!
//! [`dealer`]: ../dealer/index.html
//! [`request`]: ../request/index.html
//! [`router`]: fn.router.html
//! [`Sink`]: ../trait.Sink.html
//! [`SinkExt`]: ../trait.SinkExt.html
//! [`Stream`]: ../trait.Stream.html
//! [`StreamExt`]: ../trait.StreamExt.html

use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{
    reactor::{AsRawSocket, ZmqSocket},
    socket::{Broker, Multipart, MultipartIter, SocketBuilder},
    RecvError, SendError, Sink, SocketError, Stream,
};
use zmq::{Message, SocketType};

/// Create a ZMQ socket with ROUTER type
pub fn router<I: Iterator<Item = T> + Unpin, T: Into<Message>>(
    endpoint: &str,
) -> Result<SocketBuilder<'_, Router<I, T>>, SocketError> {
    Ok(SocketBuilder::new(SocketType::ROUTER, endpoint))
}

/// The async wrapper of ZMQ socket with ROUTER type
pub struct Router<I: Iterator<Item = T> + Unpin, T: Into<Message>>(Broker<I, T>);

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Router<I, T> {
    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        self.0.socket.as_socket()
    }
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Sink<MultipartIter<I, T>> for Router<I, T> {
    type Error = SendError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::poll_ready(Pin::new(&mut self.get_mut().0), cx)
            .map(|result| result.map_err(Into::into))
    }

    fn start_send(self: Pin<&mut Self>, item: MultipartIter<I, T>) -> Result<(), Self::Error> {
        Pin::new(&mut self.get_mut().0)
            .start_send(item)
            .map_err(Into::into)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::poll_flush(Pin::new(&mut self.get_mut().0), cx)
            .map(|result| result.map_err(Into::into))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::poll_close(Pin::new(&mut self.get_mut().0), cx)
            .map(|result| result.map_err(Into::into))
    }
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Stream for Router<I, T> {
    type Item = Result<Multipart, RecvError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0)
            .poll_next(cx)
            .map(|poll| poll.map(|result| result.map_err(Into::into)))
    }
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> From<zmq::Socket> for Router<I, T> {
    fn from(socket: zmq::Socket) -> Self {
        Self(Broker {
            socket: ZmqSocket::from(socket),
            buffer: None,
        })
    }
}
