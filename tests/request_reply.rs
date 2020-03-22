use std::time::Duration;

use async_zmq::{reply, request, Message, Result};

#[async_std::test]
async fn publish_subscribe_message() -> Result<()> {
    let uri = "tcp://0.0.0.0:5555";
    let request = request(uri)?;
    let reply = reply(uri)?;
    let request_message = "Hello";
    let reply_message = "World";

    request.send(Message::from(request_message)).await?;

    async_std::task::sleep(Duration::from_millis(1000)).await;

    let recv = reply.recv().await?;
    assert_eq!(recv[0].as_str().unwrap(), request_message);
    reply.send(Message::from(reply_message)).await?;

    async_std::task::sleep(Duration::from_millis(1000)).await;

    let recv = request.recv().await?;
    assert_eq!(recv[0].as_str().unwrap(), reply_message);

    Ok(())
}
