//! Error types for the AI Pocket core domain.

use thiserror::Error;

/// The unified error type returned by `ai-pocket-core` APIs.
///
/// Higher layers (transport, FFI, server, desktop) are expected to wrap or
/// convert this into their own error space.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CoreError {
    /// A control message or frame failed to (de)serialize.
    #[error("codec error: {0}")]
    Codec(String),

    /// A session transition was requested that is not valid for the current state.
    #[error("invalid session transition: from {from:?} via {event}")]
    InvalidTransition {
        /// The state the session was in when the transition was attempted.
        from: crate::session::SessionState,
        /// A short human-readable name of the event that was rejected.
        event: &'static str,
    },

    /// The referenced session id is unknown.
    #[error("unknown session: {0}")]
    UnknownSession(crate::session::SessionId),

    /// A device descriptor was malformed or incomplete.
    #[error("invalid device descriptor: {0}")]
    InvalidDescriptor(String),

    /// The negotiated video codec is not supported.
    ///
    /// AI Pocket locks the video codec to H.264; any other codec is rejected here.
    #[error("unsupported codec: {0}")]
    UnsupportedCodec(String),
}

/// Convenience alias for results produced by core APIs.
pub type CoreResult<T> = Result<T, CoreError>;
