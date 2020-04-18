//! PUB socket module of Pub-Sub pattern in ZMQ
//!
//! Use the [`publish`] function to instantiate a publish socket and use methods
//! from the [`Sink`]/[`SinkExt`] traits.
//!
//! A publish socket must be paired with a [`subscribe`] or [`xsubscribe`] socket.
//!
//! # Example
//!
//! ```no_run
//! use async_zmq::{Result, SinkExt};
//!
//! #[async_std::main]
//! async fn main() -> Result<()> {
//!     let mut zmq = async_zmq::publish("tcp://127.0.0.1:5555")?.bind()?;
//!
//!     zmq.send(vec!["topic", "broadcast message"].into()).await?;
//!     Ok(())
//! }
//! ```
//!
//! [`subscribe`]: ../subscribe/index.html
//! [`xsubscribe`]: ../xsubscribe/index.html
//! [`publish`]: fn.publish.html
//! [`Sink`]: ../trait.Sink.html
//! [`SinkExt`]: ../trait.SinkExt.html

use std::pin::Pin;
use std::task::{Context, Poll};

use zmq::{Message, SocketType};

use crate::{
    reactor::{AsRawSocket, ZmqSocket},
    socket::{MultipartIter, Sender, SocketBuilder},
    SendError, Sink, SocketError,
};

/// Create a ZMQ socket with PUB type
pub fn publish<I: Iterator<Item = T> + Unpin, T: Into<Message>>(
    endpoint: &str,
) -> Result<SocketBuilder<'_, Publish<I, T>>, SocketError> {
    Ok(SocketBuilder::new(SocketType::PUB, endpoint))
}

/// The async wrapper of ZMQ socket with PUB type
pub struct Publish<I: Iterator<Item = T> + Unpin, T: Into<Message>>(Sender<I, T>);

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Publish<I, T> {
    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        &self.0.socket.as_socket()
    }
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Sink<MultipartIter<I, T>> for Publish<I, T> {
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

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> From<zmq::Socket> for Publish<I, T> {
    fn from(socket: zmq::Socket) -> Self {
        Self(Sender {
            socket: ZmqSocket::from(socket),
            buffer: None,
        })
    }
}
