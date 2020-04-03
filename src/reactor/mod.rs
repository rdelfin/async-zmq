#[cfg(feature = "rt-async")]
mod async_std_zmq;
pub(crate) mod evented;
#[cfg(feature = "rt-tokio")]
mod tokio_zmq;

#[cfg(feature = "rt-async")]
pub(crate) use async_std_zmq::ZmqSocket;
#[cfg(feature = "rt-tokio")]
pub(crate) use tokio_zmq::ZmqSocket;

/// Trait to get the raw zmq socket.
pub trait AsRawSocket {
    /// Method to get the raw zmq socket reference if users need to use it directly.
    fn as_socket(&self) -> &zmq::Socket;
}
