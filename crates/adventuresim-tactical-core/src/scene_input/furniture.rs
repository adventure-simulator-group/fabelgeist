//! Deterministic furniture placement shared by server and presentation.
use adventuresim_building_generator::furniture::FurnitureKey;
use bevy::{
    math::{Vec2, Vec3},
    prelude::{Component, Reflect, ReflectComponent, ReflectDeserialize, ReflectSerialize},
};
use serde::{Deserialize, Serialize};

use super::{BuildingOrientation, GeneratedBuilding, GeneratedObstacle, TacticalSceneInput};
use crate::scene::{SceneGround, SceneTerrain};

mod candidates;
mod collision;
mod ground;
mod interior;
mod occupancy;
mod placement;
mod reservations;
mod sites;
pub use collision::furniture_collider;
pub use interior::InteriorBuildingLayout;
#[cfg(test)]
mod tests;

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Reflect,
)]
pub struct FurnitureInstanceId(pub u64);

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Reflect,
)]
pub struct FurnitureGroupId(pub u64);

/// Ownership distinguishes street activity groups from rooms inside buildings.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Reflect)]
pub enum FurnitureLocation {
    Outdoor {
        group_id: FurnitureGroupId,
    },
    Interior {
        building_id: u64,
        room_id: u16,
        storey: u16,
    },
}

/// Compact recipe identity; transform is replicated through the usual ECS path.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Component, Serialize, Deserialize, Reflect)]
#[component(immutable)]
#[reflect(Component, Serialize, Deserialize)]
pub struct SceneFurniture {
    pub id: FurnitureInstanceId,
    pub key: FurnitureKey,
    pub location: FurnitureLocation,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
pub struct GeneratedFurniture {
    pub scene: SceneFurniture,
    pub position_metres: Vec3,
    pub orientation: BuildingOrientation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Reflect)]
pub enum FurnitureGroupKind {
    Vendor,
    Receiving,
    HorseStop,
    Domestic,
    Workshop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Reflect)]
pub enum FurnitureAnchor {
    Market { patch_index: u32 },
    Building { id: u64 },
}

/// Physical kit bounds plus the access/workspace that must remain unobstructed.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
pub struct FurnitureFootprint {
    pub centre_metres: Vec2,
    pub half_extents_metres: Vec2,
    pub orientation: BuildingOrientation,
}

impl FurnitureFootprint {
    pub fn corners(self) -> [Vec2; 4] {
        [
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::ONE,
            Vec2::new(-1.0, 1.0),
        ]
        .map(|corner| {
            self.centre_metres
                + self
                    .orientation
                    .local_to_world(corner * self.half_extents_metres)
        })
    }

    pub fn contains(self, point: Vec2) -> bool {
        self.orientation
            .world_to_local(point - self.centre_metres)
            .abs()
            .cmple(self.half_extents_metres)
            .all()
    }

    pub fn intersects(self, other: Self) -> bool {
        let axes = [
            self.orientation.local_to_world(Vec2::X),
            self.orientation.local_to_world(Vec2::Y),
            other.orientation.local_to_world(Vec2::X),
            other.orientation.local_to_world(Vec2::Y),
        ];
        axes.into_iter().all(|axis| {
            let radius = |footprint: Self| {
                footprint
                    .orientation
                    .local_to_world(Vec2::X)
                    .dot(axis)
                    .abs()
                    * footprint.half_extents_metres.x
                    + footprint
                        .orientation
                        .local_to_world(Vec2::Y)
                        .dot(axis)
                        .abs()
                        * footprint.half_extents_metres.y
            };
            (self.centre_metres - other.centre_metres).dot(axis).abs()
                < radius(self) + radius(other)
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Reflect)]
pub struct FurnitureGroup {
    pub id: FurnitureGroupId,
    pub kind: FurnitureGroupKind,
    pub anchor: FurnitureAnchor,
    pub footprint: FurnitureFootprint,
}

/// Preserves accepted activity metadata in debug world dumps without repeating
/// it on each furniture instance or transmitting separate group entities.
#[derive(Clone, Debug, Component, Reflect, Serialize, Deserialize)]
#[component(immutable)]
#[reflect(Component, Serialize, Deserialize)]
pub struct SceneFurnitureGroup(pub FurnitureGroup);

/// Immutable vista furniture payload retained in debug world snapshots.
#[derive(Clone, Debug, Component, Reflect, Serialize, Deserialize)]
#[component(immutable)]
#[reflect(Component, Serialize, Deserialize)]
pub struct SceneVistaFurniture(pub Vec<GeneratedFurniture>);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FurnitureLayout {
    pub instances: Vec<GeneratedFurniture>,
    /// Accepted scenery beyond tactical world bounds; never receives physics.
    pub distant_instances: Vec<GeneratedFurniture>,
    pub groups: Vec<FurnitureGroup>,
    pub reserved_routes: Vec<FurnitureFootprint>,
    pub interiors: Vec<InteriorBuildingLayout>,
}

impl FurnitureLayout {
    pub(super) fn furnish_interiors(
        &mut self,
        buildings: &[GeneratedBuilding],
    ) -> Result<(), super::SceneInputError> {
        interior::append(self, buildings)
    }
}

pub(super) fn generate(
    input: &TacticalSceneInput,
    buildings: &[GeneratedBuilding],
    terrain: &SceneTerrain,
    ground: &SceneGround,
    obstacles: &[GeneratedObstacle],
) -> Result<FurnitureLayout, super::SceneInputError> {
    let sites = sites::collect(input, buildings)?;
    Ok(placement::generate(
        input, &sites, terrain, ground, obstacles,
    ))
}
