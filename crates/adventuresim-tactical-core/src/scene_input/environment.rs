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
    pub latitude_microdegrees: adventuresim_world_schema::coordinates::LatitudeMicrodegrees,
    pub longitude_microdegrees: adventuresim_world_schema::coordinates::LongitudeMicrodegrees,
    pub absolute_minute: StrategicMinute,
    pub lunar_phase_minute: StrategicMinute,
    #[serde(with = "super::absolute_elevation_wire")]
    pub absolute_elevation_metres: adventuresim_world_schema::ElevationMeters,
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
                latitude_microdegrees: FIXTURE_LATITUDE,
                longitude_microdegrees: FIXTURE_LONGITUDE,
                absolute_minute: StrategicMinute::ZERO.saturating_add_minutes(MINUTES_PER_DAY / 2),
                lunar_phase_minute: StrategicMinute::ZERO
                    .saturating_add_minutes(MINUTES_PER_DAY / 2),
                absolute_elevation_metres: FIXTURE_ELEVATION,
                weather: WeatherSnapshot {
                    rules_version: WEATHER_RULES_VERSION,
                    interval_start_minute:
                        adventuresim_world_schema::calendar::StrategicMinute::new(0),
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

// Authored fixture coordinates are checked during constant evaluation.
const FIXTURE_LATITUDE: adventuresim_world_schema::coordinates::LatitudeMicrodegrees =
    match adventuresim_world_schema::coordinates::LatitudeMicrodegrees::new(53_500_000) {
        Some(value) => value,
        None => panic!("invalid fixture latitude"),
    };
const FIXTURE_LONGITUDE: adventuresim_world_schema::coordinates::LongitudeMicrodegrees =
    match adventuresim_world_schema::coordinates::LongitudeMicrodegrees::new(
        10 * adventuresim_world_schema::coordinates::LongitudeMicrodegrees::UNITS_PER_DEGREE,
    ) {
        Some(value) => value,
        None => panic!("invalid fixture longitude"),
    };
const FIXTURE_ELEVATION: adventuresim_world_schema::ElevationMeters =
    match adventuresim_world_schema::ElevationMeters::new(20) {
        Some(value) => value,
        None => panic!("invalid fixture elevation"),
    };
