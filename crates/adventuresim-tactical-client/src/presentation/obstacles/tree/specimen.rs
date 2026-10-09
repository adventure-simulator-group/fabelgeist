//! Deterministic selection of existing oak variants for rendering and inspection.

use bevy::prelude::Vec3;
use fabelgeist_determinism::Seed;
use fabelgeist_determinism::StreamId;

use super::{OAK_GNARLING_SHOWCASE, source::oak_gnarling_for_site};
use crate::presentation::{SceneEnvironment, obstacle_seed};

pub(super) fn oak_variant_for_site(position: Vec3) -> (usize, Seed) {
    let index = StreamId::new("visual.obstacles.tree.specimen.variant-selection")
        .rng(obstacle_seed(position), &[])
        .index(OAK_GNARLING_SHOWCASE.len());
    (index, oak_variant_seed(index))
}

pub(crate) fn oak_root_exposure_for_site(position: Vec3, environment: &SceneEnvironment) -> f32 {
    let (index, seed) = oak_variant_for_site(position);
    oak_gnarling_for_site(OAK_GNARLING_SHOWCASE[index], environment, seed).root_exposure
}

/// Index is an authored OAK_GNARLING_SHOWCASE slot, independent of traversal.
pub(in crate::presentation) fn oak_variant_seed(index: usize) -> Seed {
    StreamId::new("visual.tree.oak-variant").seed(0.into(), &[index as u64])
}
