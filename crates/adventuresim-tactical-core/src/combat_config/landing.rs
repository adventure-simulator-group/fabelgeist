//! Landing compression tuning shared by authority and presentation.

use super::TacticalCombatConfigError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationLandingConfig {
    pub minimum_impact_speed_metres_per_second: f32,
    pub compression_per_metre_per_second: f32,
    pub minimum_compression_metres: f32,
    pub maximum_compression_metres: f32,
    pub recovery_seconds: f32,
}

impl Default for AnimationLandingConfig {
    fn default() -> Self {
        Self {
            minimum_impact_speed_metres_per_second: 1.0,
            compression_per_metre_per_second: 0.012,
            minimum_compression_metres: 0.04,
            maximum_compression_metres: 0.08,
            recovery_seconds: 0.16,
        }
    }
}

impl AnimationLandingConfig {
    pub(super) fn validate(&self) -> Result<(), TacticalCombatConfigError> {
        let finite_nonnegative = |value: f32| value.is_finite() && value >= 0.0;
        let landing = self;
        if ![
            landing.minimum_impact_speed_metres_per_second,
            landing.compression_per_metre_per_second,
            landing.minimum_compression_metres,
            landing.maximum_compression_metres,
            landing.recovery_seconds,
        ]
        .into_iter()
        .all(finite_nonnegative)
            || landing.maximum_compression_metres < landing.minimum_compression_metres
            || landing.recovery_seconds == 0.0
        {
            return Err(TacticalCombatConfigError::Validation(
                "invalid animation landing tuning",
            ));
        }
        Ok(())
    }
}
