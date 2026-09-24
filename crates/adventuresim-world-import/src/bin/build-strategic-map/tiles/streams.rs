//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const BOUNDARY_DETAIL_BROAD: StreamId = StreamId::new("map.boundary-detail-broad");
pub(super) const BOUNDARY_DETAIL_FINE: StreamId = StreamId::new("map.boundary-detail-fine");
pub(super) const BOUNDARY_WARP_X_BROAD: StreamId = StreamId::new("map.boundary-warp-x-broad");
pub(super) const BOUNDARY_WARP_X_FINE: StreamId = StreamId::new("map.boundary-warp-x-fine");
pub(super) const BOUNDARY_WARP_Y_BROAD: StreamId = StreamId::new("map.boundary-warp-y-broad");
pub(super) const BOUNDARY_WARP_Y_FINE: StreamId = StreamId::new("map.boundary-warp-y-fine");
pub(super) const LATTICE: StreamId = StreamId::new("map.lattice");
pub(super) const NOISE_OCTAVE: StreamId = StreamId::new("map.noise-octave");
pub(super) const ORGANIC_VERTEX: StreamId = StreamId::new("map.organic-vertex");
pub(super) const PARCHMENT: StreamId = StreamId::new("map.parchment");
pub(super) const RELIEF_DETAIL: StreamId = StreamId::new("map.relief-detail");
pub(super) const RELIEF_WARP_X: StreamId = StreamId::new("map.relief-warp-x");
pub(super) const RELIEF_WARP_Y: StreamId = StreamId::new("map.relief-warp-y");
