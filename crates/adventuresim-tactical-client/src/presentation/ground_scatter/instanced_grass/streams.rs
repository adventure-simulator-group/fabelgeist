//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const CELL: StreamId = StreamId::new("visual.ground-scatter.instanced-grass.cell");
pub(super) const TUFT: StreamId = StreamId::new("visual.ground-scatter.instanced-grass.tuft");
pub(super) const TUFT_MESH: StreamId =
    StreamId::new("visual.ground-scatter.instanced-grass.tuft-mesh");
