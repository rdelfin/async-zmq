//! REP socket module of Request-reply pattern in ZMQ
//!
//! Use [`reply`] function to instantiate a REP socket.
//!
//! # Example
//!
//! ```no_run
//! use async_zmq::Result;
//!
//! #[async_std::main]
//! async fn main() -> Result<()> {
//!     let mut zmq = async_zmq::reply("tcp://127.0.0.1:5555")?.bind()?;
//!
//!     let msg = zmq.recv().await?;
//!     zmq.send(vec!["broadcast message"]).await?;
//!     Ok(())
//! }
//! ```
//!
//! [`reply`]: fn.reply.html

use std::sync::atomic::{AtomicBool, Ordering};

use zmq::{Error, SocketType};

use crate::socket::{MessageBuf, Sender, SocketBuilder, ZmqSocket};

/// Create a ZMQ socket with REP type
pub fn reply(endpoint: &str) -> Result<SocketBuilder<'_, Reply>, zmq::Error> {
    let socket = zmq::Context::new().socket(SocketType::REP)?;

    Ok(SocketBuilder::new(socket, endpoint))
}

/// The async wrapper of ZMQ socket with REP type
pub struct Reply {
    inner: Sender,
    received: AtomicBool,
}

impl From<zmq::Socket> for Reply {
    fn from(socket: zmq::Socket) -> Self {
        Self {
            inner: Sender {
                socket: ZmqSocket::from(socket),
                buffer: MessageBuf::default(),
            },
            received: AtomicBool::new(false),
        }
    }
}

impl Reply {
    /// Receive request from REQ/DEALER socket. This should be the first method to be called, and then
    /// continue with receive/send pattern in synchronous way.
    pub async fn recv(&self) -> Result<MessageBuf, Error> {
        let msg = async_std::future::poll_fn(|cx| self.inner.socket.recv(cx)).await?;
        self.received.store(true, Ordering::Relaxed);
        Ok(msg)
    }

    /// Send reply to REQ/DEALER socket. [`recv`](#method.recv) must be called first in order to reply.
    pub async fn send<T: Into<MessageBuf>>(&self, msg: T) -> Result<(), Error> {
        let mut msg = msg.into();
        let res =
            async_std::future::poll_fn(move |cx| self.inner.socket.send(cx, &mut msg)).await?;
        self.received.store(false, Ordering::Relaxed);
        Ok(res)
    }

    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        &self.inner.socket.get_ref().0
    }
}
