//! REP socket module of Request-reply pattern in ZMQ
//!
//! Use the [`reply`] function to instantiate a reply socket and use methods from
//! the [`Stream`]/[`StreamExt`] traits.
//!
//! A reply socket must be paired with a [`request`] or [`dealer`] socket.
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
//! [`dealer`]: ../dealer/index.html
//! [`request`]: ../request/index.html
//! [`reply`]: fn.reply.html

use std::{
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};

use zmq::{Message, SocketType};

use crate::{
    reactor::{AsRawSocket, ZmqSocket},
    socket::{Multipart, MultipartIter, Sender, SocketBuilder},
    RecvError, RequestReplyError, SocketError,
};

use futures::{future::poll_fn, Stream};

/// Create a ZMQ socket with REP type
pub fn reply<I: Iterator<Item = T> + Unpin, T: Into<Message>>(
    endpoint: &str,
) -> Result<SocketBuilder<'_, Reply<I, T>>, SocketError> {
    Ok(SocketBuilder::new(SocketType::REP, endpoint))
}

/// The async wrapper of ZMQ socket with REP type
pub struct Reply<I: Iterator<Item = T> + Unpin, T: Into<Message>> {
    inner: Sender<I, T>,
    received: AtomicBool,
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> From<zmq::Socket> for Reply<I, T> {
    fn from(socket: zmq::Socket) -> Self {
        Self {
            inner: Sender {
                socket: ZmqSocket::from(socket),
                buffer: None,
            },
            received: AtomicBool::new(false),
        }
    }
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Reply<I, T> {
    /// Receive request from REQ/DEALER socket. This should be the first method to be called, and then
    /// continue with receive/send pattern in synchronous way.
    pub async fn recv(&self) -> Result<Multipart, RequestReplyError> {
        let msg = poll_fn(|cx| self.inner.socket.recv(cx)).await?;
        self.received.store(true, Ordering::Relaxed);
        Ok(msg)
    }

    /// Send reply to REQ/DEALER socket. [`recv`](#method.recv) must be called first in order to reply.
    pub async fn send<S: Into<MultipartIter<I, T>>>(
        &self,
        msg: S,
    ) -> Result<(), RequestReplyError> {
        let mut msg = msg.into();
        let res = poll_fn(move |cx| self.inner.socket.send(cx, &mut msg)).await?;
        self.received.store(false, Ordering::Relaxed);
        Ok(res)
    }

    /// Represent as `Socket` from zmq crate in case you want to call its methods.
    pub fn as_raw_socket(&self) -> &zmq::Socket {
        &self.inner.socket.as_socket()
    }
}

impl<I: Iterator<Item = T> + Unpin, T: Into<Message>> Stream for Reply<I, T> {
    type Item = Result<Multipart, RecvError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(Some(Ok(futures::ready!(self.inner.socket.recv(cx))?)))
    }
}
