use std::io;
use mio::unix::EventedFd;
use mio::{Evented, Poll, PollOpt, Ready, Token};
use zmq::Socket;

pub struct MioSocket(Socket);

impl MioSocket {
    pub fn new(socket: Socket) -> Self {
        Self(socket)
    }
}

impl From<Socket> for MioSocket {
    fn from(socket: Socket) -> Self {
        Self(socket)
    }
}

impl From<MioSocket> for Socket {
    fn from(socket: MioSocket) -> Self {
        socket.0
    }
}

impl Evented for MioSocket {
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