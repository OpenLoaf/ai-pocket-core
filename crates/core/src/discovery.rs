//! Device discovery descriptors shared by all transports.

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};
use crate::proto::{PROTOCOL_VERSION, VideoCodec};

/// The role a discovered peer plays in a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DeviceRole {
    /// Desktop / mobile sender that captures and encodes video.
    Sender,
    /// Hardware sink (e.g. SG2002 / ESP32-P4) that decodes and displays.
    Sink,
    /// Relay node that forwards traffic over the public network.
    Relay,
}

/// A self-description advertised by a device during discovery.
///
/// Produced on the local-network discovery plane (mDNS-style) and over the
/// relay registration plane; consumers should treat it as untrusted input and
/// call [`DeviceDescriptor::validate`] before use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceDescriptor {
    /// Stable unique id of the device (e.g. a hashed hardware serial).
    pub device_id: String,
    /// Human-friendly display name.
    pub display_name: String,
    /// The role this device plays.
    pub role: DeviceRole,
    /// Highest protocol version the device speaks.
    pub protocol_version: u16,
    /// Video codecs the device can produce or consume.
    pub codecs: Vec<VideoCodec>,
}

impl DeviceDescriptor {
    /// Validates the descriptor for internal consistency.
    ///
    /// Returns [`CoreError::InvalidDescriptor`] if required fields are empty,
    /// or [`CoreError::UnsupportedCodec`] if the device advertises no H.264
    /// support (AI Pocket requires H.264).
    pub fn validate(&self) -> CoreResult<()> {
        // 设备 id 与展示名是路由与 UI 的最小必填项。
        if self.device_id.trim().is_empty() {
            return Err(CoreError::InvalidDescriptor("empty device_id".into()));
        }
        if self.display_name.trim().is_empty() {
            return Err(CoreError::InvalidDescriptor("empty display_name".into()));
        }
        // 协议版本不能高于本端已知版本，否则无法协商。
        if self.protocol_version > PROTOCOL_VERSION {
            return Err(CoreError::InvalidDescriptor(format!(
                "protocol_version {} exceeds supported {}",
                self.protocol_version, PROTOCOL_VERSION
            )));
        }
        // AI Pocket 锁 H.264：不支持 H.264 的设备直接拒绝。
        if !self.codecs.contains(&VideoCodec::H264) {
            return Err(CoreError::UnsupportedCodec(
                "device does not advertise H264".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> DeviceDescriptor {
        DeviceDescriptor {
            device_id: "dev-001".into(),
            display_name: "Living Room TV".into(),
            role: DeviceRole::Sink,
            protocol_version: PROTOCOL_VERSION,
            codecs: vec![VideoCodec::H264],
        }
    }

    #[test]
    fn valid_descriptor_passes() {
        assert!(sample().validate().is_ok());
    }

    #[test]
    fn missing_h264_rejected() {
        // 没有 H.264 的设备应被拒绝。
        let mut d = sample();
        d.codecs.clear();
        assert!(matches!(
            d.validate().unwrap_err(),
            CoreError::UnsupportedCodec(_)
        ));
    }
}
