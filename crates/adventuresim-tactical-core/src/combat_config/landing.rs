//! Landing compression tuning shared by authority and presentation.

use super::TacticalCombatConfigError;
use crate::animation::AnimationElapsed;
use adventuresim_core::linear_speed::NonnegativeSpeed;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationLandingConfig {
    pub minimum_impact_speed_metres_per_second: NonnegativeSpeed,
    pub compression_per_metre_per_second: f32,
    pub minimum_compression_metres: f32,
    pub maximum_compression_metres: f32,
    pub recovery_seconds: AnimationElapsed,
}

impl Default for AnimationLandingConfig {
    fn default() -> Self {
        super::defaults::canonical().animation.locomotion.landing
    }
}

impl AnimationLandingConfig {
    pub(super) fn validate(&self) -> Result<(), TacticalCombatConfigError> {
        let finite_nonnegative = |value: f32| value.is_finite() && value >= 0.0;
        let landing = self;
        if ![
            landing.compression_per_metre_per_second,
            landing.minimum_compression_metres,
            landing.maximum_compression_metres,
        ]
        .into_iter()
        .all(finite_nonnegative)
            || landing.maximum_compression_metres < landing.minimum_compression_metres
            || landing.recovery_seconds.is_zero()
        {
            return Err(TacticalCombatConfigError::Validation(
                "invalid animation landing tuning",
            ));
        }
        Ok(())
    }
}
