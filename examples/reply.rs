//! cargo run --example reply --features="rt-tokio" --no-default-features

use async_zmq::Result;
use futures::StreamExt;
use std::ops::Deref;

#[tokio::main]
async fn main() -> Result<()> {
    let mut zmq = async_zmq::reply("tcp://127.0.0.1:5555")?.bind()?;

    while let Some(msg) = zmq.next().await {
        for it in msg.unwrap().deref() {
            println!("message: {:?}", it.as_str());
        }
        zmq.send(vec!["Response for You"]).await?;
    }

    Ok(())
}
