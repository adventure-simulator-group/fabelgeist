//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const FAR_ROOT: StreamId = StreamId::new("visual.grass.far-root");
pub(super) const VISTA_ROOT: StreamId = StreamId::new("visual.grass.vista-root");
pub(super) const AGE: StreamId = StreamId::new("visual.ground-scatter.grass.age");
pub(super) const BLADE: StreamId = StreamId::new("visual.ground-scatter.grass.blade");
pub(super) const BLADE_ANGLE: StreamId = StreamId::new("visual.ground-scatter.grass.blade-angle");
pub(super) const BLADE_PLACEMENT: StreamId =
    StreamId::new("visual.ground-scatter.grass.blade-placement");
pub(super) const BLADE_STYLE: StreamId = StreamId::new("visual.ground-scatter.grass.blade-style");
pub(super) const BLADE_THRESHOLD: StreamId =
    StreamId::new("visual.ground-scatter.grass.blade-threshold");
pub(super) const COMMUNITY: StreamId = StreamId::new("visual.ground-scatter.grass.community");
pub(super) const COMMUNITY_SPECIES: StreamId =
    StreamId::new("visual.ground-scatter.grass.community-species");
pub(super) const DENSITY: StreamId = StreamId::new("visual.ground-scatter.grass.density");
pub(super) const EXPOSURE: StreamId = StreamId::new("visual.ground-scatter.grass.exposure");
pub(super) const FERTILITY: StreamId = StreamId::new("visual.ground-scatter.grass.fertility");
pub(super) const HEIGHT: StreamId = StreamId::new("visual.ground-scatter.grass.height");
pub(super) const JITTER_X: StreamId = StreamId::new("visual.ground-scatter.grass.jitter-x");
pub(super) const JITTER_Z: StreamId = StreamId::new("visual.ground-scatter.grass.jitter-z");
pub(super) const LEAN: StreamId = StreamId::new("visual.ground-scatter.grass.lean");
pub(super) const MOISTURE: StreamId = StreamId::new("visual.ground-scatter.grass.moisture");
pub(super) const PANICLE: StreamId = StreamId::new("visual.ground-scatter.grass.panicle");
pub(super) const SPECIES: StreamId = StreamId::new("visual.ground-scatter.grass.species");
pub(super) const WIDTH: StreamId = StreamId::new("visual.ground-scatter.grass.width");

pub(super) fn blade_angle(seed: u64) -> f32 {
    BLADE_ANGLE.rng(seed, &[]).inclusive_unit_f32() * core::f32::consts::TAU
}
