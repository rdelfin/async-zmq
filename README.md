# Async version for ZeroMQ bindings

[![][crates-badge]][crates-url] ![][license-badge] ![][build-bade]

[crates-badge]: https://img.shields.io/crates/v/async-zmq
[crates-url]: https://crates.io/crates/async_zmq
[license-badge]: https://img.shields.io/crates/l/async-zmq
[build-bade]: https://img.shields.io/github/workflow/status/wusyong/async-zmq/Main

This is high-level bindings for [`zmq`] in asynchronous manner. Crate itself uses some modules from
[`async-std`], but it should also work on any other async reactor. The goal for this project
is providing simple interface that is compatible with any async executor and reactor.

## TODO list

- [X] PAIR
- [x] PUB
- [x] SUB
- [x] REQ
- [x] REP
- [x] DEALER
- [x] ROUTER
- [x] PULL
- [x] PUSH
- [x] XPUB
- [x] XSUB
- [x] STREAM
- [ ] More tests

## Usage

Users could simply initialize any socket type with `async_zmq::*` in mind, and then call
`bind()` or `connect` depends on your scenario. For example, if someone wants a publish socket,
then he could initialize the socket like this:

```
let zmq = async_zmq::publish("tcp://127.0.0.1:5555")?.bind();
```

To learn more about each socket type usage. See [modules](#modules) below.

### Prelude

Prelude module provides some common types, traits and their methods. This crate also re-export
so it can be easier for you to import them.

Another common issue when people adopting a library is to deal with its error handling flow.
To prevent introducing more overhead, `async_zmq` uses the exact same [`Result`]/[`Error`] type
in [`zmq`] crate and re-export them.

[`Result`]: prelude/type.Result.html
[`Error`]: prelude/type.Error.html
[`zmq`]: https://crates.io/crates/zmq
[`async-std`]: https://crates.io/crates/async-std