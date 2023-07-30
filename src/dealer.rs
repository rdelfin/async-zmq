//! DEALER socket module of Request-reply pattern in ZMQ
//!
//! Use the [`dealer`] function to instantiate a dealer socket and use methods
//! from the [`Sink`]/[`SinkExt`] and [`Stream`]/[`StreamExt`] traits.
//!
//! A dealer socket must be paired with a [`router`], [`reply`] or another
//! dealer socket.
//!
//! # Example
//!
//! ```no_run
//! ```
//!
//! [`router`]: ../router/index.html
//! [`reply`]: ../reply/index.html
//! [`dealer`]: fn.dealer.html
//! [`Sink`]: ../trait.Sink.html
//! [`SinkExt`]: ../trait.SinkExt.html
//! [`Stream`]: ../trait.Stream.html
//! [`StreamExt`]: ../trait.StreamExt.html

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    reactor::{AsRawSocket, ZmqSocket},
    socket::{Broker, Multipart, MultipartIter, SocketBuilder},
    RecvError, SendError, Sink, SocketError, Stream,
};
use zmq::{Message, SocketType};

/// Create a ZMQ socket with DEALER type
pub fn dealer<I: Iterator<Item = T> + Unpin, T: Into<Message>>(
    endpoint: &str,
) -> Result<SocketBuilder<'_, Dealer<I, T>>, SocketError> {
    Ok(SocketBuilder::new(SocketType::DEALER, endpoint))
}

/// The async wrapper of ZMQ socket with DEALER type
pub struct Dealer<I: Iterator<Item = T> + Unpin, T: Into<Message>>(Broker<I, T>);

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Dealer<I, T> {
    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        self.0.socket.as_socket()
    }
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Sink<MultipartIter<I, T>> for Dealer<I, T> {
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

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Stream for Dealer<I, T> {
    type Item = Result<Multipart, RecvError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0)
            .poll_next(cx)
            .map(|poll| poll.map(|result| result.map_err(Into::into)))
    }
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> From<zmq::Socket> for Dealer<I, T> {
    fn from(socket: zmq::Socket) -> Self {
        Self(Broker {
            socket: ZmqSocket::from(socket),
            buffer: None,
        })
    }
}
