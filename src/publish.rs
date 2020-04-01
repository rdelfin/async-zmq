//! PUB socket module of Pub-Sub pattern in ZMQ
//!
//! Use [`publish`] function to instantiate a publisher and the you will be able to use methods from [`Sink`]/[`SinkExt`] trait.
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
//!     zmq.send(vec!["topic", "broadcast message"]).await?;
//!     Ok(())
//! }
//! ```
//!
//! [`publish`]: fn.publish.html
//! [`Sink`]: ../prelude/trait.Sink.html
//! [`SinkExt`]: ../prelude/trait.SinkExt.html

use std::pin::Pin;
use std::task::{Context, Poll};

use zmq::{Error, SocketType};

use crate::socket::{MessageBuf, Sender, SocketBuilder, SocketEvented};
use crate::Sink;

/// Create a ZMQ socket with PUB type
pub fn publish(endpoint: &str) -> Result<SocketBuilder<'_, Publish>, zmq::Error> {
    let socket = zmq::Context::new().socket(SocketType::PUB)?;

    Ok(SocketBuilder::new(socket, endpoint))
}

/// The async wrapper of ZMQ socket with PUB type
pub struct Publish(Sender);

impl Publish {
    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        &self.0.socket.get_ref()
    }
}

impl<T: Into<MessageBuf>> Sink<T> for Publish {
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

impl From<zmq::Socket> for Publish {
    fn from(socket: zmq::Socket) -> Self {
        Self(Sender {
            socket: SocketEvented::from(socket),
            buffer: MessageBuf::default(),
        })
    }
}
