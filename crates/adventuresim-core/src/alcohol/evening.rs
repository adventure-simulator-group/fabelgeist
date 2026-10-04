//! Stable nightly identities and interval opportunities at the alcohol boundary.
//!
//! ```compile_fail
//! use adventuresim_core::alcohol::{nightly_morale_effect, TemperancePreference};
//! use adventuresim_world_schema::calendar::StrategicDayIndex;
//! nightly_morale_effect(StrategicDayIndex::new(3), TemperancePreference::Neutral, false, true);
//! ```

use super::{
    AlcoholIntervalError, EVENING_BOUNDARY_MINUTE, NIGHT_END_MINUTE, ROLLING_WEEK_DAYS,
    checked_interval,
};
use adventuresim_world_schema::calendar::{StrategicDayIndex, StrategicMinute};

/// An evening's identity is its boundary's absolute day, rather than the day
/// of a later overnight observation. Before the first boundary, identity is zero.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EveningId(StrategicDayIndex);

impl EveningId {
    pub const FIRST: Self = Self(StrategicDayIndex::FIRST);
    pub const fn new(index: u64) -> Self {
        Self(StrategicDayIndex::new(index))
    }
    pub const fn get(self) -> u64 {
        self.0.get()
    }

    pub const fn from_minute(minute: StrategicMinute) -> Self {
        Self(
            minute
                .saturating_sub_minutes(EVENING_BOUNDARY_MINUTE)
                .day_index(),
        )
    }
    pub const fn boundary(self) -> Option<StrategicMinute> {
        match self.0.checked_start() {
            Some(start) => start.checked_add_minutes(EVENING_BOUNDARY_MINUTE),
            None => None,
        }
    }
    /// The earlier evening belongs to the rolling week only when strictly
    /// earlier and fewer than seven boundaries before the selected evening.
    pub const fn is_recent_prior_to(self, later: Self) -> bool {
        self.get() < later.get() && later.get() - self.get() < ROLLING_WEEK_DAYS
    }
    const fn checked_following(self) -> Option<Self> {
        match self.0.checked_following() {
            Some(day) => Some(Self(day)),
            None => None,
        }
    }
}
impl std::fmt::Display for EveningId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl From<EveningId> for u64 {
    fn from(evening: EveningId) -> Self {
        evening.get()
    }
}

pub const fn next_evening_boundary_after(minute: StrategicMinute) -> Option<StrategicMinute> {
    let next = if StrategicMinute::new(EVENING_BOUNDARY_MINUTE).is_after(minute) {
        EveningId::FIRST
    } else {
        match EveningId::from_minute(minute).checked_following() {
            Some(evening) => evening,
            None => return None,
        }
    };
    next.boundary()
}

#[derive(Clone, Debug)]
pub struct EveningIds {
    next: Option<EveningId>,
    last: Option<EveningId>,
}
impl Iterator for EveningIds {
    type Item = EveningId;
    fn next(&mut self) -> Option<Self::Item> {
        let last = self.last?;
        let value = self.next?;
        if value > last {
            self.next = None;
            return None;
        }
        self.next = if value == last {
            None
        } else {
            value.checked_following()
        };
        Some(value)
    }
}

pub fn crossed_evenings(
    start: StrategicMinute,
    end: StrategicMinute,
) -> Result<EveningIds, AlcoholIntervalError> {
    checked_interval(start, end)?;
    let first = if start < StrategicMinute::new(EVENING_BOUNDARY_MINUTE) {
        EveningId::FIRST
    } else {
        EveningId::from_minute(start)
            .checked_following()
            .ok_or(AlcoholIntervalError::EveningOverflow)?
    };
    let last =
        (end >= StrategicMinute::new(EVENING_BOUNDARY_MINUTE)).then(|| EveningId::from_minute(end));
    Ok(EveningIds {
        next: Some(first),
        last,
    })
}

#[derive(Clone, Debug)]
pub struct RestEvenings {
    next: Option<EveningId>,
    last: EveningId,
    start: StrategicMinute,
    end: StrategicMinute,
}
impl Iterator for RestEvenings {
    type Item = EveningId;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(evening) = self.next {
            if evening > self.last {
                self.next = None;
                return None;
            }
            self.next = if evening == self.last {
                None
            } else {
                evening.checked_following()
            };
            let boundary = evening.boundary()?;
            let sleep_end = boundary
                .day_start()
                .checked_add_days(1)?
                .checked_add_minutes(NIGHT_END_MINUTE)?;
            if self.start < sleep_end && self.end >= boundary {
                return Some(evening);
            }
        }
        None
    }
}

/// Nightly opportunities overlapped by a rest interval, including the current
/// evening for rests beginning after 18:00 or before 08:00.
pub fn rest_evenings(
    start: StrategicMinute,
    end: StrategicMinute,
) -> Result<RestEvenings, AlcoholIntervalError> {
    checked_interval(start, end)?;
    Ok(RestEvenings {
        next: Some(EveningId(start.day_index().preceding())),
        last: EveningId(end.day_index()),
        start,
        end,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alcohol::MAX_ALCOHOL_INTERVAL_MINUTES;
    use adventuresim_world_schema::calendar::{DAYS_PER_YEAR, MINUTES_PER_DAY};

    #[test]
    fn nightly_identity_changes_at_six_pm_and_keeps_overnight_history() {
        let boundary = StrategicMinute::new(MINUTES_PER_DAY + EVENING_BOUNDARY_MINUTE);
        assert_eq!(
            EveningId::from_minute(boundary.saturating_sub_minutes(1)),
            EveningId::FIRST
        );
        assert_eq!(EveningId::from_minute(boundary), EveningId::new(1));
        assert_eq!(
            EveningId::from_minute(boundary.saturating_add_minutes(14 * 60)),
            EveningId::new(1)
        );
        assert_eq!(EveningId::new(u64::MAX).boundary(), None);
    }

    #[test]
    fn rolling_week_excludes_the_current_future_and_seventh_prior_evening() {
        let selected = EveningId::new(23);
        assert!(!selected.is_recent_prior_to(selected));
        assert!(EveningId::new(22).is_recent_prior_to(selected));
        assert!(EveningId::new(17).is_recent_prior_to(selected));
        assert!(!EveningId::new(16).is_recent_prior_to(selected));
        assert!(!EveningId::new(24).is_recent_prior_to(selected));
    }
    #[test]
    fn evening_enumeration_is_chunk_invariant() {
        let whole: Vec<_> = crossed_evenings(StrategicMinute::new(0), StrategicMinute::new(4_000))
            .unwrap()
            .collect();
        let split: Vec<_> = crossed_evenings(StrategicMinute::new(0), StrategicMinute::new(2_000))
            .unwrap()
            .chain(
                crossed_evenings(StrategicMinute::new(2_000), StrategicMinute::new(4_000)).unwrap(),
            )
            .collect();
        assert_eq!(whole, split);
        assert_eq!(
            whole,
            vec![EveningId::new(0), EveningId::new(1), EveningId::new(2)]
        );
    }

    #[test]
    fn rest_after_evening_boundary_includes_the_current_night() {
        assert_eq!(
            rest_evenings(
                StrategicMinute::new(19 * 60),
                StrategicMinute::new(MINUTES_PER_DAY + 7 * 60)
            )
            .unwrap()
            .collect::<Vec<_>>(),
            vec![EveningId::FIRST]
        );
        assert_eq!(
            rest_evenings(
                StrategicMinute::new(MINUTES_PER_DAY + 2 * 60),
                StrategicMinute::new(MINUTES_PER_DAY + 8 * 60)
            )
            .unwrap()
            .collect::<Vec<_>>(),
            vec![EveningId::FIRST]
        );
    }

    #[test]
    fn nightly_rest_enumeration_is_chunk_idempotent() {
        let whole: Vec<_> = rest_evenings(
            StrategicMinute::new(19 * 60),
            StrategicMinute::new(3 * MINUTES_PER_DAY + 7 * 60),
        )
        .unwrap()
        .collect();
        let mut split: Vec<_> = rest_evenings(
            StrategicMinute::new(19 * 60),
            StrategicMinute::new(MINUTES_PER_DAY + 7 * 60),
        )
        .unwrap()
        .chain(
            rest_evenings(
                StrategicMinute::new(MINUTES_PER_DAY + 7 * 60),
                StrategicMinute::new(3 * MINUTES_PER_DAY + 7 * 60),
            )
            .unwrap(),
        )
        .collect();
        split.dedup();
        assert_eq!(whole, split);
        assert_eq!(
            whole,
            vec![EveningId::new(0), EveningId::new(1), EveningId::new(2)]
        );
    }

    #[test]
    fn evening_iterators_reject_reversed_or_unbounded_work() {
        assert!(crossed_evenings(StrategicMinute::new(2), StrategicMinute::new(1)).is_err());
        assert!(
            rest_evenings(
                StrategicMinute::new(0),
                StrategicMinute::new(MAX_ALCOHOL_INTERVAL_MINUTES + 1)
            )
            .is_err()
        );
        assert_eq!(
            crossed_evenings(
                StrategicMinute::new(0),
                StrategicMinute::new(MAX_ALCOHOL_INTERVAL_MINUTES)
            )
            .unwrap()
            .count(),
            usize::try_from(DAYS_PER_YEAR).unwrap()
        );
    }
}
