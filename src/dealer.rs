//! DEALER socket module of Request-reply pattern in ZMQ
//!
//! Use [`dealer`] function to instantiate a dealerer and the you will be able to use methods from
//! both [`Sink`]/[`SinkExt`] and [`Stream`]/[`StreamExt`] traits.
//!
//! # Example
//!
//! ```no_run
//! ```
//!
//! [`dealer`]: fn.dealer.html
//! [`Sink`]: ../prelude/trait.Sink.html
//! [`SinkExt`]: ../prelude/trait.SinkExt.html
//! [`Stream`]: ../prelude/trait.Stream.html
//! [`StreamExt`]: ../prelude/trait.StreamExt.html

use std::pin::Pin;
use std::task::{Context, Poll};

use zmq::{Error, SocketType};

use crate::socket::{MessageBuf, Broker, ZmqSocket};
use crate::{Sink, Stream};

/// Create a ZMQ socket with DEALER type
pub fn dealer(endpoint: &str) -> Result<Dealer, zmq::Error> {
    let socket = zmq::Context::new().socket(SocketType::DEALER)?;

    socket.connect(endpoint)?;

    Ok(Dealer::from(socket))
}

/// The async wrapper of ZMQ socket with DEALER type
pub struct Dealer(Broker);

impl Dealer {
    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        &self.0.socket.get_ref().0
    }
}

impl<T: Into<MessageBuf>> Sink<T> for Dealer {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<T>::poll_ready(Pin::new(&mut self.get_mut().0), cx)
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        Pin::new(&mut self.get_mut().0).start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Sink::<T>::poll_flush(Pin::new(&mut self.get_mut().0), cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Sink::<T>::poll_close(Pin::new(&mut self.get_mut().0), cx)
    }
}

impl Stream for Dealer {
    type Item = Result<MessageBuf, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0).poll_next(cx)
    }
}

impl From<zmq::Socket> for Dealer {
    fn from(socket: zmq::Socket) -> Self {
        Self(Broker {
            socket: ZmqSocket::from(socket),
            buffer: MessageBuf::default(),
        })
    }
}
