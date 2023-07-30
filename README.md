# Async version for ZeroMQ bindings

[![][crates-badge]][crates-url] ![][license-badge] ![][build-bade]

[crates-badge]: https://img.shields.io/crates/v/async-zmq
[crates-url]: https://crates.io/crates/async_zmq
[license-badge]: https://img.shields.io/crates/l/async-zmq
[build-bade]: https://img.shields.io/github/actions/workflow/status/rdelfin/async-zmq/main.yml?branch=main

Async-zmq is high-level bindings for [`zmq`] in asynchronous manner which is compatible to **every** async runtime.
No need for configuring or tuning features. Just plug in and see how it works!

## Usage

Users could simply initialize any socket type with `async_zmq::*` in mind, and then call
`bind()` or `connect` depends on your scenario. For example, if someone wants a publish socket,
then he could initialize the socket like this:

```
let zmq = async_zmq::publish("tcp://127.0.0.1:5555")?.bind();
```

If there's context need to be shared between different socket, we can set it during building the socket:

```
let context = Context::new();
let xpub = async_zmq::xpublish("inproc://example")?.with_context(&context).bind();
let sub = subscribe("inproc://example")?.with_context(&context).connect()?;
```

Since the use case of this crate is mostly for sending/recieving multipart message. So it provides [`Multipart`]
which is a type alias for `Vec<Message>` when recieving message on type implemented with `Stream`, and [`MultipartIter`]
which is a generic struct make any queue can turn into iterator and then send via type  implemented with `Sink`.

To learn more about each socket type usage. See [modules](#modules) below.

[`zmq`]: https://crates.io/crates/zmq
[`async-std`]: https://crates.io/crates/async-std
