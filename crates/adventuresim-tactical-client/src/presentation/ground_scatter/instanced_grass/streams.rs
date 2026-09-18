//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const CELL: StreamId = StreamId::new("visual.ground-scatter.instanced-grass.cell");
pub(super) const JITTER_X: StreamId =
    StreamId::new("visual.ground-scatter.instanced-grass.jitter-x");
pub(super) const JITTER_Z: StreamId =
    StreamId::new("visual.ground-scatter.instanced-grass.jitter-z");
pub(super) const SHADER_SEED: StreamId =
    StreamId::new("visual.ground-scatter.instanced-grass.shader-seed");
pub(super) const SPECIES: StreamId = StreamId::new("visual.ground-scatter.instanced-grass.species");
pub(super) const TUFT: StreamId = StreamId::new("visual.ground-scatter.instanced-grass.tuft");
pub(super) const TUFT_MESH: StreamId =
    StreamId::new("visual.ground-scatter.instanced-grass.tuft-mesh");
pub(super) const YAW: StreamId = StreamId::new("visual.ground-scatter.instanced-grass.yaw");
