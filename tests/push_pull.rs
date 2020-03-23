use async_std::sync::{Arc, Mutex};
use async_std::task::spawn;

use async_zmq::{pull, push, MessageBuf, Result, SinkExt, StreamExt};

#[async_std::test]
async fn push_pull_message() -> Result<()> {
    let uri = "tcp://0.0.0.0:5565";
    let mut push = push(uri)?.bind()?;
    let mut pull = pull(uri)?.connect()?;
    let message = vec!["Hello", "World"];
    let expected = message.clone();
    let running = Arc::new(Mutex::new(true));
    let notify = running.clone();

    let send_handle = spawn(async move {
        while *running.lock().await {
            let _ = push.send(message.clone()).await;
        }
    });

    let receive_handle = spawn(async move {
        while let Some(recv) = pull.next().await {
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
