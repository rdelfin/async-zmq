//!
//! # Errors
//!
//! We define a separate error type for three classes of operation:
//!  * socket creation
//!  * sending messages
//!  * receiving messages
//!
//! We provide an error variant for each ØMQ error code that could occur for
//! the given operation, excluding codes that are not relevant to the
//! operation, are handled internally, or are not possible due to the design of
//! this library.
//!
//! In each case we also provide a catch-all `Unexpected` variant. This variant
//! indicates that an error code was produced by ØMQ which should not have been
//! possible in the situation. Encountering this error variant should be
//! considered a bug; in ØMQ itself, in its documentation or, most likely, in
//! this crate.
//!

use thiserror::Error;

/// The type of errors that can occur when creating a new ØMQ socket.
#[derive(Clone, Copy, Debug, Error)]
pub enum SocketError {
    /// The requested socket type is invalid.
    /// Corresponds to ØMQ error code `EINVAL`.
    #[error("the requested socket type is invalid")]
    InvalidSocketType,

    /// The provided context is invalid.
    /// Corresponds to ØMQ error code `EFAULT`.
    #[error("the provided context is invalid")]
    InvalidContext,

    /// The limit on the total number of open ØMQ sockets has been reached.
    /// Corresponds to ØMQ error code `EMFILE`.
    #[error("the limit on the total number of open ØMQ sockets has been reached")]
    SocketLimitReached,

    /// The context specified was terminated.
    /// Corresponds to ØMQ error code `ETERM`.
    #[error("the context specified was terminated")]
    ContextTerminated,

    /// ØMQ produced an error variant that is not documented to occur when
    /// creating a new socket. This should never happen and should be treated
    /// as a bug.
    #[error("an unexpected error occurred: {0}")]
    Unexpected(#[source] zmq::Error),
}

impl SocketError {
    fn to_zmq_error(self) -> zmq::Error {
        match self {
            SocketError::InvalidSocketType => zmq::Error::EINVAL,
            SocketError::InvalidContext => zmq::Error::EFAULT,
            SocketError::SocketLimitReached => zmq::Error::EMFILE,
            SocketError::ContextTerminated => zmq::Error::ETERM,
            SocketError::Unexpected(error) => error,
        }
    }
}

impl From<SocketError> for zmq::Error {
    fn from(other: SocketError) -> Self {
        other.to_zmq_error()
    }
}

impl From<zmq::Error> for SocketError {
    fn from(other: zmq::Error) -> Self {
        match other {
            zmq::Error::EINVAL => SocketError::InvalidSocketType,
            zmq::Error::EFAULT => SocketError::InvalidContext,
            zmq::Error::EMFILE => SocketError::SocketLimitReached,
            zmq::Error::ETERM => SocketError::ContextTerminated,
            error => SocketError::Unexpected(error),
        }
    }
}

/// The type of errors that can occur when sending a ØMQ message.
///
/// The following ØMQ error codes may occur in the underlying ØMQ implementation,
/// but do not need to be handled by users of this crate:
///
///  * `EAGAIN` - this crate will automatically retry if this error code is
///     produced
///  * `ENOTSUP` - unsupported operations are prevented by the design of this
///     crate
///  * `EINVAL` - multipart messages are not yet supported
///  * `ENOTSOCK` - the design of this crate prevents sending messages on an
///     invalid socket
///  * `EFSM` - this applies only to REP/REQ sockets which have their own error
///     type
#[derive(Clone, Copy, Debug, Error)]
pub enum SendError {
    /// The ØMQ context associated with the specified socket was terminated.
    ///
    /// Note that this error cannot occur unless you access the raw socket
    /// using `as_raw_socket()`.
    ///
    /// Corresponds to ØMQ error code `ETERM`
    #[error("the context specified was terminated")]
    ContextTerminated,

    /// The host is unreachable and the message cannot be routed.
    ///
    /// Corresponds to ØMQ error code `EHOSTUNREACH`.
    #[error("the message cannot be routed")]
    HostUnreachable,

    /// The message is invalid.
    ///
    /// Corresponds to ØMQ error code `EFAULT`.
    #[error("the message is invalid")]
    InvalidMessage,

    /// The operation was interrupted by delivery of a signal before the
    /// message was sent.
    ///
    /// Corresponds to ØMQ error code `EINTR`.
    /// TODO verify if this can actually occur, or if it can be handled
    /// internally with a retry
    #[error("the operation was interrupted by delivery of a signal before the message was sent")]
    Interrupted,

    /// ØMQ produced an error variant that is not documented to occur when
    /// sending a message. This should never happen and should be treated as a
    /// bug.
    #[error("an unexpected error occurred: {0}")]
    Unexpected(#[source] zmq::Error),
}

impl SendError {
    fn to_zmq_error(self) -> zmq::Error {
        match self {
            SendError::ContextTerminated => zmq::Error::ETERM,
            SendError::HostUnreachable => zmq::Error::EHOSTUNREACH,
            SendError::InvalidMessage => zmq::Error::EFAULT,
            SendError::Interrupted => zmq::Error::EINTR,
            SendError::Unexpected(error) => error,
        }
    }
}

impl From<SendError> for zmq::Error {
    fn from(other: SendError) -> Self {
        other.to_zmq_error()
    }
}

impl From<zmq::Error> for SendError {
    fn from(other: zmq::Error) -> Self {
        match other {
            zmq::Error::ETERM => SendError::ContextTerminated,
            zmq::Error::EHOSTUNREACH => SendError::HostUnreachable,
            zmq::Error::EFAULT => SendError::InvalidMessage,
            zmq::Error::EINTR => SendError::Interrupted,
            error => SendError::Unexpected(error),
        }
    }
}

/// The type of errors that can occur when receiving a ØMQ message.
///
/// The following ØMQ error codes may occur in the underlying ØMQ implementation,
/// but do not need to be handled by users of this crate:
///
///  * `EAGAIN` - this crate will automatically retry if this error code is
///     produced
///  * `ENOTSUP` - unsupported operations are prevented by the design of this
///     crate
///  * `EINVAL` - multipart messages are not yet supported
///  * `ENOTSOCK` - the design of this crate prevents sending messages on an
///     invalid socket
///  * `EFSM` - this applies only to REP/REQ sockets which have their own error
///     type
#[derive(Clone, Copy, Debug, Error)]
pub enum RecvError {
    /// The ØMQ context associated with the specified socket was terminated.
    ///
    /// Note that this error cannot occur unless you access the raw socket
    /// using `as_raw_socket()`.
    ///
    /// Corresponds to ØMQ error code `ETERM`
    #[error("the context specified was terminated")]
    ContextTerminated,

    /// The operation was interrupted by delivery of a signal before the
    /// message was received.
    ///
    /// Corresponds to ØMQ error code `EINTR`.
    /// TODO verify if this can actually occur, or if it can be handled
    /// internally
    #[error(
        "the operation was interrupted by delivery of a signal before the message was received"
    )]
    Interrupted,

    /// ØMQ produced an error variant that is not documented to occur when
    /// receiving a message. This should never happen and should be treated as
    /// a bug.
    #[error("an unexpected error occurred: {0}")]
    Unexpected(#[source] zmq::Error),
}

impl RecvError {
    fn to_zmq_error(self) -> zmq::Error {
        match self {
            RecvError::ContextTerminated => zmq::Error::ETERM,
            RecvError::Interrupted => zmq::Error::EINTR,
            RecvError::Unexpected(error) => error,
        }
    }
}

impl From<RecvError> for zmq::Error {
    fn from(other: RecvError) -> Self {
        other.to_zmq_error()
    }
}

impl From<zmq::Error> for RecvError {
    fn from(other: zmq::Error) -> Self {
        match other {
            zmq::Error::ETERM => RecvError::ContextTerminated,
            zmq::Error::EINTR => RecvError::Interrupted,
            error => RecvError::Unexpected(error),
        }
    }
}

/// The type of errors that can occur when sending on a request channel.
///
/// The following ØMQ error codes may occur in the underlying ØMQ implementation,
/// but do not need to be handled by users of this crate:
///
///  * `EAGAIN` - this crate will automatically retry if this error code is
///     produced
///  * `ENOTSUP` - unsupported operations are prevented by the design of this
///     crate
///  * `EINVAL` - multipart messages are not yet supported
///  * `ENOTSOCK` - the design of this crate prevents sending messages on an
///     invalid socket
#[derive(Clone, Copy, Debug, Error)]
pub enum RequestReplyError {
    /// The socket was in the incorrect state for the operation.
    ///
    /// This can occur when sending a message on a request or reply socket when
    /// the socket is in a waiting state.
    ///
    /// Corresponds to ØMQ error code `EFSM`.
    #[error("this socket cannot send when it is awaiting a reply")]
    AwaitingReply,

    /// The ØMQ context associated with the specified socket was terminated.
    ///
    /// Note that this error cannot occur unless you access the raw socket
    /// using `as_raw_socket()`.
    ///
    /// Corresponds to ØMQ error code `ETERM`
    #[error("the context specified was terminated")]
    ContextTerminated,

    /// The host is unreachable and the message cannot be routed.
    ///
    /// Corresponds to ØMQ error code `EHOSTUNREACH`.
    #[error("the message cannot be routed")]
    HostUnreachable,

    /// The operation was interrupted by delivery of a signal before the
    /// message was sent.
    ///
    /// Corresponds to ØMQ error code `EINTR`.
    /// TODO verify if this can actually occur, or if it can be handled
    /// internally with a retry
    #[error("the operation was interrupted by delivery of a signal before the message was sent")]
    Interrupted,

    /// ØMQ produced an error variant that is not documented to occur when
    /// sending a message. This should never happen and should be treated as a
    /// bug.
    #[error("an unexpected error occurred: {0}")]
    Unexpected(#[source] zmq::Error),
}

impl RequestReplyError {
    fn to_zmq_error(self) -> zmq::Error {
        match self {
            RequestReplyError::AwaitingReply => zmq::Error::EFSM,
            RequestReplyError::ContextTerminated => zmq::Error::ETERM,
            RequestReplyError::HostUnreachable => zmq::Error::EHOSTUNREACH,
            RequestReplyError::Interrupted => zmq::Error::EINTR,
            RequestReplyError::Unexpected(error) => error,
        }
    }
}

impl From<RequestReplyError> for zmq::Error {
    fn from(other: RequestReplyError) -> Self {
        other.to_zmq_error()
    }
}

impl From<zmq::Error> for RequestReplyError {
    fn from(other: zmq::Error) -> Self {
        match other {
            zmq::Error::EFSM => RequestReplyError::AwaitingReply,
            zmq::Error::ETERM => RequestReplyError::ContextTerminated,
            zmq::Error::EHOSTUNREACH => RequestReplyError::HostUnreachable,
            zmq::Error::EINTR => RequestReplyError::Interrupted,
            error => RequestReplyError::Unexpected(error),
        }
    }
}

/// The type of errors that can occur when setting or unsetting a subscription
/// topic.
///
/// The following ØMQ error codes may occur in the underlying ØMQ implementation,
/// but do not need to be handled by users of this crate:
///
///  * `EINVAL` - the option name is always correct
///  * `ENOTSOCK` - the design of this crate prevents sending messages on an
///     invalid socket
#[derive(Clone, Copy, Debug, Error)]
pub enum SubscribeError {
    /// The ØMQ context associated with the specified socket was terminated.
    ///
    /// Note that this error cannot occur unless you access the raw socket
    /// using `as_raw_socket()`.
    ///
    /// Corresponds to ØMQ error code `ETERM`
    #[error("the context specified was terminated")]
    ContextTerminated,

    /// The operation was interrupted by delivery of a signal before the
    /// message was sent.
    ///
    /// Corresponds to ØMQ error code `EINTR`.
    /// TODO verify if this can actually occur, or if it can be handled
    /// internally with a retry
    #[error("the operation was interrupted by delivery of a signal before the message was sent")]
    Interrupted,

    /// ØMQ produced an error variant that is not documented to occur when
    /// sending a message. This should never happen and should be treated as a
    /// bug.
    #[error("an unexpected error occurred: {0}")]
    Unexpected(#[source] zmq::Error),
}

impl SubscribeError {
    fn to_zmq_error(self) -> zmq::Error {
        match self {
            SubscribeError::ContextTerminated => zmq::Error::ETERM,
            SubscribeError::Interrupted => zmq::Error::EINTR,
            SubscribeError::Unexpected(error) => error,
        }
    }
}

impl From<SubscribeError> for zmq::Error {
    fn from(other: SubscribeError) -> Self {
        other.to_zmq_error()
    }
}

impl From<zmq::Error> for SubscribeError {
    fn from(other: zmq::Error) -> Self {
        match other {
            zmq::Error::ETERM => SubscribeError::ContextTerminated,
            zmq::Error::EINTR => SubscribeError::Interrupted,
            error => SubscribeError::Unexpected(error),
        }
    }
}
