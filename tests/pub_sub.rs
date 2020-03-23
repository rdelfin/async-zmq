use async_std::sync::{Arc, Mutex};
use async_std::task::spawn;

use async_zmq::{publish, subscribe, MessageBuf, Result, SinkExt, StreamExt};

#[async_std::test]
async fn publish_subscribe_message() -> Result<()> {
    let uri = "tcp://0.0.0.0:5555";
    let mut publish = publish(uri)?.bind()?;
    let mut subscribe = subscribe(uri)?.connect()?;
    let topic = "Topic";
    subscribe.set_subscribe(topic)?;
    let message = vec![topic, "Hello", "World"];
    let expected = message.clone();
    let running = Arc::new(Mutex::new(true));
    let notify = running.clone();

    let send_handle = spawn(async move {
        while *running.lock().await {
            let _ = publish.send(message.clone()).await;
        }
    });

    let receive_handle = spawn(async move {
        while let Some(recv) = subscribe.next().await {
            if let Ok(recv) = recv {
                assert_eq!(
                    recv,
                    expected.iter().map(|i| i.into()).collect::<MessageBuf>()
                );
                *notify.lock().await = false;
                break;
            }
        }
    });

    send_handle.await;
    receive_handle.await;
    Ok(())
}
