use std::time::Duration;

use async_std::task::{spawn};

use async_zmq::{push, Result, SinkExt, StreamExt, pull, MessageBuf};

#[async_std::test]
async fn push_pull_message() -> Result<()> {
    let uri = "tcp://0.0.0.0:2020";
    let mut push = push(uri)?;
    let mut pull = pull(uri)?;
    let message = vec!["Hello", "World"];
    let expected = message.clone();

    let send_handle = spawn( async move {
        push.send(message).await.unwrap();
    });

    let receive_handle = spawn( async move {
        async_std::task::sleep(Duration::from_millis(1000)).await;
        while let Some(recv) = pull.next().await {
            let recv = recv.unwrap();
            assert_eq!(recv, expected.iter().map(|i| i.into()).collect::<MessageBuf>());
            break;
        }
    });

    send_handle.await;
    receive_handle.await;
    Ok(())
}
