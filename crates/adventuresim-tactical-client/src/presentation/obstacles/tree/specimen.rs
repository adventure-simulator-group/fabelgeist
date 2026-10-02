//! Deterministic selection of existing oak variants for rendering and inspection.

use bevy::prelude::Vec3;
use fabelgeist_determinism::splitmix64;

use super::{OAK_GNARLING_SHOWCASE, source::oak_gnarling_for_site};
use crate::presentation::{SceneEnvironment, obstacle_seed};

pub(super) fn oak_variant_for_site(position: Vec3) -> (usize, u64) {
    let index = obstacle_seed(position) as usize % OAK_GNARLING_SHOWCASE.len();
    (index, splitmix64(0x6f61_6b00 ^ index as u64))
}

pub(crate) fn oak_root_exposure_for_site(position: Vec3, environment: &SceneEnvironment) -> f32 {
    let (index, seed) = oak_variant_for_site(position);
    oak_gnarling_for_site(OAK_GNARLING_SHOWCASE[index], environment, seed).root_exposure
}
