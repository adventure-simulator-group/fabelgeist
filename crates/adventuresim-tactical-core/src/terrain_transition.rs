use adventuresim_world_schema::BASIS_POINTS_PER_WHOLE;
use bevy::prelude::*;
use fabelgeist_determinism::StreamId;
use serde::{Deserialize, Serialize};

use crate::scene_coordinates::{Scene, ScenePlanPoint};
use adventuresim_building_generator::spatial_geometry::{PlanDirection, PositiveLength};
mod admission;
pub use admission::{CollarWidthVariation, RuptureWander};

/// Checked scene geometry and engineering leaves for an implicit terrain collar.
pub struct TerrainCollarParameters {
    pub origin: ScenePlanPoint,
    pub tangent: PlanDirection<Scene>,
    pub half_length: PositiveLength,
    pub half_width: PositiveLength,
    pub width: PositiveLength,
    pub seed: fabelgeist_determinism::Seed,
    pub wander: RuptureWander,
    pub width_variation: CollarWidthVariation,
}
/// Bounded implicit footprint where a volumetric terrain patch takes ownership
/// from the heightfield and blends back through its irregular outer collar.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
#[component(immutable)]
#[reflect(opaque)]
#[serde(try_from = "CollarWire")]
pub struct TerrainTransitionCollar {
    origin: Vec2,
    tangent: Vec2,
    half_length_metres: f32,
    half_width_metres: f32,
    width_metres: f32,
    seed: fabelgeist_determinism::Seed,
    wander_metres: f32,
    width_variation_bps: u16,
}

#[derive(Deserialize)]
struct CollarWire {
    origin: Vec2,
    tangent: Vec2,
    half_length_metres: f32,
    half_width_metres: f32,
    width_metres: f32,
    seed: fabelgeist_determinism::Seed,
    wander_metres: f32,
    width_variation_bps: u16,
}
impl TryFrom<CollarWire> for TerrainTransitionCollar {
    type Error = crate::volumetric_terrain::TerrainRecipeError;
    fn try_from(wire: CollarWire) -> Result<Self, Self::Error> {
        // Admission checks the normalized encoded axis without renormalizing
        // its bits; interpolation and all existing samples retain their order.
        let collar = Self {
            origin: wire.origin,
            tangent: wire.tangent,
            half_length_metres: wire.half_length_metres,
            half_width_metres: wire.half_width_metres,
            width_metres: wire.width_metres,
            seed: wire.seed,
            wander_metres: wire.wander_metres,
            width_variation_bps: wire.width_variation_bps,
        };
        if adventuresim_building_generator::spatial_geometry::PlanDirection::<
            crate::scene_coordinates::Scene,
        >::from_normalized(wire.tangent)
        .is_err()
        {
            return Err(crate::volumetric_terrain::TerrainRecipeError::Tangent);
        }
        if !collar.valid() {
            return Err(crate::volumetric_terrain::TerrainRecipeError::Dimensions);
        }
        Ok(collar)
    }
}

impl TerrainTransitionCollar {
    pub fn irregular_ellipse(parameters: TerrainCollarParameters) -> Option<Self> {
        let admitted = Self {
            origin: parameters.origin.metres(),
            tangent: parameters.tangent.vector(),
            half_length_metres: parameters.half_length.metres(),
            half_width_metres: parameters.half_width.metres(),
            width_metres: parameters.width.metres(),
            seed: parameters.seed,
            wander_metres: parameters.wander.metres(),
            width_variation_bps: parameters.width_variation.basis_points(),
        };
        admitted.valid().then_some(admitted)
    }

    fn valid(self) -> bool {
        !(!self.origin.is_finite()
            || !self.tangent.is_finite()
            || !(0.98..=1.02).contains(&self.tangent.length())
            || !self.half_length_metres.is_finite()
            || !self.half_width_metres.is_finite()
            || !self.width_metres.is_finite()
            || !self.wander_metres.is_finite()
            || self.half_length_metres <= self.width_metres
            || self.half_width_metres <= self.width_metres
            || self.width_metres <= 0.0
            || self.wander_metres < 0.0
            || self.width_variation_bps > 5_000)
    }

    pub fn cuts_out(self, point: ScenePlanPoint) -> bool {
        self.radial_coordinate(point.metres()).0 < 1.0
    }

    pub fn contains(self, point: ScenePlanPoint) -> bool {
        self.radial_coordinate(point.metres()).0 <= 1.0
    }

    /// Native scene-metre raster field kernel; callers admit the raster bounds.
    pub(crate) fn blend_weight(self, point: Vec2) -> f32 {
        let (radial, minimum_extent) = self.radial_coordinate(point);
        let inner = 1.0 - self.width_metres / minimum_extent;
        smoothstep01((1.0 - radial) / (1.0 - inner))
    }

    /// Native scene-metre affine/field scratch, used within terrain extraction.
    pub(crate) fn local_coordinates(self, point: Vec2) -> Vec2 {
        let normal = Vec2::new(-self.tangent.y, self.tangent.x);
        let relative = point - self.origin;
        let along = relative.dot(self.tangent);
        let clamped = along.clamp(-self.half_length_metres, self.half_length_metres);
        let wander = smooth_value_noise(
            StreamId::new("terrain.transition.rupture-broad").seed(self.seed, &[]),
            clamped / 5.5,
        ) * 0.72
            + smooth_value_noise(
                StreamId::new("terrain.transition.rupture-fine").seed(self.seed, &[]),
                clamped / 1.8,
            ) * 0.28;
        Vec2::new(along, relative.dot(normal) - wander * self.wander_metres)
    }

    fn radial_coordinate(self, point: Vec2) -> (f32, f32) {
        let local = self.local_coordinates(point);
        let variation = f32::from(self.width_variation_bps) / BASIS_POINTS_PER_WHOLE as f32;
        let width_noise = smooth_value_noise(
            StreamId::new("terrain.transition.width").seed(self.seed, &[]),
            local.x / 4.2,
        ) * 0.5
            + 0.5;
        let local_half_width = self.half_width_metres * (1.0 - variation + width_noise * variation);
        let radial = Vec2::new(
            local.x / self.half_length_metres,
            local.y / local_half_width,
        )
        .length();
        (radial, self.half_length_metres.min(local_half_width))
    }
}

fn smooth_value_noise(seed: fabelgeist_determinism::Seed, coordinate: f32) -> f32 {
    let cell = coordinate.floor() as i64;
    let fraction = smoothstep01(coordinate - coordinate.floor());
    let sample = |offset: i64| {
        StreamId::new("terrain.transition.lattice")
            .rng(seed, &[cell.wrapping_add(offset) as u64])
            .inclusive_unit_f32()
            * 2.0
            - 1.0
    };
    sample(0).lerp(sample(1), fraction)
}

fn smoothstep01(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}
