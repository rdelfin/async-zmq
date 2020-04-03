use std::task::Context;

/// Define all actions possible on a socket
pub(crate) trait InnerSocket: Sized
where
    Self: AsSocket,
{
    type Error;

    type Item;

    type Request;

    type Response;

    /// Send buffer data
    fn send(&self, cx: &mut Context<'_>, buffer: &mut Self::Item) -> Self::Request;

    /// Recive data
    fn recv(&self, cx: &mut Context<'_>) -> Self::Response;
}

/// Into Socket
pub(crate) trait AsSocket {
    /// Socket
    fn as_socket(&self) -> &zmq::Socket;
}
