//! SUB socket module of Pub-Sub pattern in ZMQ
//!
//! Use [`subscribe`] function to instantiate a subscriber and the you will be able to use methods from [`Stream`]/[`StreamExt`] trait.
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
//! [`subscribe`]: fn.subscribe.html
//! [`Stream`]: ../prelude/trait.Stream.html
//! [`StreamExt`]: ../prelude/trait.StreamExt.html

use std::pin::Pin;
use std::task::{Context, Poll};

use zmq::{Error, SocketType};

use crate::{
    reactor::{AsSocket, ZmqSocket},
    socket::{MessageBuf, Reciever, SocketBuilder},
    Stream,
};

/// Create a ZMQ socket with SUB type
pub fn subscribe(endpoint: &str) -> Result<SocketBuilder<'_, Subscribe>, zmq::Error> {
    let socket = zmq::Context::new().socket(SocketType::SUB)?;

    Ok(SocketBuilder::new(socket, endpoint))
}

/// The async wrapper of ZMQ socket with SUB type
pub struct Subscribe(Reciever);

impl From<zmq::Socket> for Subscribe {
    fn from(socket: zmq::Socket) -> Self {
        Self(Reciever {
            socket: ZmqSocket::from(socket),
        })
    }
}

impl Stream for Subscribe {
    type Item = Result<MessageBuf, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0).poll_next(cx)
    }
}

impl Subscribe {
    /// Subscribe a topic to the socket
    pub fn set_subscribe(&self, topic: &str) -> Result<(), zmq::Error> {
        self.as_raw_socket().set_subscribe(topic.as_bytes())
    }

    /// Remove a topic from the socket
    pub fn set_unsubscribe(&self, topic: &str) -> Result<(), zmq::Error> {
        self.as_raw_socket().set_unsubscribe(topic.as_bytes())
    }

    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        &self.0.socket.as_socket()
    }
}
