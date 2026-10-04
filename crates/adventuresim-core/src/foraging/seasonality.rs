//! The seasonality contract shared by forage calculation and presentation.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForageSeasonalityPolicy {
    YearRoundUntilStrategicSeasons,
}

pub const SEASONALITY_POLICY: ForageSeasonalityPolicy =
    ForageSeasonalityPolicy::YearRoundUntilStrategicSeasons;
