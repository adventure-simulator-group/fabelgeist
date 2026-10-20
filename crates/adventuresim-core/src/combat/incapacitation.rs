use crate::elapsed::{AnimationElapsed, ElapsedError, ElapsedResult};
use crate::prelude::*;
use crate::unit_interval::{FractionResult, UnitInterval};

/// Recovery available at this performance snapshot, not a permanent capability.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(into = "Option<AnimationElapsed>")]
pub enum RecoveryTime {
    After(AnimationElapsed),
    CannotRecover,
}

impl RecoveryTime {
    pub fn available(self) -> Option<AnimationElapsed> {
        match self {
            Self::After(elapsed) => Some(elapsed),
            Self::CannotRecover => None,
        }
    }
}

impl From<RecoveryTime> for Option<AnimationElapsed> {
    fn from(recovery: RecoveryTime) -> Self {
        recovery.available()
    }
}

/// The remaining combat effectiveness after all wheel contributions, not a
/// separate fatigue penalty. Limb injury effects remain independently modeled.
#[must_use]
pub fn combat_incapacitation_performance(incapacitation: f32) -> FractionResult<UnitInterval> {
    UnitInterval::from_fraction(1.0 - incapacitation.clamp(0.0, 1.0))
}

/// Zero recovery needs no performance. Positive recovery is unavailable at zero
/// performance; finite arithmetic overflow remains a timing error.
pub fn incapacitation_adjusted_recovery_seconds(
    base_recovery: AnimationElapsed,
    performance: UnitInterval,
) -> ElapsedResult<RecoveryTime> {
    if base_recovery.is_zero() {
        return Ok(RecoveryTime::After(AnimationElapsed::ZERO));
    }
    let performance = performance.fraction().clamp(0.0, 1.0);
    if performance == 0.0 {
        return Ok(RecoveryTime::CannotRecover);
    }
    let seconds = base_recovery.seconds() / performance;
    if !seconds.is_finite() {
        return Err(ElapsedError::Overflow);
    }
    Ok(RecoveryTime::After(AnimationElapsed::from_seconds(
        seconds,
    )?))
}

/// Burden uses the existing carrying-capacity rule, but is now an explicit
/// combat incapacitation source rather than a hidden physical-skill multiplier.
#[must_use]
pub fn combat_encumbrance_incapacitation(
    attributes: &impl PlayerAttributes,
    body: &impl PlayerBody,
    equipment: &impl PlayerEquipment,
) -> f32 {
    1.0 - equipment.encumbrance_penalty_by_parts(attributes, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn performance_depends_on_total_not_the_source() {
        let fatigue = 0.25;
        let burden = 0.15;
        let injury = 0.10;
        assert_eq!(
            combat_incapacitation_performance(fatigue + burden + injury)
                .unwrap()
                .fraction(),
            0.5
        );
        assert_eq!(
            combat_incapacitation_performance(1.2).unwrap().fraction(),
            0.0
        );
        assert_eq!(
            combat_incapacitation_performance(-0.2).unwrap().fraction(),
            1.0
        );
        assert_eq!(
            incapacitation_adjusted_recovery_seconds(
                AnimationElapsed::from_seconds(0.4).unwrap(),
                crate::unit_interval::UnitInterval::from_fraction(0.5).unwrap()
            )
            .unwrap(),
            RecoveryTime::After(AnimationElapsed::from_seconds(0.8).unwrap())
        );
        assert_eq!(
            incapacitation_adjusted_recovery_seconds(
                AnimationElapsed::from_seconds(0.0).unwrap(),
                crate::unit_interval::UnitInterval::from_fraction(0.0).unwrap()
            )
            .unwrap(),
            RecoveryTime::After(AnimationElapsed::ZERO)
        );
        assert_eq!(
            incapacitation_adjusted_recovery_seconds(
                AnimationElapsed::from_seconds(0.4).unwrap(),
                crate::unit_interval::UnitInterval::from_fraction(0.0).unwrap()
            )
            .unwrap(),
            RecoveryTime::CannotRecover
        );
    }
}
