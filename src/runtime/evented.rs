use std::io;

use mio::unix::EventedFd;
use mio::{Evented, Poll, PollOpt, Ready, Token};
use zmq::Socket;

pub(crate) struct ZmqSocket(pub(crate) Socket);
/*
impl ZmqSocket {
    pub fn new(socket: Socket) -> Self {
        Self(socket)
    }
}
*/

impl Evented for ZmqSocket {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.0.get_fd()?).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.0.get_fd()?).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.0.get_fd()?).deregister(poll)
    }
}
