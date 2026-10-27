//! Deterministic furniture placement shared by server and presentation.
use adventuresim_building_generator::furniture::FurnitureKey;
use bevy::{
    math::{Vec2, Vec3},
    prelude::{Component, Reflect, ReflectComponent, ReflectDeserialize, ReflectSerialize},
};
use serde::{Deserialize, Serialize};

use super::{BuildingOrientation, GeneratedBuilding, GeneratedObstacle, TacticalSceneInput};
use crate::scene::{SceneGround, SceneTerrain};

pub use adventuresim_building_generator::{RoomIndex, StoreyIndex};
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
pub use sites::FurnitureSiteRecipe;
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
        building_id: crate::scene_input::SceneBuildingId,
        room_id: adventuresim_building_generator::RoomIndex,
        storey: adventuresim_building_generator::StoreyIndex,
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
    pub position_metres: adventuresim_building_generator::spatial_geometry::Position<
        crate::scene_coordinates::Scene,
    >,
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
    Building { id: super::SceneBuildingId },
}

mod footprint;
pub use footprint::FurnitureFootprint;

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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FurnitureLayout {
    pub instances: Vec<GeneratedFurniture>,
    /// Accepted scenery beyond tactical world bounds; never receives physics.
    pub distant_instances: Vec<GeneratedFurniture>,
    pub groups: Vec<FurnitureGroup>,
    pub reserved_routes: Vec<FurnitureFootprint>,
    pub interiors: Vec<InteriorBuildingLayout>,
}

impl FurnitureLayout {
    /// Install a worker-prepared layout with the destination's physical placement.
    pub fn install_interior(
        &mut self,
        building: &GeneratedBuilding,
        layout: adventuresim_building_generator::interior::InteriorLayout,
    ) -> Result<(), super::SceneInputError> {
        interior::install(self, building, layout)
    }
    pub fn furnish_interiors(
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
    recipes: &mut super::GeneratedBuildingRecipes,
) -> Result<FurnitureLayout, super::SceneInputError> {
    let sites = sites::collect(input, buildings, recipes)?;
    placement::generate(input, &sites, terrain, ground, obstacles)
}
