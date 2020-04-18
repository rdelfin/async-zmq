//! SUB socket module of Pub-Sub pattern in ZMQ
//!
//! Use the [`subscribe`] function to instantiate a subscribe socket and use
//! methods from the [`Stream`]/[`StreamExt`] traits.
//!
//! A subscribe socket must be paired with a [`publish`] or [`xpublish`] socket.
//!
//! # Example
//!
//! ```no_run
//! use async_zmq::{Result, StreamExt};
//!
//! #[async_std::main]
//! async fn main() -> Result<()> {
//!     let mut zmq = async_zmq::subscribe("tcp://127.0.0.1:5555")?.connect()?;
//!
//!     // Subscribe the topic you want to listen.
//!     // Users can subscribe multiple topics and even unsubscribe later.
//!     zmq.set_subscribe("topic")?;
//!
//!     while let Some(msg) = zmq.next().await {
//!         // Received message is a type of Result<MessageBuf>
//!         let msg = msg?;
//!
//!         println!("{:?}", msg.iter());
//!     }
//!     Ok(())
//! }
//! ```
//!
//! [`xpublish`]: ../xpublish/index.html
//! [`publish`]: ../publish/index.html
//! [`subscribe`]: fn.subscribe.html
//! [`Stream`]: ../trait.Stream.html
//! [`StreamExt`]: ../trait.StreamExt.html

use std::pin::Pin;
use std::task::{Context, Poll};

use zmq::SocketType;

use crate::{
    reactor::{AsRawSocket, ZmqSocket},
    socket::{Multipart, Receiver, SocketBuilder},
    RecvError, SocketError, Stream, SubscribeError,
};

/// Create a ZMQ socket with SUB type
pub fn subscribe(endpoint: &str) -> Result<SocketBuilder<'_, Subscribe>, SocketError> {
    Ok(SocketBuilder::new(SocketType::SUB, endpoint))
}

/// The async wrapper of ZMQ socket with SUB type
pub struct Subscribe(Receiver);

impl From<zmq::Socket> for Subscribe {
    fn from(socket: zmq::Socket) -> Self {
        Self(Receiver {
            socket: ZmqSocket::from(socket),
        })
    }
}

impl Stream for Subscribe {
    type Item = Result<Multipart, RecvError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0)
            .poll_next(cx)
            .map(|poll| poll.map(|result| result.map_err(Into::into)))
    }
}

impl Subscribe {
    /// Subscribe a topic to the socket
    pub fn set_subscribe(&self, topic: &str) -> Result<(), SubscribeError> {
        Ok(self.as_raw_socket().set_subscribe(topic.as_bytes())?)
    }

    /// Remove a topic from the socket
    pub fn set_unsubscribe(&self, topic: &str) -> Result<(), SubscribeError> {
        Ok(self.as_raw_socket().set_unsubscribe(topic.as_bytes())?)
    }

    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        &self.0.socket.as_socket()
    }
}
