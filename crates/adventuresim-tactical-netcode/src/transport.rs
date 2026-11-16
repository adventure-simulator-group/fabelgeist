//! A bounded per-connection budget for canonical town initialization.

use aeronet::transport::TransportConfig;
use bevy::prelude::*;

// A town's accepted support surface and vista can each exceed the transport's
// 4-MiB default. Allow their reliable initialization plus fragment overhead,
// without making the connection's send/receive queues unbounded.
const SCENE_CONNECTION_MEMORY_BYTES: usize = 64 * 1024 * 1024;

pub(crate) fn configure_scene_transport(
    event: On<Add, TransportConfig>,
    mut configurations: Query<&mut TransportConfig>,
) -> Result {
    configurations.get_mut(event.entity)?.max_memory_usage = SCENE_CONNECTION_MEMORY_BYTES;
    Ok(())
}
