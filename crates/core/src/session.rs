//! Session identity and the streaming session state machine.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::{CoreError, CoreResult};

/// Opaque identifier for a streaming session.
///
/// Wraps a `u64` so the type is cheap to copy and compare while still being
/// distinct from arbitrary integers at the type level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SessionId(u64);

impl SessionId {
    /// Creates a session id from its raw numeric value.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw numeric value of this id.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "session#{}", self.0)
    }
}

/// Lifecycle states of a streaming session.
///
/// The legal transitions are encoded in [`Session::apply`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SessionState {
    /// Created but not yet handshaken.
    Idle,
    /// Handshake completed; ready to stream but not streaming.
    Ready,
    /// Actively streaming media.
    Streaming,
    /// Cleanly closed; terminal state.
    Closed,
}

/// Events that drive a [`Session`] between [`SessionState`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SessionEvent {
    /// The handshake completed successfully.
    Handshaked,
    /// A start request was accepted.
    Started,
    /// A stop request was accepted.
    Stopped,
    /// The session was torn down.
    Closed,
}

impl SessionEvent {
    /// 事件的稳定短名，用于错误信息（避免在 error 里依赖 Debug 排版）。
    fn name(self) -> &'static str {
        match self {
            SessionEvent::Handshaked => "handshaked",
            SessionEvent::Started => "started",
            SessionEvent::Stopped => "stopped",
            SessionEvent::Closed => "closed",
        }
    }
}

/// A single streaming session and its current lifecycle state.
#[derive(Debug, Clone)]
pub struct Session {
    id: SessionId,
    state: SessionState,
}

impl Session {
    /// Creates a new session in the [`SessionState::Idle`] state.
    pub fn new(id: SessionId) -> Self {
        Self {
            id,
            state: SessionState::Idle,
        }
    }

    /// Returns this session's id.
    pub fn id(&self) -> SessionId {
        self.id
    }

    /// Returns the current lifecycle state.
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Applies a lifecycle event, advancing the state machine.
    ///
    /// Returns the new state on success, or [`CoreError::InvalidTransition`]
    /// if the event is not legal from the current state.
    pub fn apply(&mut self, event: SessionEvent) -> CoreResult<SessionState> {
        use SessionEvent as Ev;
        use SessionState as St;

        // 合法转移表：任何未列出的 (state, event) 组合都视为非法转移。
        let next = match (self.state, event) {
            (St::Idle, Ev::Handshaked) => St::Ready,
            (St::Ready, Ev::Started) => St::Streaming,
            (St::Streaming, Ev::Stopped) => St::Ready,
            // 任意非终态都允许被关闭。
            (St::Idle | St::Ready | St::Streaming, Ev::Closed) => St::Closed,
            (from, ev) => {
                return Err(CoreError::InvalidTransition {
                    from,
                    event: ev.name(),
                });
            }
        };

        self.state = next;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_transitions() {
        // 验证标准生命周期：idle -> ready -> streaming -> ready -> closed。
        let mut s = Session::new(SessionId::new(1));
        assert_eq!(s.state(), SessionState::Idle);
        assert_eq!(
            s.apply(SessionEvent::Handshaked).unwrap(),
            SessionState::Ready
        );
        assert_eq!(
            s.apply(SessionEvent::Started).unwrap(),
            SessionState::Streaming
        );
        assert_eq!(s.apply(SessionEvent::Stopped).unwrap(), SessionState::Ready);
        assert_eq!(s.apply(SessionEvent::Closed).unwrap(), SessionState::Closed);
    }

    #[test]
    fn illegal_transition_rejected() {
        // 未握手就 Start 必须被拒绝。
        let mut s = Session::new(SessionId::new(2));
        let err = s.apply(SessionEvent::Started).unwrap_err();
        assert!(matches!(err, CoreError::InvalidTransition { .. }));
    }
}
