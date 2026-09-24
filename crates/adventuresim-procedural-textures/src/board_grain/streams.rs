//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const ARCH_X: StreamId = StreamId::new("texture.board-grain.arch-x");
pub(super) const ARCH_Y: StreamId = StreamId::new("texture.board-grain.arch-y");
pub(super) const KNOT_PRESENCE: StreamId = StreamId::new("texture.board-grain.knot-presence");
pub(super) const KNOT_X: StreamId = StreamId::new("texture.board-grain.knot-x");
pub(super) const KNOT_Y: StreamId = StreamId::new("texture.board-grain.knot-y");
pub(super) const RING_COLOR: StreamId = StreamId::new("texture.board-grain.ring-color");
pub(super) const RING_PHASE: StreamId = StreamId::new("texture.board-grain.ring-phase");
pub(super) const RING_WANDER: StreamId = StreamId::new("texture.board-grain.ring-wander");
pub(super) const RING_WIDTH: StreamId = StreamId::new("texture.board-grain.ring-width");
pub(super) const SAWN_ARCH_PRESENCE: StreamId =
    StreamId::new("texture.board-grain.sawn-arch-presence");
