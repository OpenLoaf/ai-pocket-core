//! Wire protocol primitives: control signalling and H.264 frame envelopes.
//!
//! The video codec for AI Pocket is locked to **H.264**. These types describe
//! the control plane (signalling) and the data plane (encoded frame envelopes)
//! that flow between the desktop sender, the relay server, and the device sink.

use serde::{Deserialize, Serialize};

/// Protocol version negotiated during the handshake.
///
/// Bumped on any breaking change to the control or frame wire format.
pub const PROTOCOL_VERSION: u16 = 1;

/// Control-plane signalling messages exchanged over the reliable channel.
///
/// These are codec-agnostic on the control side; the media itself is carried as
/// [`H264Frame`] envelopes on the data channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ControlMsg {
    /// Initial offer from a sender, advertising its protocol version and codec.
    Hello {
        /// Protocol version the sender speaks.
        version: u16,
        /// Codec the sender intends to use; AI Pocket expects [`VideoCodec::H264`].
        codec: VideoCodec,
    },
    /// Acceptance of a [`ControlMsg::Hello`] by the peer.
    Welcome {
        /// Protocol version the peer agreed on.
        version: u16,
        /// Identifier assigned to the newly accepted session.
        session: crate::session::SessionId,
    },
    /// Request to start streaming media for an accepted session.
    Start {
        /// Session the start applies to.
        session: crate::session::SessionId,
    },
    /// Request to stop streaming media for a session.
    Stop {
        /// Session the stop applies to.
        session: crate::session::SessionId,
    },
    /// Liveness probe; the peer is expected to reply with [`ControlMsg::Pong`].
    Ping {
        /// Monotonic nonce echoed back in the matching pong.
        nonce: u64,
    },
    /// Reply to a [`ControlMsg::Ping`].
    Pong {
        /// The nonce copied from the originating ping.
        nonce: u64,
    },
    /// Terminal error notification; the session is considered closed afterwards.
    Error {
        /// Machine-readable error code.
        code: u32,
        /// Human-readable diagnostic message.
        message: String,
    },
}

/// Supported video codecs.
///
/// AI Pocket locks streaming to H.264; the enum exists so the handshake can
/// explicitly reject anything else rather than silently misbehaving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum VideoCodec {
    /// H.264 / AVC — the only codec AI Pocket currently streams.
    H264,
}

/// The kind of an encoded H.264 access unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FrameKind {
    /// IDR / keyframe — decodable without reference to previous frames.
    Key,
    /// Non-IDR predicted frame.
    Delta,
    /// Parameter set carrier (SPS/PPS), no displayable picture.
    Config,
}

/// An envelope wrapping a single encoded H.264 access unit for transport.
///
/// The `payload` carries the raw Annex-B (or AVCC) bytes; this type only adds
/// the metadata needed for ordering, timing, and session routing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct H264Frame {
    /// Session this frame belongs to.
    pub session: crate::session::SessionId,
    /// Monotonically increasing sequence number within the session.
    pub seq: u64,
    /// Presentation timestamp in 90 kHz ticks (RTP-style clock).
    pub pts_90khz: u64,
    /// The kind of access unit carried by `payload`.
    pub kind: FrameKind,
    /// Raw encoded H.264 bytes.
    pub payload: Vec<u8>,
}

impl H264Frame {
    /// Returns `true` if this frame can be decoded independently (a keyframe).
    pub fn is_keyframe(&self) -> bool {
        matches!(self.kind, FrameKind::Key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::SessionId;

    #[test]
    fn control_msg_roundtrips_json() {
        // 控制信令必须能稳定地序列化/反序列化，握手两端依赖该不变量。
        let msg = ControlMsg::Hello {
            version: PROTOCOL_VERSION,
            codec: VideoCodec::H264,
        };
        let s = serde_json::to_string(&msg).unwrap();
        let back: ControlMsg = serde_json::from_str(&s).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn keyframe_predicate() {
        // 关键帧判定供丢包恢复逻辑使用，这里固化语义。
        let f = H264Frame {
            session: SessionId::new(7),
            seq: 0,
            pts_90khz: 0,
            kind: FrameKind::Key,
            payload: vec![0, 0, 0, 1],
        };
        assert!(f.is_keyframe());
    }
}
