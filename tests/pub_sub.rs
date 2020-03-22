use std::time::Duration;

use async_std::sync::{Arc, Barrier};
use async_std::task::{spawn};

use async_zmq::{publish, Result, SinkExt, StreamExt, subscribe, MessageBuf};

#[async_std::test]
async fn publish_subribe_topic() -> Result<()> {
    let uri = "tcp://0.0.0.0:2020";
    let mut socket = publish(uri)?;
    let topic = "Topic";
    let message = vec![topic, "Hello", "World"];
    let expected = message.clone();
    let barrier = Arc::new(Barrier::new(2));
    let handle = barrier.clone();

    let join = spawn( async move {
        let mut socket = subscribe(&uri).unwrap();
        socket.set_subscribe(&topic).unwrap();
        handle.wait().await;
        
        async_std::task::sleep(Duration::from_millis(2000)).await;
        let recieved = socket.next().await.unwrap().unwrap();
        assert_eq!(recieved, expected.iter().map(|i| i.into()).collect::<MessageBuf>());
        }
    );

    barrier.wait().await;
    async_std::task::sleep(Duration::from_millis(1000)).await;
    socket.send(message).await.unwrap();
    join.await;
    Ok(())
}
