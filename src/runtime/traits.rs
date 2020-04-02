use std::task::Context;

///
/// Define all actions possible on a socket
///
pub(crate) trait InnerSocket: Sized
where
    Self: AsSocket,
{
    type Error;

    type Item;

    type Request;

    type Response;

    ///
    /// Send buffer data
    ///
    fn send(&self, cx: &mut Context<'_>, buffer: &mut Self::Item) -> Self::Request;

    ///
    /// Recive data
    ///
    fn recv(&self, cx: &mut Context<'_>) -> Self::Response;

    //    fn stream(self) -> Self::Stream;
    //
    //    fn sink(self, buffer_sized: usize) -> Self::Sink
    //
    //    fn sink_stream(self, buffer_sized: usize) -> SinkStream;
}

///
/// Into Socket
///
pub(crate) trait AsSocket {
    ///
    /// Socket
    ///
    fn as_socket(&self) -> &zmq::Socket;
}
