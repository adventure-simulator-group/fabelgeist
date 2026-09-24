use super::*;
use bevy::prelude::Component;

/// Compact immutable presentation handoff. Large vista grids remain outside
/// ordinary ECS replication; this component carries only weather, provenance,
/// and broad material coverage needed by every client.
#[derive(Clone, Debug, Eq, PartialEq, Component, Serialize, Deserialize)]
#[component(immutable)]
#[serde(deny_unknown_fields)]
pub struct SceneEnvironment {
    pub scene_digest: String,
    pub generation_version: u16,
    pub latitude_microdegrees: i32,
    pub longitude_microdegrees: i32,
    pub absolute_minute: u64,
    pub lunar_phase_minute: u64,
    pub absolute_elevation_metres: i16,
    pub weather: WeatherSnapshot,
    pub canopy_bps: u16,
    pub wetland_bps: u16,
    pub cultivation_bps: u16,
    pub water_bps: u16,
    pub hilly_bps: u16,
}

/// Explicit environment profiles for deterministic tactical-only fixtures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneEnvironmentFixture {
    TemperateHills,
}

impl SceneEnvironmentFixture {
    pub fn snapshot(self, scene_digest: impl Into<String>) -> SceneEnvironment {
        match self {
            Self::TemperateHills => SceneEnvironment {
                scene_digest: scene_digest.into(),
                generation_version: TACTICAL_SCENE_GENERATION_VERSION,
                latitude_microdegrees: 53_500_000,
                longitude_microdegrees: 10_000_000,
                absolute_minute: MINUTES_PER_DAY / 2,
                lunar_phase_minute: MINUTES_PER_DAY / 2,
                absolute_elevation_metres: 20,
                weather: WeatherSnapshot {
                    rules_version: WEATHER_RULES_VERSION,
                    interval_start_minute: 0,
                    cell_latitude: 0,
                    cell_longitude: 0,
                    temperature_deci_c: 100,
                    wind_speed_bps: 1_500,
                    precipitation: Precipitation::Clear,
                    intensity_bps: 0,
                    ground_moisture_bps: 0,
                    snow_cover_bps: 0,
                    atmosphere: Default::default(),
                },
                canopy_bps: 1_200,
                wetland_bps: 0,
                cultivation_bps: 0,
                water_bps: 0,
                hilly_bps: 7_000,
            },
        }
    }
}
