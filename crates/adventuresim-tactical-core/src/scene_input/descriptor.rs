//! Versioned scene document shared by production dispatch and capture tools.
use super::*;

pub const TACTICAL_SCENE_SCHEMA_VERSION: u16 = 20;
pub const TACTICAL_SCENE_GENERATION_VERSION: u16 = 47;
pub const MAX_SCENE_INPUT_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalSceneInput {
    pub schema_version: u16,
    pub generation_version: u16,
    pub seed: u64,
    pub scene_key: String,
    pub source: SceneSource,
    pub latitude_microdegrees: i32,
    pub longitude_microdegrees: i32,
    pub absolute_minute: u64,
    pub lunar_phase_minute: u64,
    pub absolute_elevation_metres: i16,
    pub playable: TerrainSampleGrid,
    pub landform: Option<TerrainLandformRecipe>,
    pub streets: Vec<CityStreetPatch>,
    pub yards: Vec<CityYardPatch>,
    pub parishes: Vec<crate::city_layout::CityParish>,
    pub compounds: Vec<crate::city_layout::CityCompound>,
    pub buildings: Vec<TacticalBuildingPlacement>,
    pub distant_buildings: Vec<DistantBuildingPlacement>,
    pub vista: VistaSample,
    pub weather: WeatherSnapshot,
}
