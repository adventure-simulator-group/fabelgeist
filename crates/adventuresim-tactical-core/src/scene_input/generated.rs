use super::{GeneratedBuilding, GeneratedObstacle, furniture::FurnitureLayout};
use crate::{
    scene::{SceneGround, SceneTerrain},
    volumetric_terrain::SceneTerrainPatch,
};

#[derive(Debug)]
pub struct GeneratedTacticalScene {
    pub digest: String,
    pub terrain: SceneTerrain,
    pub ground: SceneGround,
    pub obstacles: Vec<GeneratedObstacle>,
    pub terrain_patch: Option<SceneTerrainPatch>,
    pub buildings: Vec<GeneratedBuilding>,
    pub furniture: FurnitureLayout,
    pub repairs: SceneRepairReport,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SceneRepairReport {
    pub upsampled_height_samples: u32,
    pub microrelief_adjusted_samples: u32,
    pub adjusted_height_samples: u32,
    pub repaired_water_samples: u32,
    pub removed_corridor_obstacles: u32,
    pub levelled_building_samples: u32,
    pub removed_building_obstacles: u32,
}

impl SceneRepairReport {
    pub const fn was_repaired(self) -> bool {
        self.adjusted_height_samples != 0
            || self.repaired_water_samples != 0
            || self.removed_corridor_obstacles != 0
            || self.levelled_building_samples != 0
            || self.removed_building_obstacles != 0
    }
}
