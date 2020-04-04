//! STREAM socket module of Pub-Sub pattern in ZMQ
//!
//! Use [`stream`] function to instantiate a stream socket and the you will be able to use methods from [`Stream`]/[`StreamExt`] trait.
//!
//! # Example
//!
//! ```no_run
//! use async_zmq::{Result, StreamExt};
//!
//! #[async_std::main]
//! async fn main() -> Result<()> {
//!     let mut zmq = async_zmq::stream("tcp://127.0.0.1:5555")?.bind()?;
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
//! [`stream`]: fn.stream.html
//! [`Stream`]: ../prelude/trait.Stream.html
//! [`StreamExt`]: ../prelude/trait.StreamExt.html

use std::pin::Pin;
use std::task::{Context, Poll};

use zmq::{Error, SocketType};

use crate::{
    reactor::{AsRawSocket, ZmqSocket},
    socket::{MessageBuf, Receiver, SocketBuilder},
    Stream,
};

/// Create a ZMQ socket with STREAM type
pub fn stream(endpoint: &str) -> Result<SocketBuilder<'_, ZmqStream>, zmq::Error> {
    Ok(SocketBuilder::new(SocketType::STREAM, endpoint))
}

/// The async wrapper of ZMQ socket with STREAM type
pub struct ZmqStream(Receiver);

impl From<zmq::Socket> for ZmqStream {
    fn from(socket: zmq::Socket) -> Self {
        Self(Receiver {
            socket: ZmqSocket::from(socket),
        })
    }
}

impl Stream for ZmqStream {
    type Item = Result<MessageBuf, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0).poll_next(cx)
    }
}

impl ZmqStream {
    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        &self.0.socket.as_socket()
    }
}
