//! XSUB socket module of Pub-Sub pattern in ZMQ
//!
//! Use the [`xsubscribe`] function to instantiate an xsubscribe socket and use
//! methods from the [`Stream`]/[`StreamExt`] trait.
//!
//! An xsubscribe socket must be paired with a [`publish`] or [`xpublish`] socket.
//!
//! # Example
//!
//! ```no_run
//! use async_zmq::{Result, StreamExt};
//!
//! #[async_std::main]
//! async fn main() -> Result<()> {
//!     let mut zmq = async_zmq::xsubscribe("tcp://127.0.0.1:5555")?.bind()?;
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
//! [`xsubscribe`]: fn.xsubscribe.html
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

/// Create a ZMQ socket with XSUB type
pub fn xsubscribe(endpoint: &str) -> Result<SocketBuilder<'_, XSubscribe>, SocketError> {
    Ok(SocketBuilder::new(SocketType::XSUB, endpoint))
}

/// The async wrapper of ZMQ socket with XSUB type
pub struct XSubscribe(Receiver);

impl From<zmq::Socket> for XSubscribe {
    fn from(socket: zmq::Socket) -> Self {
        Self(Receiver {
            socket: ZmqSocket::from(socket),
        })
    }
}

impl Stream for XSubscribe {
    type Item = Result<Multipart, RecvError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0)
            .poll_next(cx)
            .map(|poll| poll.map(|result| result.map_err(Into::into)))
    }
}

impl XSubscribe {
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
        self.0.socket.as_socket()
    }
}
