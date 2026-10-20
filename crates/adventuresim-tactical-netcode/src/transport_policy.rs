//! Shared finite session buffering for tree-bank, vista and replication traffic.

use aeronet::transport::TransportConfig;

/// A ceiling on buffered session bytes, not a reservation or process RAM claim.
const TACTICAL_TRANSPORT_BYTES: usize = 32 * 1024 * 1024;

#[cfg(any(feature = "client", feature = "server"))]
pub(crate) fn tactical_transport_config() -> TransportConfig {
    TransportConfig {
        max_memory_usage: TACTICAL_TRANSPORT_BYTES,
        ..Default::default()
    }
}

/// Required before scene asset delivery on an accepted connection.
pub fn tactical_asset_transport_ready(config: &TransportConfig) -> bool {
    config.max_memory_usage == TACTICAL_TRANSPORT_BYTES
}
