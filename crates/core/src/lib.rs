//! # ai-pocket-core
//!
//! Shared, transport-agnostic domain core for the AI Pocket screen-casting and
//! capture product. This crate has no I/O dependencies; it defines the data
//! model and state machines that every edge of the system agrees on:
//!
//! - [`proto`] — control signalling ([`proto::ControlMsg`]) and H.264 frame
//!   envelopes ([`proto::H264Frame`]). Video is locked to H.264.
//! - [`session`] — session identity ([`session::SessionId`]) and the streaming
//!   [`session::Session`] state machine.
//! - [`discovery`] — [`discovery::DeviceDescriptor`] used by both the
//!   local-network and relay discovery planes.
//! - [`error`] — the unified [`error::CoreError`] type.
//!
//! Higher layers (`ai-pocket-transport`, `ai-pocket-ffi`, the server, and the
//! desktop app) depend on this crate and add networking, FFI, and UI concerns.

#![forbid(unsafe_code)]

pub mod discovery;
pub mod error;
pub mod proto;
pub mod session;

pub use discovery::{DeviceDescriptor, DeviceRole};
pub use error::{CoreError, CoreResult};
pub use proto::{ControlMsg, FrameKind, H264Frame, PROTOCOL_VERSION, VideoCodec};
pub use session::{Session, SessionEvent, SessionId, SessionState};

/// Returns the semantic version of this crate as compiled.
///
/// Useful for handshakes and diagnostics so peers can log which core build
/// they negotiated against.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_reported() {
        assert!(!version().is_empty());
    }
}
