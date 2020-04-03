//!
//! Runtimes settup module
//!
#[cfg(feature = "rt-async")]
mod async_std_zmq;
pub(crate) mod evented;
#[cfg(feature = "rt-tokio")]
mod tokio_zmq;
mod traits;

#[cfg(feature = "rt-async")]
pub(crate) use async_std_zmq::ZmqSocket;
#[cfg(feature = "rt-tokio")]
pub(crate) use tokio_zmq::ZmqSocket;
pub(crate) use traits::{InnerSocket, AsSocket};
