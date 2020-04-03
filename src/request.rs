//! REQ socket module of Request-request pattern in ZMQ
//!
//! Use [`request`] function to instantiate a REQ socket.
//!
//! # Example
//!
//! ```no_run
//! use async_zmq::Result;
//!
//! #[async_std::main]
//! async fn main() -> Result<()> {
//!     let mut zmq = async_zmq::request("tcp://127.0.0.1:5555")?.connect()?;
//!
//!     zmq.send(vec!["broadcast message"]).await?;
//!     let msg = zmq.recv().await?;
//!     Ok(())
//! }
//! ```
//!
//! [`request`]: fn.request.html

use crate::{
    reactor::{InnerSocket, AsSocket, ZmqSocket},
    socket::{MessageBuf, Sender, SocketBuilder},
};
use futures::future::poll_fn;
use std::sync::atomic::{AtomicBool, Ordering};
use zmq::{Error, SocketType};

/// Create a ZMQ socket with REQ type
pub fn request(endpoint: &str) -> Result<SocketBuilder<'_, Request>, zmq::Error> {
    let socket = zmq::Context::new().socket(SocketType::REQ)?;

    Ok(SocketBuilder::new(socket, endpoint))
}

/// The async wrapper of ZMQ socket with REQ type
pub struct Request {
    inner: Sender,
    received: AtomicBool,
}

impl From<zmq::Socket> for Request {
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

impl Request {
    /// Send request to REP/ROUTER socket. This should be the first method to be called, and then
    /// continue with send/receive pattern in synchronous way.
    pub async fn send<T: Into<MessageBuf>>(&self, msg: T) -> Result<(), Error> {
        let mut msg = msg.into();
        let res = poll_fn(move |cx| self.inner.socket.send(cx, &mut msg)).await?;
        self.received.store(false, Ordering::Relaxed);
        Ok(res)
    }

    /// Receive reply from REP/ROUTER socket. [`send`](#method.send) must be called first in order to receive reply.
    pub async fn recv(&self) -> Result<MessageBuf, Error> {
        let msg = poll_fn(|cx| self.inner.socket.recv(cx)).await?;
        self.received.store(true, Ordering::Relaxed);
        Ok(msg)
    }

    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub async fn as_raw_socket(&self) -> &zmq::Socket {
        &self.inner.socket.as_socket()
    }
}
