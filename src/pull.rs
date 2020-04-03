//! PULL socket module of Pipeline  pattern in ZMQ
//!
//! Use [`pull`] function to instantiate a PULL socket and the you will be able to use methods from [`Stream`]/[`StreamExt`] trait.
//!
//! # Example
//!
//! ```no_run
//! use async_zmq::{Result, StreamExt};
//!
//! #[async_std::main]
//! async fn main() -> Result<()> {
//!     let mut zmq = async_zmq::pull("tcp://127.0.0.1:5555")?.connect()?;
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
//! [`pull`]: fn.pull.html
//! [`Stream`]: ../prelude/trait.Stream.html
//! [`StreamExt`]: ../prelude/trait.StreamExt.html

use std::pin::Pin;
use std::task::{Context, Poll};

use zmq::{Error, SocketType};

use crate::{
    reactor::{AsRawSocket, ZmqSocket},
    socket::{MessageBuf, Reciever, SocketBuilder},
    Stream,
};

/// Create a ZMQ socket with PULL type
pub fn pull(endpoint: &str) -> Result<SocketBuilder<'_, Pull>, zmq::Error> {
    Ok(SocketBuilder::new(SocketType::PULL, endpoint))
}

/// The async wrapper of ZMQ socket with PULL type
pub struct Pull(Reciever);

impl Pull {
    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        &self.0.socket.as_socket()
    }
}

impl From<zmq::Socket> for Pull {
    fn from(socket: zmq::Socket) -> Self {
        Self(Reciever {
            socket: ZmqSocket::from(socket),
        })
    }
}

impl Stream for Pull {
    type Item = Result<MessageBuf, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0).poll_next(cx)
    }
}
