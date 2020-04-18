//! # Async version for ZeroMQ bindings
//!
//! This is high-level bindings for [`zmq`] in asynchronous manner. Crate itself uses some modules from
//! [`async-std`], but it should also work on any other async reactor. The goal for this project
//! is providing simple interface that is compatible with any async executor and reactor.
//!
//! ## Usage
//!
//! Users could simply initialize any socket type with `async_zmq::*` in mind, and then call
//! `bind()` or `connect` depends on your scenario. For example, if someone wants a publish socket,
//! then he could initialize the socket like this:
//!
//! ```ignore
//! let zmq = async_zmq::publish("tcp://127.0.0.1:5555")?.bind();
//! ```
//!
//! If there's context need to be shared between different socket, we can set it during building the socket:
//!
//! ```ignore
//! let context = Context::new();
//! let xpub = async_zmq::xpublish("inproc://example")?.with_context(&context).bind();
//! let sub = subscribe("inproc://example")?.with_context(&context).connect()?;
//! ```
//! 
//! Since the use case of this crate is mostly for sending/recieving multipart message. So it provides [`Multipart`]
//! which is a type alias for `Vec<Message>` when recieving message on type implemented with `Stream`, and [`MultipartIter`]
//! which is a generic struct make any queue can turn into iterator and then send via type  implemented with `Sink`.
//!
//! To learn more about each socket type usage. See [modules](#modules) below.
//!
//! [`Result`]: type.Result.html
//! [`zmq`]: https://crates.io/crates/zmq
//! [`async-std`]: https://crates.io/crates/async-std
//! [`Multipart`]: type.Multipart.html
//! [`MultipartIter`]: struct.MultipartIter.html


#![deny(unused_extern_crates, unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]

pub mod dealer;
pub mod errors;
pub mod pair;
pub mod publish;
pub mod pull;
pub mod push;
pub mod reply;
pub mod request;
pub mod router;
pub mod stream;
pub mod subscribe;
pub mod xpublish;
pub mod xsubscribe;

mod reactor;
mod socket;

pub use crate::dealer::{dealer, Dealer};
pub use crate::errors::*;
pub use crate::pair::{pair, Pair};
pub use crate::publish::{publish, Publish};
pub use crate::pull::{pull, Pull};
pub use crate::push::{push, Push};
pub use crate::reactor::AsRawSocket;
pub use crate::reply::{reply, Reply};
pub use crate::request::{request, Request};
pub use crate::router::{router, Router};
pub use crate::socket::{Multipart, MultipartIter, SocketBuilder};
pub use crate::stream::{stream, ZmqStream};
pub use crate::subscribe::{subscribe, Subscribe};
pub use crate::xpublish::{xpublish, XPublish};
pub use crate::xsubscribe::{xsubscribe, XSubscribe};
pub use futures::sink::{Sink, SinkExt};
pub use futures::stream::{Stream, StreamExt};
pub use zmq::{self, Context, Error, Message, Result};
