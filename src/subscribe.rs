use zmq::{Context, SocketType, Result};

use crate::socket::{Reciever, ZmqSocket};

pub fn subscribe(endpoint: &str) -> Result<Subscribe> {
    let socket = Context::new()
        .socket(SocketType::SUB)?;
    
    socket.connect(endpoint)?;
    
    Ok(Subscribe(
        Reciever {
            socket: ZmqSocket::from(socket)
        }
    ))
}

pub struct Subscribe(Reciever);

impl Subscribe {
    pub fn subscribe(&self, topic: &str) -> Result<()> {
        self.0.socket.get_ref().0.set_subscribe(topic.as_bytes())
    }

    pub fn unsubscribe(&self, topic: &str) -> Result<()> {
        self.0.socket.get_ref().0.set_unsubscribe(topic.as_bytes())
    }
}