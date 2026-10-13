//! Shared ground pigment at the material adapter, without synthetic scene state.
use super::bps;
use adventuresim_tactical_core::prelude::{EnvironmentalSample, SceneEnvironment, WeatherSnapshot};
use adventuresim_world_schema::UnitBasisPoints;
use bevy::prelude::Color;

const WATER_DOMINANCE: UnitBasisPoints = UnitBasisPoints::saturating(5_000);
const WETLAND_DOMINANCE: UnitBasisPoints = UnitBasisPoints::saturating(4_000);
const CULTIVATION_DOMINANCE: UnitBasisPoints = UnitBasisPoints::saturating(4_000);
pub(super) const TACTICAL_DIRT_SRGB: [u8; 3] = [101, 82, 49];

/// Protocol coverage is admitted to the existing clamped shader fraction here.
/// These conversions occur at the material port, not during geographic sampling.
struct GroundCover {
    water: UnitBasisPoints,
    wetland: UnitBasisPoints,
    cultivation: UnitBasisPoints,
}

impl From<&SceneEnvironment> for GroundCover {
    fn from(environment: &SceneEnvironment) -> Self {
        Self {
            water: UnitBasisPoints::saturating(environment.water_bps),
            wetland: UnitBasisPoints::saturating(environment.wetland_bps),
            cultivation: UnitBasisPoints::saturating(environment.cultivation_bps),
        }
    }
}

impl From<EnvironmentalSample> for GroundCover {
    fn from(sample: EnvironmentalSample) -> Self {
        Self {
            water: UnitBasisPoints::saturating(sample.water_bps),
            wetland: UnitBasisPoints::saturating(sample.wetland_bps),
            cultivation: UnitBasisPoints::saturating(sample.cultivation_bps),
        }
    }
}

impl GroundCover {
    fn color(self, weather: WeatherSnapshot) -> Color {
        let mut rgb = if self.water >= WATER_DOMINANCE {
            [52.0, 83.0, 98.0]
        } else if self.wetland >= WETLAND_DOMINANCE {
            [70.0, 62.0, 43.0]
        } else if self.cultivation >= CULTIVATION_DOMINANCE {
            [116.0, 91.0, 49.0]
        } else {
            TACTICAL_DIRT_SRGB.map(f32::from)
        };
        let snow = bps(weather.snow_cover_bps);
        let wet = bps(weather.ground_moisture_bps);
        for channel in &mut rgb {
            *channel *= 1.0 - wet * 0.22;
            *channel = *channel * (1.0 - snow) + 220.0 * snow;
        }
        Color::srgb(rgb[0] / 255.0, rgb[1] / 255.0, rgb[2] / 255.0)
    }
}

pub(super) fn scene_ground_color(environment: &SceneEnvironment) -> Color {
    GroundCover::from(environment).color(environment.weather)
}

pub(super) fn sample_ground_color(sample: EnvironmentalSample, weather: WeatherSnapshot) -> Color {
    GroundCover::from(sample).color(weather)
}
