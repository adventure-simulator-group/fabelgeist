//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const CELL: StreamId = StreamId::new("texture.surface.cell");
pub(super) const CRACK_OFFSET: StreamId = StreamId::new("texture.surface.crack-offset");
pub(super) const CRACK_PHASE: StreamId = StreamId::new("texture.surface.crack-phase");
pub(super) const CRACK_SECONDARY_PHASE: StreamId =
    StreamId::new("texture.surface.crack-secondary-phase");
pub(super) const EDGE: StreamId = StreamId::new("texture.surface.edge");
pub(super) const EDGE_FREQUENCY: StreamId = StreamId::new("texture.surface.edge-frequency");
pub(super) const EDGE_MEANDER: StreamId = StreamId::new("texture.surface.edge-meander");
pub(super) const EDGE_PHASE: StreamId = StreamId::new("texture.surface.edge-phase");
pub(super) const LATTICE: StreamId = StreamId::new("texture.surface.lattice");
