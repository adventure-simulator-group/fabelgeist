use super::TacticalCombatConfigError;
use crate::animation::{SignedHalfTurns, UnitInterval};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationStateTransitionConfig {
    /// Start of root-facing transfer within normalized dive contact recovery.
    pub dive_root_handoff_start_fraction: UnitInterval,
    /// End of airborne root-facing transfer within normalized contact recovery.
    pub dive_airborne_root_handoff_end_fraction: UnitInterval,
    pub downed_facing_sector_half_width: SignedHalfTurns,
    pub downed_facing_edge_stickiness: SignedHalfTurns,
}

impl AnimationStateTransitionConfig {
    pub(super) fn validate(&self) -> Result<(), TacticalCombatConfigError> {
        if self.dive_root_handoff_start_fraction >= self.dive_airborne_root_handoff_end_fraction {
            return Err(TacticalCombatConfigError::Validation(
                "dive root handoff end must exceed its start",
            ));
        }
        Ok(())
    }
}
