//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const ANGLE: StreamId = StreamId::new("texture.stamps.angle");
pub(super) const CLUSTER: StreamId = StreamId::new("texture.stamps.cluster");
pub(super) const DEPTH: StreamId = StreamId::new("texture.stamps.depth");
pub(super) const LATTICE: StreamId = StreamId::new("texture.stamps.lattice");
pub(super) const PRESENCE: StreamId = StreamId::new("texture.stamps.presence");
pub(super) const SITE_X: StreamId = StreamId::new("texture.stamps.site-x");
pub(super) const SITE_Y: StreamId = StreamId::new("texture.stamps.site-y");
pub(super) const SIZE: StreamId = StreamId::new("texture.stamps.size");
