//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const BROAD: StreamId = StreamId::new("visual.clouds.broad");
pub(super) const DETAIL: StreamId = StreamId::new("visual.clouds.detail");
pub(super) const ENVIRONMENT: StreamId = StreamId::new("visual.clouds.environment");
pub(super) const FINE: StreamId = StreamId::new("visual.clouds.fine");
pub(super) const LATTICE: StreamId = StreamId::new("visual.clouds.lattice");
pub(super) const MEDIUM: StreamId = StreamId::new("visual.clouds.medium");
pub(super) const VERTICAL: StreamId = StreamId::new("visual.clouds.vertical");
pub(super) const WARP_X: StreamId = StreamId::new("visual.clouds.warp-x");
pub(super) const WARP_Y: StreamId = StreamId::new("visual.clouds.warp-y");
pub(super) const WARP_Z: StreamId = StreamId::new("visual.clouds.warp-z");
