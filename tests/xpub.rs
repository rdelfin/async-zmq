use async_zmq::{subscribe, xpublish, Context, Message, Result, StreamExt};
use std::vec::IntoIter;

#[async_std::test]
async fn xpublish_subscribe() -> Result<()> {
    let uri = "inproc://xpub_xsub";
    let context = Context::new();
    let mut xpublish = xpublish::<IntoIter<Message>, Message>(uri)?
        .with_context(&context)
        .bind()?;
    let subscribe = subscribe(uri)?.with_context(&context).connect()?;

    let topic = "Topic";
    subscribe.set_subscribe(topic)?;

    // It uses StreamExt trait method here simply we want to make sure `MessageBuf` works.
    // It probably makes more sense to call the method from raw zmq socket.
    let event = xpublish.next().await.unwrap().unwrap();
    assert_eq!(&event[0][0], &1);

    let expected = std::str::from_utf8(&event[0][1..]).unwrap();
    assert_eq!(expected, topic);

    Ok(())
}
