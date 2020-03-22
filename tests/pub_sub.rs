use std::time::Duration;

use async_std::task::spawn;

use async_zmq::{publish, subscribe, MessageBuf, Result, SinkExt, StreamExt};

#[async_std::test]
async fn publish_subscribe_message() -> Result<()> {
    let uri = "tcp://0.0.0.0:5555";
    let mut publish = publish(uri)?;
    let mut subscribe = subscribe(uri)?;
    let topic = "Topic";
    subscribe.set_subscribe(topic)?;
    let message = vec![topic, "Hello", "World"];
    let expected = message.clone();

    let send_handle = spawn(async move {
        publish.send(message).await.unwrap();
    });

    let receive_handle = spawn(async move {
        async_std::task::sleep(Duration::from_millis(1000)).await;

        while let Some(recv) = subscribe.next().await {
            let recv = recv.unwrap();
            assert_eq!(
                recv,
                expected.iter().map(|i| i.into()).collect::<MessageBuf>()
            );
            break;
        }
    });

    send_handle.await;
    receive_handle.await;
    Ok(())
}
