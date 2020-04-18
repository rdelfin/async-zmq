//! XPUB socket module of Pub-Sub pattern in ZMQ
//!
//! Use the [`xpublish`] function to instantiate an xpublish socket and use
//! methods from the [`Sink`]/[`SinkExt`] and [`Stream`]/[`StreamExt`] traits.
//!
//! An xpublish socket must be paired with a [`subscribe`] or [`xsubscribe`] socket.
//!
//! # Example
//!
//! ```no_run
//! use async_zmq::{Result, SinkExt};
//!
//! #[async_std::main]
//! async fn main() -> Result<()> {
//!     let mut zmq = async_zmq::xpublish("tcp://127.0.0.1:5555")?.bind()?;
//!
//!     zmq.send(vec!["topic", "broadcast message"].into()).await?;
//!     Ok(())
//! }
//! ```
//!
//! [`xpublish`]: fn.xpublish.html
//! [`xsubscribe`]: ../xsubscribe/index.html
//! [`subscribe`]: ../subscribe/index.html
//! [`Sink`]: ../prelude/trait.Sink.html
//! [`SinkExt`]: ../prelude/trait.SinkExt.html
//! [`Stream`]: ../prelude/trait.Stream.html
//! [`StreamExt`]: ../prelude/trait.StreamExt.html

use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{
    reactor::{AsRawSocket, ZmqSocket},
    socket::{Broker, Multipart, MultipartIter, SocketBuilder},
    SendError, Sink, SocketError, Stream,
};
use zmq::{SocketType, Message};

/// Create a ZMQ socket with XPUB type
pub fn xpublish<I: Iterator<Item=T> + Unpin, T: Into<Message>>(endpoint: &str) -> Result<SocketBuilder<'_, XPublish<I, T>>, SocketError> {
    Ok(SocketBuilder::new(SocketType::XPUB, endpoint))
}

/// The async wrapper of ZMQ socket with XPUB type
pub struct XPublish<I: Iterator<Item=T> + Unpin, T: Into<Message>>(Broker<I, T>);

impl<I: Iterator<Item=T> + Unpin, T: Into<Message>> XPublish<I, T> {
    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        &self.0.socket.as_socket()
    }
}

impl<I: Iterator<Item=T> + Unpin, T: Into<Message>> Sink<MultipartIter<I,T>> for XPublish<I, T> {
    type Error = SendError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::poll_ready(Pin::new(&mut self.get_mut().0), cx)
            .map(|result| result.map_err(Into::into))
    }

    fn start_send(self: Pin<&mut Self>, item: MultipartIter<I,T>) -> Result<(), Self::Error> {
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

impl<I: Iterator<Item=T> + Unpin, T: Into<Message>> Stream for XPublish<I, T> {
    type Item = Result<Multipart, SendError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0)
            .poll_next(cx)
            .map(|poll| poll.map(|result| result.map_err(Into::into)))
    }
}

impl<I: Iterator<Item=T> + Unpin, T: Into<Message>> From<zmq::Socket> for XPublish<I, T> {
    fn from(socket: zmq::Socket) -> Self {
        Self(Broker {
            socket: ZmqSocket::from(socket),
            buffer: None,
        })
    }
}
