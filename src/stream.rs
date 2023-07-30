//! STREAM socket module of Pub-Sub pattern in ZMQ
//!
//! Use the [`stream`] function to instantiate a stream socket and use methods
//! from the [`Stream`]/[`StreamExt`] traits.
//!
//! A stream socket is for interacting with TCP data from sources other than
//! ZMQ. It is not compatible with any of the other ZMQ socket types.
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
//! [`Stream`]: ../trait.Stream.html
//! [`StreamExt`]: ../trait.StreamExt.html

use std::pin::Pin;
use std::task::{Context, Poll};

use zmq::SocketType;

use crate::{
    reactor::{AsRawSocket, ZmqSocket},
    socket::{Multipart, Receiver, SocketBuilder},
    RecvError, SocketError, Stream,
};

/// Create a ZMQ socket with STREAM type
pub fn stream(endpoint: &str) -> Result<SocketBuilder<'_, ZmqStream>, SocketError> {
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
    type Item = Result<Multipart, RecvError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0)
            .poll_next(cx)
            .map(|poll| poll.map(|result| result.map_err(Into::into)))
    }
}

impl ZmqStream {
    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        self.0.socket.as_socket()
    }
}
