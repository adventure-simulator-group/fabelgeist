use adventuresim_building_generator::spatial_geometry::GeometryResult;
mod window;
use adventuresim_building_generator::{
    BuildingArchetype, BuildingCollision, BuildingPlan, BuildingProgram,
};
#[cfg(test)]
use adventuresim_building_generator::{compile_building_collision, generate};
use bevy::{math::Vec2, prelude::Component};
use serde::{Deserialize, Serialize};
mod identity;
pub use identity::SceneBuildingId;
pub use window::{SceneWindow, SceneWindowError};

use super::{GeneratedObstacle, SceneInputError, SceneInputResult, invalid};
use crate::city_layout::MAX_CITY_BUILDING_INSTANCES;
mod collision;
mod exterior;
mod placement;
pub use collision::compile_tactical_building_collider;
pub use exterior::DistantBuildingVariant;

pub(crate) const MAX_TACTICAL_BUILDINGS: usize = 64;
const LEVEL_MARGIN_METRES: f32 = 1.5;
const TERRACE_APRON_METRES: f32 = 4.0;
const PARTY_WALL_PROJECTION_ALLOWANCE_METRES: f32 = 0.4;

/// Rotation of one orthogonal building grid in the settlement plane.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, bevy::prelude::Reflect)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct BuildingOrientation(f32);

impl<'de> Deserialize<'de> for BuildingOrientation {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let orientation = Self(f32::deserialize(d)?);
        if orientation.is_valid() {
            Ok(orientation)
        } else {
            Err(serde::de::Error::custom(
                "building orientation must be finite and in [-pi, pi)",
            ))
        }
    }
}

impl BuildingOrientation {
    pub const IDENTITY: Self = Self(0.0);

    pub fn from_radians(yaw_radians: f32) -> Option<Self> {
        yaw_radians.is_finite().then(|| {
            Self(
                (yaw_radians + core::f32::consts::PI).rem_euclid(core::f32::consts::TAU)
                    - core::f32::consts::PI,
            )
        })
    }

    pub fn from_frontage_tangent(tangent: Vec2) -> Option<Self> {
        (tangent.is_finite() && tangent.length_squared() > f32::EPSILON).then(|| {
            let tangent = tangent.normalize();
            let yaw = (-tangent.y).atan2(tangent.x);
            // Signed zero can represent the positive endpoint of atan2's
            // range. Canonicalize only that previously inadmissible endpoint.
            Self(if yaw == core::f32::consts::PI {
                -core::f32::consts::PI
            } else {
                yaw
            })
        })
    }

    pub fn yaw_radians(self) -> f32 {
        self.0
    }

    pub fn is_valid(self) -> bool {
        self.0.is_finite() && self.0 >= -core::f32::consts::PI && self.0 < core::f32::consts::PI
    }

    pub fn local_to_world(self, vector: Vec2) -> Vec2 {
        let (sine, cosine) = self.0.sin_cos();
        Vec2::new(
            cosine * vector.x + sine * vector.y,
            -sine * vector.x + cosine * vector.y,
        )
    }

    pub fn world_to_local(self, vector: Vec2) -> Vec2 {
        let (sine, cosine) = self.0.sin_cos();
        Vec2::new(
            cosine * vector.x - sine * vector.y,
            sine * vector.x + cosine * vector.y,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalBuildingPlacement {
    pub id: u64,
    pub program: BuildingProgram,
    pub centre_metres: Vec2,
    /// Scene elevation of architectural Y=0, retained through detail changes.
    /// Buried slabs and footings extend below this datum.
    pub base_elevation_metres: f32,
    pub orientation: BuildingOrientation,
}

/// Compact physical program outside the active tactical area. All visual detail
/// levels compile this occupied recipe at its original dimensions. Meshes remain
/// shared by equal programs; distant instances receive no tactical tick state.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DistantBuildingPlacement {
    pub id: u64,
    pub prosperity: adventuresim_world_schema::ProsperityTier,
    pub archetype: BuildingArchetype,
    pub usage: Option<adventuresim_world_schema::settlement_buildings::BuildingUse>,
    pub service_size: Option<adventuresim_building_generator::ServiceBuildingSize>,
    pub seed: u64,
    pub centre_metres: Vec2,
    pub base_elevation_metres: f32,
    pub orientation: BuildingOrientation,
}

impl DistantBuildingPlacement {
    /// The occupied recipe is retained for promotion into a playable venue.
    pub fn occupied_program(self) -> BuildingProgram {
        let mut program = match self.usage {
            Some(usage) => BuildingProgram::settlement(self.archetype, Some(usage), self.seed),
            None => BuildingProgram::fixture(self.archetype, self.seed),
        };
        if let Some(size) = self.service_size {
            program = program.with_service_size(size);
        }
        program
    }
}

impl From<DistantBuildingPlacement> for TacticalBuildingPlacement {
    fn from(placement: DistantBuildingPlacement) -> Self {
        Self {
            id: placement.id,
            program: placement.occupied_program(),
            centre_metres: placement.centre_metres,
            base_elevation_metres: placement.base_elevation_metres,
            orientation: placement.orientation,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Component, Serialize, Deserialize)]
#[component(immutable)]
#[serde(deny_unknown_fields)]
pub struct SceneBuilding {
    pub id: SceneBuildingId,
    pub program: BuildingProgram,
    pub orientation: BuildingOrientation,
}

/// Compact identity and dimensions for one server-authoritative operable leaf.
/// Positions and normalized directions are in the scene frame after explicit
/// architectural-floor, collision-centre, or gate-datum conversion.
#[derive(Clone, Copy, Debug, PartialEq, Component, Serialize)]
#[component(immutable)]
pub struct SceneDoor {
    pub building_id: SceneBuildingId,
    pub opening_id: adventuresim_building_generator::OpeningAssemblyId,
    pub size_metres: adventuresim_building_generator::spatial_geometry::LeafDimensions,
    pub doorway_centre_metres: adventuresim_building_generator::spatial_geometry::Position<
        crate::scene_coordinates::Scene,
    >,
    pub tangent: adventuresim_building_generator::spatial_geometry::SpatialDirection<
        crate::scene_coordinates::Scene,
    >,
    pub outward: adventuresim_building_generator::spatial_geometry::SpatialDirection<
        crate::scene_coordinates::Scene,
    >,
}
impl<'de> Deserialize<'de> for SceneDoor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use adventuresim_building_generator::spatial_geometry::{
            LeafDimensions, Position, SpatialDirection,
        };
        #[derive(Deserialize)]
        struct SerializedDoor {
            building_id: u64,
            opening_id: u64,
            size_metres: bevy::math::Vec3,
            doorway_centre_metres: bevy::math::Vec3,
            tangent: bevy::math::Vec3,
            outward: bevy::math::Vec3,
        }
        let value = SerializedDoor::deserialize(deserializer)?;
        let construct = || {
            Ok(Self {
                building_id: value.building_id.into(),
                opening_id: adventuresim_building_generator::OpeningAssemblyId(value.opening_id),
                size_metres: LeafDimensions::from_metres(value.size_metres)?,
                doorway_centre_metres: Position::from_metres(value.doorway_centre_metres)?,
                tangent: SpatialDirection::from_normalized(value.tangent)?,
                outward: SpatialDirection::from_normalized(value.outward)?,
            })
        };
        construct().map_err(|cause| {
            serde::de::Error::custom(SceneDoorError {
                building_id: crate::scene_input::SceneBuildingId::from(value.building_id),
                opening_id: adventuresim_building_generator::OpeningAssemblyId(value.opening_id),
                cause,
            })
        })
    }
}
#[derive(Debug, thiserror::Error)]
#[error("building {building_id}, door {opening_id}: {cause}")]
pub struct SceneDoorError {
    pub building_id: SceneBuildingId,
    pub opening_id: adventuresim_building_generator::OpeningAssemblyId,
    #[source]
    pub cause: adventuresim_building_generator::spatial_geometry::GeometryError,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedBuilding {
    pub placement: TacticalBuildingPlacement,
    pub plan: BuildingPlan,
    pub collision: BuildingCollision,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BuildingPad {
    pub centre: Vec2,
    pub half_extents: Vec2,
    pub orientation: BuildingOrientation,
    pub elevation_metres: f32,
}

impl BuildingPad {
    fn local_offset(self, point: Vec2) -> Vec2 {
        let offset = point - self.centre;
        self.orientation.world_to_local(offset)
    }

    pub(crate) fn contains_level_ground(self, point: Vec2) -> bool {
        let offset = self.local_offset(point).abs();
        offset
            .cmple(self.half_extents + Vec2::splat(LEVEL_MARGIN_METRES))
            .all()
    }

    pub(super) fn contains_apron(self, point: Vec2) -> bool {
        let offset = self.local_offset(point).abs();
        offset
            .cmple(self.half_extents + Vec2::splat(LEVEL_MARGIN_METRES + TERRACE_APRON_METRES))
            .all()
    }
}

pub(super) fn validate_building_placements(
    placements: &[TacticalBuildingPlacement],
) -> SceneInputResult<()> {
    if placements.len() > MAX_TACTICAL_BUILDINGS {
        return invalid("scene has too many tactical buildings");
    }
    let mut ids = std::collections::BTreeSet::new();
    for placement in placements {
        if placement.id == 0 || !ids.insert(placement.id) {
            return invalid("building identity is zero or duplicated");
        }
        if !placement.centre_metres.is_finite()
            || !placement.base_elevation_metres.is_finite()
            || !placement.orientation.is_valid()
        {
            return invalid("building placement is invalid");
        }
    }
    Ok(())
}

pub(super) fn validate_distant_building_placements(
    placements: &[DistantBuildingPlacement],
) -> SceneInputResult<()> {
    if placements.len() > MAX_CITY_BUILDING_INSTANCES {
        return invalid("scene has too many distant buildings");
    }
    let mut ids = std::collections::BTreeSet::new();
    for placement in placements {
        if placement.id == 0 || !ids.insert(placement.id) {
            return invalid("distant building identity is zero or duplicated");
        }
        if !placement.centre_metres.is_finite()
            || !placement.base_elevation_metres.is_finite()
            || !placement.orientation.is_valid()
        {
            return invalid("distant building placement is invalid");
        }
    }
    Ok(())
}

pub(super) fn prepare_buildings(
    placements: &[TacticalBuildingPlacement],
    recipes: &mut super::GeneratedBuildingRecipes,
) -> SceneInputResult<Vec<GeneratedBuilding>> {
    placements
        .iter()
        .cloned()
        .map(|placement| {
            let recipe = recipes
                .take(&placement.program)
                .map(Ok)
                .unwrap_or_else(|| {
                    super::GeneratedBuildingRecipe::generate(placement.program.clone())
                })
                .map_err(|error| {
                    SceneInputError::Validation(format!(
                        "building {} program is invalid: {error}",
                        placement.id
                    ))
                })?;
            Ok(GeneratedBuilding {
                placement,
                plan: recipe.plan,
                collision: recipe.collision,
            })
        })
        .collect()
}

pub(super) fn validate_building_pads(buildings: &[GeneratedBuilding]) -> SceneInputResult<()> {
    let footprints = buildings
        .iter()
        .map(|building| {
            let half_extents = (building.collision.bounds.plan_half_extents()?.metres()
                - Vec2::splat(PARTY_WALL_PROJECTION_ALLOWANCE_METRES))
            .max(Vec2::splat(0.1));
            Ok((
                building.placement.id,
                building.placement.centre_metres,
                half_extents,
                building.placement.orientation,
            ))
        })
        .collect::<GeometryResult<Vec<_>>>()?;
    for (index, &(id, centre, half_extents, orientation)) in footprints.iter().enumerate() {
        for &(other_id, other_centre, other_half_extents, other_orientation) in
            &footprints[index + 1..]
        {
            if oriented_rectangles_overlap(
                centre,
                half_extents,
                orientation,
                other_centre,
                other_half_extents,
                other_orientation,
            ) {
                return invalid(format!(
                    "building {id} footprint overlaps building {other_id}"
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn oriented_rectangles_overlap(
    first_centre: Vec2,
    first_half_extents: Vec2,
    first_orientation: BuildingOrientation,
    second_centre: Vec2,
    second_half_extents: Vec2,
    second_orientation: BuildingOrientation,
) -> bool {
    let first_axes = [
        first_orientation.local_to_world(Vec2::X),
        first_orientation.local_to_world(Vec2::Y),
    ];
    let second_axes = [
        second_orientation.local_to_world(Vec2::X),
        second_orientation.local_to_world(Vec2::Y),
    ];
    let centre_delta = second_centre - first_centre;
    first_axes.into_iter().chain(second_axes).all(|axis| {
        let first_radius = first_half_extents.x * axis.dot(first_axes[0]).abs()
            + first_half_extents.y * axis.dot(first_axes[1]).abs();
        let second_radius = second_half_extents.x * axis.dot(second_axes[0]).abs()
            + second_half_extents.y * axis.dot(second_axes[1]).abs();
        centre_delta.dot(axis).abs() < first_radius + second_radius
    })
}

pub(super) fn obstacle_intersects_building(
    obstacle: GeneratedObstacle,
    obstacle_spacing: f32,
    terrain_extent: Vec2,
    pads: &[BuildingPad],
) -> bool {
    let (x, z) = match obstacle {
        GeneratedObstacle::Tree { x, z } | GeneratedObstacle::Rock { x, z, .. } => (x, z),
    };
    let point = Vec2::new(f32::from(x), f32::from(z)) * obstacle_spacing - terrain_extent * 0.5;
    pads.iter().any(|pad| pad.contains_apron(point))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_orientation_is_finite_canonical_and_rejects_a_zero_frontage() {
        let wrapped = BuildingOrientation::from_radians(core::f32::consts::TAU + 0.25).unwrap();
        assert!((wrapped.yaw_radians() - 0.25).abs() < 0.000_001);
        assert!(BuildingOrientation::from_radians(f32::NAN).is_none());
        assert!(BuildingOrientation::from_frontage_tangent(Vec2::ZERO).is_none());
    }

    #[test]
    fn oriented_pad_overlap_uses_both_local_frames() {
        let identity = BuildingOrientation::IDENTITY;
        let diagonal = BuildingOrientation::from_radians(core::f32::consts::FRAC_PI_4).unwrap();
        let half_extents = Vec2::new(4.0, 1.0);

        assert!(oriented_rectangles_overlap(
            Vec2::ZERO,
            half_extents,
            identity,
            Vec2::new(2.0, 0.0),
            half_extents,
            diagonal,
        ));
        assert!(!oriented_rectangles_overlap(
            Vec2::ZERO,
            half_extents,
            identity,
            Vec2::new(0.0, 5.0),
            half_extents,
            diagonal,
        ));
    }
}

#[cfg(test)]
mod occupied_recipe_tests {
    use super::*;
    use adventuresim_world_schema::settlement_buildings::BuildingUse;

    #[test]
    fn distant_buildings_reconstruct_the_same_occupied_recipe() {
        let placement = DistantBuildingPlacement {
            prosperity: adventuresim_world_schema::ProsperityTier::Comfortable,
            id: 1,
            archetype: BuildingArchetype::HallHouse,
            usage: Some(BuildingUse::Stable),
            service_size: Some(adventuresim_building_generator::ServiceBuildingSize::Large),
            seed: 42,
            centre_metres: Vec2::ZERO,
            base_elevation_metres: 0.0,
            orientation: BuildingOrientation::IDENTITY,
        };
        assert_eq!(
            placement.occupied_program(),
            BuildingProgram::settlement(
                BuildingArchetype::HallHouse,
                Some(BuildingUse::Stable),
                42
            )
            .with_service_size(adventuresim_building_generator::ServiceBuildingSize::Large)
        );
        assert_ne!(
            placement.occupied_program(),
            BuildingProgram::fixture(BuildingArchetype::HallHouse, 42)
        );
    }
}

#[cfg(test)]
mod service_recipe_tests;
