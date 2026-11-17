//! Preserve the single visible fatigue value across a live/durable handoff.

use super::{PlayerAttributes, PlayerBody, PlayerEssentials, fatigue_capacity_kcal};
use crate::morale::{
    FATIGUE_IMPAIRMENT_ONSET, fatigue_incapacitation, fatigue_ratio_for_incapacitation,
};

/// A finite nonnegative fraction of incapacitation caused by general fatigue.
/// Existing durable fatigue can exceed the wheel's display cutoff, so this
/// checked quantity deliberately does not truncate values at that cutoff.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FatigueIncapacitation(f32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum FatigueHandoffError {
    #[error("Fatigue incapacitation must be finite and nonnegative")]
    InvalidIncapacitation,
    #[error("Stored fatigue and its calorie capacity must be finite and valid")]
    InvalidStoredFatigue,
}

impl FatigueIncapacitation {
    pub fn fraction(self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for FatigueIncapacitation {
    type Error = FatigueHandoffError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if !value.is_finite() || value < 0.0 {
            return Err(FatigueHandoffError::InvalidIncapacitation);
        }
        Ok(Self(value))
    }
}

/// Convert a final live fatigue fraction to the durable calorie coordinate.
/// Native floats are used only at this existing physiology-kernel boundary.
/// An unchanged value preserves subthreshold reserves and avoids granting
/// recovery merely by entering, reconnecting or leaving a scene.
pub fn fatigue_calories_after_handoff(
    starting: &impl PlayerEssentials,
    attributes: &impl PlayerAttributes,
    body: &impl PlayerBody,
    final_fatigue: FatigueIncapacitation,
) -> Result<f32, FatigueHandoffError> {
    let calories = starting.calories_used_today();
    let capacity = fatigue_capacity_kcal(attributes, body);
    if !calories.is_finite() || calories < 0.0 || !capacity.is_finite() || capacity <= 0.0 {
        return Err(FatigueHandoffError::InvalidStoredFatigue);
    }
    let before = fatigue_incapacitation(calories / capacity);
    if before == final_fatigue.fraction() {
        return Ok(calories);
    }
    let after = if final_fatigue.fraction() == 0.0 {
        calories.min(capacity * FATIGUE_IMPAIRMENT_ONSET)
    } else {
        capacity * fatigue_ratio_for_incapacitation(final_fatigue.fraction())
    };
    if !after.is_finite() {
        return Err(FatigueHandoffError::InvalidStoredFatigue);
    }
    Ok(after)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stub::{StubAttributes, StubBody};

    struct SpentCalories(f32);

    impl PlayerEssentials for SpentCalories {
        fn calories_used_today(&self) -> f32 {
            self.0
        }
        fn focus_level(&self) -> f32 {
            1.0
        }
    }

    #[test]
    fn unchanged_visible_fatigue_preserves_reserves_at_every_handoff() {
        for calories in [0.0, 250.0, 500.0, 750.0, 1_000.0, 2_000.0] {
            let starting = SpentCalories(calories);
            let fatigue = FatigueIncapacitation::try_from(fatigue_incapacitation(
                starting.fatigue_by_parts(&StubAttributes, &StubBody),
            ))
            .unwrap();
            assert_eq!(
                fatigue_calories_after_handoff(&starting, &StubAttributes, &StubBody, fatigue)
                    .unwrap(),
                calories
            );
        }
    }

    #[test]
    fn work_and_recovery_follow_the_existing_single_fatigue_curve() {
        let worked = fatigue_calories_after_handoff(
            &SpentCalories(250.0),
            &StubAttributes,
            &StubBody,
            FatigueIncapacitation::try_from(0.25).unwrap(),
        )
        .unwrap();
        assert_eq!(worked, 750.0);
        let recovered = fatigue_calories_after_handoff(
            &SpentCalories(worked),
            &StubAttributes,
            &StubBody,
            FatigueIncapacitation::try_from(0.0).unwrap(),
        )
        .unwrap();
        assert_eq!(recovered, 500.0);
        assert_eq!(
            fatigue_incapacitation(recovered / fatigue_capacity_kcal(&StubAttributes, &StubBody)),
            0.0
        );
        assert!(FatigueIncapacitation::try_from(f32::NAN).is_err());
        assert!(FatigueIncapacitation::try_from(-0.1).is_err());
    }
}
