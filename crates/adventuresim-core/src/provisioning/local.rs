//! Unassisted needs during a live interval. Supplies require an explicit action.

use adventuresim_world_schema::calendar::MINUTES_PER_DAY;

use super::{STRATEGIC_TRAVEL_KCAL_PER_DAY, STRATEGIC_TRAVEL_WATER_ML_PER_DAY};
use crate::{morale, strategic_time::PersonalClock};

/// Native calorie/millilitre balances at the existing physiology-kernel boundary.
/// Negative balances represent a deficit, using the same curve as travel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalNeeds {
    food_balance_kcal: f32,
    water_balance_ml: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum LocalNeedsError {
    #[error("Local food and water balances must be finite")]
    NonfiniteBalance,
    #[error("Local needs cannot advance backwards in personal time")]
    BackwardsInterval,
}

impl LocalNeeds {
    pub fn from_balances_kcal_ml(
        food_balance_kcal: f32,
        water_balance_ml: f32,
    ) -> Result<Self, LocalNeedsError> {
        if !food_balance_kcal.is_finite() || !water_balance_ml.is_finite() {
            return Err(LocalNeedsError::NonfiniteBalance);
        }
        Ok(Self {
            food_balance_kcal,
            water_balance_ml,
        })
    }

    /// Always project from the captured interval start, never from a preceding
    /// frame. Whole minutes debit once; the personal clock carries fractions
    /// across subsequent visits and journeys.
    pub fn advance_between(
        self,
        starting: PersonalClock,
        ending: PersonalClock,
    ) -> Result<Self, LocalNeedsError> {
        let minutes = ending
            .minute()
            .checked_elapsed_since(starting.minute())
            .ok_or(LocalNeedsError::BackwardsInterval)?;
        if minutes == 0 && u64::from(ending.fraction()) < u64::from(starting.fraction()) {
            return Err(LocalNeedsError::BackwardsInterval);
        }
        let days = minutes as f32 / MINUTES_PER_DAY as f32;
        Self::from_balances_kcal_ml(
            self.food_balance_kcal - days * STRATEGIC_TRAVEL_KCAL_PER_DAY,
            self.water_balance_ml - days * STRATEGIC_TRAVEL_WATER_ML_PER_DAY,
        )
    }

    pub fn food_balance_kcal(self) -> f32 {
        self.food_balance_kcal
    }
    pub fn water_balance_ml(self) -> f32 {
        self.water_balance_ml
    }

    pub fn hunger(self) -> f32 {
        morale::hunger_incapacitation(self.food_balance_kcal, STRATEGIC_TRAVEL_KCAL_PER_DAY)
    }

    pub fn thirst(self) -> f32 {
        morale::thirst_incapacitation(self.water_balance_ml, STRATEGIC_TRAVEL_WATER_ML_PER_DAY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic_time::CalendarMinuteFraction;
    use adventuresim_world_schema::calendar::WORLD_START_MINUTE;
    use std::time::Duration;

    #[test]
    fn repeated_previews_do_not_consume_extra_supplies_or_hide_deficits() {
        let initial = LocalNeeds::from_balances_kcal_ml(6_000.0, 4_000.0).unwrap();
        let starting = PersonalClock::new(WORLD_START_MINUTE, CalendarMinuteFraction::ZERO);
        let ending = PersonalClock::new(
            WORLD_START_MINUTE.checked_add_days(2).unwrap(),
            starting.fraction(),
        );
        let committed = initial.advance_between(starting, ending).unwrap();
        for _ in 0..1_000 {
            assert_eq!(
                initial.advance_between(starting, ending).unwrap(),
                committed
            );
        }
        assert_eq!(committed.food_balance_kcal(), -6_000.0);
        assert_eq!(committed.water_balance_ml(), -4_000.0);
        assert!(committed.hunger() > 0.0);
        assert!(committed.thirst() > 0.0);
    }

    #[test]
    fn visits_carry_fraction_until_a_whole_minute_is_consumed() {
        let initial = LocalNeeds::from_balances_kcal_ml(6_000.0, 4_000.0).unwrap();
        let starting = PersonalClock::new(WORLD_START_MINUTE, CalendarMinuteFraction::ZERO);
        let midpoint = starting
            .advance_real_time(Duration::from_millis(600))
            .unwrap();
        let ending = midpoint
            .advance_real_time(Duration::from_millis(600))
            .unwrap();
        assert_eq!(
            initial.advance_between(starting, midpoint).unwrap(),
            initial
        );
        assert_eq!(
            initial.advance_between(midpoint, ending).unwrap(),
            initial.advance_between(starting, ending).unwrap()
        );
        assert!(ending.minute() > starting.minute());
        assert!(initial.advance_between(ending, midpoint).is_err());
    }
}
