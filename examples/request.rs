//! cargo run --example request --features="rt-tokio" --no-default-features

use async_zmq::Result;
use std::ops::Deref;

#[tokio::main]
async fn main() -> Result<()> {
    let zmq = async_zmq::request("tcp://127.0.0.1:5555")?.connect()?;
    zmq.send(vec!["broadcast message"]).await?;
    let msg = zmq.recv().await?;

    for it in msg.deref() {
        println!("{:?}", it.as_str());
    }

    Ok(())
}
