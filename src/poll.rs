use std::task::{Context, Poll};

use anyhow::Result;
use async_std::task::ready;

use crate::socket::{MessageBuf, ZmqSocket};
use crate::watcher::Watcher;

pub type Poller = Watcher<ZmqSocket>;

impl Poller {
    pub fn send(&self, cx: &mut Context<'_>, buffer: &mut MessageBuf) -> Poll<Result<()>> {
        ready!(self.poll_write_ready(cx));
        ready!(self.poll_event(zmq::POLLIN))?;

        while let Some(msg) = buffer.pop_front() {
            let mut flags = zmq::DONTWAIT;
            if !buffer.is_empty() {
                flags |= zmq::SNDMORE;
            }

            match self.get_ref().0.send(msg, flags) {
                Ok(_) => {}
                Err(zmq::Error::EAGAIN) => return Poll::Pending,
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }

        Poll::Ready(Ok(()))
    }

    pub fn recv(&self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        ready!(self.poll_read_ready(cx));
        ready!(self.poll_event(zmq::POLLOUT))?;

        let mut buffer = MessageBuf::new();
        let mut more = true;

        while more {
            let mut msg = zmq::Message::new();
            match self.get_ref().0.recv(&mut msg, zmq::DONTWAIT) {
                Ok(_) => {
                    more = msg.get_more();
                    buffer.push_back(msg);
                }
                Err(zmq::Error::EAGAIN) => return Poll::Pending,
                Err(e) => return Poll::Ready(Err(e.into())),
            }
        }

        Poll::Ready(Ok(()))
    }

    fn poll_event(&self, event: zmq::PollEvents) -> Poll<Result<()>> {
        if self.get_ref().0.get_events()?.contains(event) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}
