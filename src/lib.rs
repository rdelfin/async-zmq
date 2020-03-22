//! # Async version for ZeroMQ bindings
//! 
//! This is high-level bindings for [`zmq`] in asynchronous manner. Crate itself uses some modules from
//! [`async-std`], but it should also work on any other async runtime. The goal for this project
//! is providing simple interface that is compatible with any async executor and reactor.
//! 
//! ## TODO list
//! 
//! - [ ] PAIR
//! - [x] PUB
//! - [x] SUB
//! - [ ] REQ
//! - [ ] REP
//! - [ ] DEALER
//! - [ ] ROUTER
//! - [ ] PULL
//! - [ ] PUSH
//! - [ ] XPUB
//! - [ ] XSUB
//! - [ ] STREAM
//! 
//! ## Usage
//! 
//! Users could simply initialize any socket type with `async_zmq::*` in mind. For example, if
//! someone wants a publish socket, then he could call the function:
//! 
//! ```ignore
//! let zmq = async_zmq::publish("tcp://127.0.0.1:2020")?;
//! ```
//! 
//! To learn more about each socket type usage. See [modules](#modules) below.
//! 
//! ### Prelude
//! 
//! Prelude module provides some common types, traits and their methods. This crate also re-export
//! so it can be easier for you to import them.
//! 
//! Another common issue when people adopting a library is to deal with its error handling flow.
//! To prevent introducing more overhead, `async_zmq` uses the exact same [`Result`]/[`Error`] type
//! in [`zmq`] crate and re-export them.
//! 
//! [`Result`]: prelude/type.Result.html
//! [`Error`]: prelude/type.Error.html
//! [`zmq`]: https://crates.io/crates/zmq
//! [`async-std`]: https://crates.io/crates/async-std

#![deny(unused_extern_crates, unsafe_code)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]

pub mod publish;
pub mod subscribe;
pub mod push;
pub mod pull;

mod evented;
mod socket;
mod watcher;

/// The prelude re-exports most commonly used traits and macros from this crate.
pub mod prelude {
    pub use crate::publish::{publish, Publish};
    pub use crate::socket::MessageBuf;
    pub use crate::subscribe::{subscribe, Subscribe};
    pub use crate::pull::{pull, Pull};
    pub use crate::push::{push, Push};
    pub use async_std::stream::{Stream, StreamExt};
    pub use futures_util::sink::{Sink, SinkExt};
    pub use zmq::{self, Error, Message, Result};
}

pub use prelude::*;
