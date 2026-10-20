use crate::animation::PositiveAnimationDuration;
use serde::{Deserialize, Serialize};

/// Yellow wheel extensions show recent deterioration projected over this
/// horizon. These durations affect presentation, never combat outcomes.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IncapacitationForecastConfig {
    pub horizon_seconds: PositiveAnimationDuration,
    pub trend_response_seconds: PositiveAnimationDuration,
}

impl Default for IncapacitationForecastConfig {
    fn default() -> Self {
        super::defaults::canonical()
            .presentation
            .incapacitation_forecast
    }
}
