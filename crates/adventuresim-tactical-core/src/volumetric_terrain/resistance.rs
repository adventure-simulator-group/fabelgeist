//! Immutable edge resistance for the bounded fault-scarp erosion stencil.
//!
//! Grid coordinates use the solver's native horizontal metre frame. Resistance
//! depends on the recipe and edge midpoint, independently of evolving offsets.
//! Sampling it once keeps every erosion step's floating-point update unchanged.
use super::{TerrainLandformRecipe, smooth_value_noise, smoothstep01};
use crate::terrain_transition::TerrainTransitionCollar;
use bevy::math::Vec2;
use fabelgeist_determinism::{Seed, StreamId};
use rayon::prelude::*;

const SCARP_FREE_FACE_HALF_WIDTH_METRES: f32 = 0.55;
const SCARP_GULLY_COUNT: usize = 4;

pub(super) struct ScarpResistanceGrid {
    cells: Vec<CellResistance>,
}

pub(super) struct CellResistance {
    pub x_neighbor: Option<ScarpResistance>,
    pub z_neighbor: Option<ScarpResistance>,
}

/// A dimensionless material resistance admitted by the existing clamped field.
#[derive(Clone, Copy)]
pub(super) struct ScarpResistance(f32);

impl ScarpResistanceGrid {
    /// Native numerical boundary: square-grid indices, spacing and minimum use
    /// the same bounded lattice and metre coordinates as the erosion solver.
    pub(super) fn new(
        masks: &[f32],
        recipe: TerrainLandformRecipe,
        collar: TerrainTransitionCollar,
        side: usize,
        spacing: f32,
        minimum: Vec2,
    ) -> Self {
        let half_length = f32::from(recipe.half_length_cm()) / 100.0;
        let cells = (0..masks.len())
            .into_par_iter()
            .map(|source| {
                let x = source % side;
                let z = source / side;
                let source_point = minimum + Vec2::new(x as f32, z as f32) * spacing;
                let sample = |nx: usize, nz: usize| {
                    if nx >= side || nz >= side || masks[source] <= 0.0 {
                        return None;
                    }
                    if masks[nz * side + nx] <= 0.0 {
                        return None;
                    }
                    let target_point = minimum + Vec2::new(nx as f32, nz as f32) * spacing;
                    Some(ScarpResistance::sample(
                        recipe.seed(),
                        collar,
                        (source_point + target_point) * 0.5,
                        half_length,
                    ))
                };
                CellResistance {
                    x_neighbor: sample(x + 1, z),
                    z_neighbor: sample(x, z + 1),
                }
            })
            .collect();
        Self { cells }
    }

    pub(super) fn cell(&self, source: usize) -> &CellResistance {
        &self.cells[source]
    }
}

impl ScarpResistance {
    /// Native stencil handoff: preserve the field's coefficient and arithmetic.
    pub(super) fn coefficient(self) -> f32 {
        self.0
    }

    fn sample(seed: Seed, collar: TerrainTransitionCollar, point: Vec2, half_length: f32) -> Self {
        let local = collar.local_coordinates(point);
        // The uplifted block retains a narrower bedrock face than the broader
        // colluvial toe on the hanging-wall side.
        let free_face_distance = (local.y + 0.35).abs();
        let free_face = smoothstep01(
            (SCARP_FREE_FACE_HALF_WIDTH_METRES - free_face_distance)
                / SCARP_FREE_FACE_HALF_WIDTH_METRES,
        );
        let coherent_material = (smooth_value_noise(
            StreamId::new("terrain.volume.resistance").seed(seed, &[]),
            local.x / 6.5 + local.y / 8.0,
        ) * 0.5
            + 0.5)
            * 0.18;
        let drainage = scarp_gully_strength(seed, local.x, half_length)
            * smoothstep01((SCARP_FREE_FACE_HALF_WIDTH_METRES * 2.4 - local.y.abs()) / 3.0);
        Self((0.08 + free_face * 0.78 + coherent_material - drainage * 0.58).clamp(0.0, 1.0))
    }
}

fn scarp_gully_strength(seed: Seed, along: f32, half_length: f32) -> f32 {
    (0..SCARP_GULLY_COUNT)
        .map(|index| {
            let mut random = StreamId::new("terrain.gully").rng(seed, &[index as u64]);
            let interval = (index as f32 + 0.5) / SCARP_GULLY_COUNT as f32;
            let jitter = (random.inclusive_unit_f32() - 0.5) * 0.16;
            let centre = (interval + jitter) * half_length * 1.8 - half_length * 0.9;
            let width = 1.4 + random.inclusive_unit_f32() * 1.3;
            smoothstep01((width - (along - centre).abs()) / width)
        })
        .fold(0.0, f32::max)
}
