//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const ANGLE: StreamId = StreamId::new("texture.ground.scatter.angle");
pub(super) const ASPECT: StreamId = StreamId::new("texture.ground.scatter.aspect");
pub(super) const CENTER_X: StreamId = StreamId::new("texture.ground.scatter.center-x");
pub(super) const CENTER_Y: StreamId = StreamId::new("texture.ground.scatter.center-y");
pub(super) const LEAF_DECAY: StreamId = StreamId::new("texture.ground.scatter.leaf-decay");
pub(super) const LEAF_NOISE: StreamId = StreamId::new("texture.ground.scatter.leaf-noise");
pub(super) const LEAF_PATCH: StreamId = StreamId::new("texture.ground.scatter.leaf-patch");
pub(super) const LEAF_SIDE: StreamId = StreamId::new("texture.ground.scatter.leaf-side");
pub(super) const PHASE: StreamId = StreamId::new("texture.ground.scatter.phase");
pub(super) const PIGMENT: StreamId = StreamId::new("texture.ground.scatter.pigment");
pub(super) const PRESENCE: StreamId = StreamId::new("texture.ground.scatter.presence");
pub(super) const RADIUS: StreamId = StreamId::new("texture.ground.scatter.radius");
