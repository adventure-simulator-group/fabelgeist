//! Validate the combined autonomous-control and combat tuning.

use super::{TacticalAiConfig, TacticalCombatConfigError};

impl TacticalAiConfig {
    pub(super) fn validate(&self) -> Result<(), TacticalCombatConfigError> {
        self.control.validate()?;
        let ai = &self.ordinary;
        if !ai.defense.dodge_chance.is_finite() || !(0.0..=1.0).contains(&ai.defense.dodge_chance) {
            return Err(TacticalCombatConfigError::Validation(
                "AI dodge probability must be between zero and one",
            ));
        }
        if ai.defense.reaction_delay_max_seconds < ai.defense.reaction_delay_min_seconds
            || !ai.offense.has_valid_intervals()
        {
            return Err(TacticalCombatConfigError::Validation("invalid AI interval"));
        }
        Ok(())
    }
}
