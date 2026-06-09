//! # ai-pocket-ffi
//!
//! UniFFI facade that exposes the AI Pocket domain to mobile platforms
//! (iOS via Swift, Android via Kotlin). It re-exposes a small, FFI-friendly
//! surface over [`ai_pocket_core`] and [`ai_pocket_transport`] rather than
//! leaking the full Rust API across the language boundary.
//!
//! Bindings are generated with the `uniffi-bindgen` binary in this crate; see
//! `scripts/gen-bindings.sh`.

// 注册 UniFFI scaffolding（proc-macro 模式，无 .udl）。
uniffi::setup_scaffolding!();

use std::sync::Mutex;

use ai_pocket_core::{Session, SessionEvent, SessionId, SessionState};

/// Errors surfaced across the FFI boundary.
///
/// Kept deliberately coarse so each platform can map it to an idiomatic error.
#[derive(Debug, thiserror::Error, uniffi::Error)]
#[non_exhaustive]
pub enum PocketError {
    /// A session operation was invalid for the current state.
    #[error("invalid session operation: {reason}")]
    InvalidSession {
        /// Human-readable reason for the rejection.
        reason: String,
    },
}

/// The lifecycle state of a session, mirrored for FFI consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum PocketSessionState {
    /// Created but not handshaken.
    Idle,
    /// Ready to stream.
    Ready,
    /// Actively streaming.
    Streaming,
    /// Closed.
    Closed,
}

impl From<SessionState> for PocketSessionState {
    fn from(s: SessionState) -> Self {
        // 把 core 的状态映射到 FFI 暴露的枚举。
        match s {
            SessionState::Idle => PocketSessionState::Idle,
            SessionState::Ready => PocketSessionState::Ready,
            SessionState::Streaming => PocketSessionState::Streaming,
            SessionState::Closed => PocketSessionState::Closed,
            // core 的 SessionState 标了 non_exhaustive：未知新状态先归一到 Closed，
            // 避免跨 crate 编译失败；core 新增状态时应在此显式补一行映射。
            _ => PocketSessionState::Closed,
        }
    }
}

/// A handle to a started session returned to mobile callers.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct PocketSessionHandle {
    /// Raw session id (opaque to callers; useful for logging / correlation).
    pub id: u64,
    /// Current lifecycle state at the time the handle was produced.
    pub state: PocketSessionState,
}

/// The top-level client used by mobile apps to drive AI Pocket.
///
/// This is a thin, stateful facade; real networking is wired in by the host
/// app through the transport layer. The current implementation focuses on the
/// session lifecycle so bindings and call sites can be built end-to-end.
#[derive(uniffi::Object)]
pub struct PocketClient {
    // 单会话占位：内部用互斥锁保护当前会话，后续可扩展为多会话表。
    current: Mutex<Option<Session>>,
}

#[uniffi::export]
impl PocketClient {
    /// Creates a new client.
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            current: Mutex::new(None),
        }
    }

    /// Returns the version of the underlying core crate.
    pub fn version(&self) -> String {
        ai_pocket_core::version().to_string()
    }

    /// Starts a new session with the given numeric id and returns a handle.
    ///
    /// This is a stub: it creates the session and advances it through the
    /// handshake to [`PocketSessionState::Ready`]. Media wiring is left to the
    /// host transport.
    pub fn start_session(&self, id: u64) -> Result<PocketSessionHandle, PocketError> {
        let mut guard = self.current.lock().expect("poisoned");
        let mut session = Session::new(SessionId::new(id));
        // stub：直接走完握手到 Ready，真实实现会在收到 Welcome 后再推进。
        let state =
            session
                .apply(SessionEvent::Handshaked)
                .map_err(|e| PocketError::InvalidSession {
                    reason: e.to_string(),
                })?;
        let handle = PocketSessionHandle {
            id: session.id().get(),
            state: state.into(),
        };
        *guard = Some(session);
        Ok(handle)
    }
}

impl Default for PocketClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_starts_session() {
        // 验证 façade 能创建并推进会话到 Ready。
        let c = PocketClient::new();
        assert!(!c.version().is_empty());
        let h = c.start_session(42).unwrap();
        assert_eq!(h.id, 42);
        assert_eq!(h.state, PocketSessionState::Ready);
    }
}
