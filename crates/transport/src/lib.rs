//! # ai-pocket-transport
//!
//! Async transport abstractions for AI Pocket built on top of
//! [`ai_pocket_core`]. This crate defines two traits that decouple the rest of
//! the system from any concrete networking stack:
//!
//! - [`Discovery`] — local-network device discovery (mDNS-style).
//! - [`RelayClient`] — a client to the public-network relay server.
//!
//! Concrete implementations live behind feature flags or in downstream crates
//! (the server and desktop apps). A minimal in-process [`stub`] implementation
//! is provided for tests when the `stub` feature is enabled.

#![forbid(unsafe_code)]

use std::future::Future;

use ai_pocket_core::{ControlMsg, DeviceDescriptor, H264Frame, SessionId};
use thiserror::Error;

/// Errors produced by transport implementations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TransportError {
    /// A lower-level I/O failure.
    #[error("io error: {0}")]
    Io(String),

    /// The peer or relay is unreachable.
    #[error("peer unreachable")]
    Unreachable,

    /// The transport was closed by the peer or locally.
    #[error("transport closed")]
    Closed,

    /// An error originating in the shared core domain.
    #[error(transparent)]
    Core(#[from] ai_pocket_core::CoreError),
}

/// Convenience alias for transport results.
pub type TransportResult<T> = Result<T, TransportError>;

/// Local-network device discovery.
///
/// Implementations browse the LAN (typically via mDNS) and surface
/// [`DeviceDescriptor`]s for peers that can participate in a session.
pub trait Discovery: Send + Sync {
    /// Starts advertising this device so peers can find it.
    fn advertise(
        &self,
        descriptor: DeviceDescriptor,
    ) -> impl Future<Output = TransportResult<()>> + Send;

    /// Returns a snapshot of peers discovered so far.
    fn peers(&self) -> impl Future<Output = TransportResult<Vec<DeviceDescriptor>>> + Send;
}

/// Client to the public-network relay server.
///
/// Used when two peers cannot reach each other directly on the LAN and must
/// route control messages and [`H264Frame`]s through the relay.
pub trait RelayClient: Send + Sync {
    /// Connects to the relay and registers this device.
    fn connect(
        &self,
        descriptor: DeviceDescriptor,
    ) -> impl Future<Output = TransportResult<()>> + Send;

    /// Sends a control message for the given session.
    fn send_control(
        &self,
        session: SessionId,
        msg: ControlMsg,
    ) -> impl Future<Output = TransportResult<()>> + Send;

    /// Sends an encoded H.264 frame for the given session.
    fn send_frame(&self, frame: H264Frame) -> impl Future<Output = TransportResult<()>> + Send;
}

#[cfg(any(feature = "stub", test))]
pub mod stub {
    //! 进程内回环 stub 实现，仅用于测试 / 无网络环境。
    //!
    //! 这些实现不做任何真实网络 I/O：discovery 维护内存里的 peer 列表，
    //! relay client 只把调用记到内部计数，方便上层在没有真机时跑通流程。

    use super::*;
    use std::sync::Mutex;
    use tokio::sync::Mutex as AsyncMutex;

    /// In-memory [`Discovery`] used in tests.
    #[derive(Default)]
    pub struct StubDiscovery {
        // 已发现/已广播的 peer，受互斥锁保护。
        peers: Mutex<Vec<DeviceDescriptor>>,
    }

    impl Discovery for StubDiscovery {
        async fn advertise(&self, descriptor: DeviceDescriptor) -> TransportResult<()> {
            // 广播即把自己加入本地 peer 表，便于回环测试观察。
            descriptor.validate()?;
            self.peers.lock().expect("poisoned").push(descriptor);
            Ok(())
        }

        async fn peers(&self) -> TransportResult<Vec<DeviceDescriptor>> {
            Ok(self.peers.lock().expect("poisoned").clone())
        }
    }

    /// In-memory [`RelayClient`] used in tests; counts what it would have sent.
    #[derive(Default)]
    pub struct StubRelayClient {
        // 记录发出的控制消息与帧数，断言用。
        sent_control: AsyncMutex<u64>,
        sent_frames: AsyncMutex<u64>,
    }

    impl StubRelayClient {
        /// Returns the number of control messages "sent".
        pub async fn control_count(&self) -> u64 {
            *self.sent_control.lock().await
        }

        /// Returns the number of frames "sent".
        pub async fn frame_count(&self) -> u64 {
            *self.sent_frames.lock().await
        }
    }

    impl RelayClient for StubRelayClient {
        async fn connect(&self, descriptor: DeviceDescriptor) -> TransportResult<()> {
            descriptor.validate()?;
            Ok(())
        }

        async fn send_control(&self, _session: SessionId, _msg: ControlMsg) -> TransportResult<()> {
            *self.sent_control.lock().await += 1;
            Ok(())
        }

        async fn send_frame(&self, _frame: H264Frame) -> TransportResult<()> {
            *self.sent_frames.lock().await += 1;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_pocket_core::{DeviceRole, FrameKind, PROTOCOL_VERSION, VideoCodec};

    fn descriptor() -> DeviceDescriptor {
        DeviceDescriptor {
            device_id: "dev-x".into(),
            display_name: "Test".into(),
            role: DeviceRole::Sender,
            protocol_version: PROTOCOL_VERSION,
            codecs: vec![VideoCodec::H264],
        }
    }

    #[tokio::test]
    async fn stub_discovery_roundtrip() {
        // 验证 stub discovery 的广播/列举回环。
        let d = stub::StubDiscovery::default();
        d.advertise(descriptor()).await.unwrap();
        assert_eq!(d.peers().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn stub_relay_counts() {
        // 验证 stub relay 的发送计数。
        let c = stub::StubRelayClient::default();
        c.connect(descriptor()).await.unwrap();
        c.send_control(SessionId::new(1), ControlMsg::Ping { nonce: 1 })
            .await
            .unwrap();
        c.send_frame(H264Frame {
            session: SessionId::new(1),
            seq: 0,
            pts_90khz: 0,
            kind: FrameKind::Key,
            payload: vec![],
        })
        .await
        .unwrap();
        assert_eq!(c.control_count().await, 1);
        assert_eq!(c.frame_count().await, 1);
    }
}
