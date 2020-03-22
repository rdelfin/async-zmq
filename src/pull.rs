//! Pull socket module of Push/Pull pattern in ZMQ
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
//!     let mut zmq = async_zmq::pull("tcp://127.0.0.1:2020")?;
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

use zmq::{SocketType, Error};

use crate::Stream;
use crate::socket::{AsRaw, MessageBuf, Reciever, ZmqSocket};

/// Create a ZMQ socket with SUB type
pub fn pull(endpoint: &str) -> Result<Pull, zmq::Error> {
    let socket = zmq::Context::new().socket(SocketType::PULL)?;

    socket.connect(endpoint)?;

    Ok(Pull::from(socket))
}

/// The async wrapper of ZMQ socket with SUB type
pub struct Pull(Reciever);

impl AsRaw for Pull {
    fn as_raw_socket(&self) -> &zmq::Socket {
        &self.0.socket.get_ref().0
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
