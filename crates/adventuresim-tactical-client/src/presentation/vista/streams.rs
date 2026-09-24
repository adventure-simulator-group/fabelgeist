//! Named random purposes owned by this generator. Names are part of its replay contract.
use bevy::math::Vec2;
use fabelgeist_determinism::StreamId;

pub(super) const GRASS: StreamId = StreamId::new("visual.vista.grass");
pub(super) const GRASS_LOD: StreamId = StreamId::new("visual.vista.grass-lod");
pub(super) const JITTER_X: StreamId = StreamId::new("visual.vista.jitter-x");
pub(super) const JITTER_Z: StreamId = StreamId::new("visual.vista.jitter-z");
pub(super) const ROCK: StreamId = StreamId::new("visual.vista.rock");
pub(super) const ROCK_PRESENCE: StreamId = StreamId::new("visual.vista.rock-presence");
pub(super) const ROCK_SCALE: StreamId = StreamId::new("visual.vista.rock-scale");
pub(super) const ROCK_YAW: StreamId = StreamId::new("visual.vista.rock-yaw");
#[cfg(test)]
pub(super) const TEST_COUNT: StreamId = StreamId::new("visual.vista.test-count");
pub(super) const TREE: StreamId = StreamId::new("visual.vista.tree");
pub(super) const TREE_COUNT: StreamId = StreamId::new("visual.vista.tree-count");
pub(super) const TREE_COUNT_FRACTION: StreamId = StreamId::new("visual.vista.tree-count-fraction");
pub(super) const TREE_JITTER_Z: StreamId = StreamId::new("visual.vista.tree-jitter-z");
pub(super) const TREE_SCALE: StreamId = StreamId::new("visual.vista.tree-scale");

pub(super) fn rock_cell(root: u64, x: i32, z: i32, spacing: f32) -> (u64, Vec2) {
    let seed = ROCK
        .seed(root, &[x as u32 as u64, z as u32 as u64])
        .to_u64();
    let jitter = Vec2::new(
        JITTER_X.rng(seed, &[]).inclusive_unit_f32() - 0.5,
        JITTER_Z.rng(seed, &[]).inclusive_unit_f32() - 0.5,
    ) * spacing
        * 0.72;
    (seed, jitter)
}

pub(super) fn tree_count_seed(root: u64, x: usize, z: usize) -> u64 {
    TREE_COUNT.seed(root, &[x as u64, z as u64]).to_u64()
}

pub(super) fn tree_seed(root: u64, x: usize, z: usize, candidate: usize) -> u64 {
    TREE.seed(root, &[x as u64, z as u64, candidate as u64])
        .to_u64()
}

pub(super) fn tree_jitter(seed: u64) -> Vec2 {
    Vec2::new(
        JITTER_X.rng(seed, &[]).inclusive_unit_f32(),
        TREE_JITTER_Z.rng(seed, &[]).inclusive_unit_f32(),
    )
}
