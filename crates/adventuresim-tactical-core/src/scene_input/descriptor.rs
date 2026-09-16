//! Versioned scene document shared by production dispatch and capture tools.
use super::*;

pub const TACTICAL_SCENE_SCHEMA_VERSION: u16 = 21;
pub const TACTICAL_SCENE_GENERATION_VERSION: u16 = 48;
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
    pub gardens: Vec<crate::city_layout::CityGarden>,
    pub buildings: Vec<TacticalBuildingPlacement>,
    pub distant_buildings: Vec<DistantBuildingPlacement>,
    pub vista: VistaSample,
    pub weather: WeatherSnapshot,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_scene_catalog_uses_the_complete_current_schema() {
        let directory =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/tactical-scenes");
        let mut count = 0;
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            let value: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            if value.get("scene_key").is_none() {
                continue;
            }
            TacticalSceneInput::load(&path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            count += 1;
        }
        assert!(count > 0, "the shipped catalog must contain scene inputs");
    }
}
