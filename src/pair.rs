//! PAIR socket module of Exclusive pair pattern in ZMQ
//!
//! Use [`pair`] function to instantiate a pair socket and the you will be able to use methods from
//! both [`Sink`]/[`SinkExt`] and [`Stream`]/[`StreamExt`] traits.
//!
//! # Example
//!
//! ```no_run
//! use async_zmq::{Result, SinkExt};
//!
//! #[async_std::main]
//! async fn main() -> Result<()> {
//!     let mut zmq = async_zmq::pair("tcp://127.0.0.1:5555")?.bind()?;
//!
//!     zmq.send(vec!["broadcast message"]).await?;
//!     Ok(())
//! }
//! ```
//!
//! [`pair`]: fn.pair.html
//! [`Sink`]: ../prelude/trait.Sink.html
//! [`SinkExt`]: ../prelude/trait.SinkExt.html
//! [`Stream`]: ../prelude/trait.Stream.html
//! [`StreamExt`]: ../prelude/trait.StreamExt.html

use std::pin::Pin;
use std::task::{Context, Poll};

use zmq::{Error, SocketType};

use crate::{
    reactor::{AsRawSocket, ZmqSocket},
    socket::{Broker, MessageBuf, SocketBuilder},
    Sink, Stream,
};

/// Create a ZMQ socket with PAIR type
pub fn pair(endpoint: &str) -> Result<SocketBuilder<'_, Pair>, zmq::Error> {
    Ok(SocketBuilder::new(SocketType::PAIR, endpoint))
}

/// The async wrapper of ZMQ socket with PAIR type
pub struct Pair(Broker);

impl Pair {
    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        &self.0.socket.as_socket()
    }
}

impl<T: Into<MessageBuf>> Sink<T> for Pair {
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

impl Stream for Pair {
    type Item = Result<MessageBuf, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0).poll_next(cx)
    }
}

impl From<zmq::Socket> for Pair {
    fn from(socket: zmq::Socket) -> Self {
        Self(Broker {
            socket: ZmqSocket::from(socket),
            buffer: MessageBuf::default(),
        })
    }
}
