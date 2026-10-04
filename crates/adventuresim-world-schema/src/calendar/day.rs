//! Absolute zero-based strategic days, distinct from minutes and durations.
//!
//! Native row and protocol integers are admitted at construction. Day ordering,
//! weekday projection, inclusive iteration and conversion to minutes belong here.
//!
//! ```compile_fail
//! use adventuresim_world_schema::calendar::{StrategicDayIndex, StrategicMinute};
//! let day = StrategicDayIndex::new(3);
//! let at: StrategicMinute = day;
//! ```

use super::{DAYS_PER_WEEK, MINUTES_PER_DAY, StrategicMinute};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct StrategicDayIndex(u64);

impl StrategicDayIndex {
    pub const FIRST: Self = Self(0);
    pub const MAX: Self = Self(u64::MAX);

    pub const fn new(index: u64) -> Self {
        Self(index)
    }
    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn following(self) -> Self {
        Self(self.0.saturating_add(1))
    }
    pub const fn checked_following(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(index) => Some(Self(index)),
            None => None,
        }
    }
    pub const fn preceding(self) -> Self {
        Self(self.0.saturating_sub(1))
    }

    /// Stored day indices beyond the minute range preserve saturating projection.
    pub const fn start(self) -> StrategicMinute {
        StrategicMinute::new(self.0.saturating_mul(MINUTES_PER_DAY))
    }
    pub const fn checked_start(self) -> Option<StrategicMinute> {
        match self.0.checked_mul(MINUTES_PER_DAY) {
            Some(minutes) => Some(StrategicMinute::new(minutes)),
            None => None,
        }
    }

    pub const fn weekday(self) -> StrategicWeekday {
        match self.0 % DAYS_PER_WEEK {
            0 => StrategicWeekday::Monday,
            1 => StrategicWeekday::Tuesday,
            2 => StrategicWeekday::Wednesday,
            3 => StrategicWeekday::Thursday,
            4 => StrategicWeekday::Friday,
            5 => StrategicWeekday::Saturday,
            _ => StrategicWeekday::Sunday,
        }
    }

    pub fn through(self, last: Self) -> StrategicDays {
        StrategicDays {
            next: (self <= last).then_some(self),
            last,
        }
    }
}

impl std::fmt::Display for StrategicDayIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl From<u64> for StrategicDayIndex {
    fn from(index: u64) -> Self {
        Self(index)
    }
}
impl From<StrategicDayIndex> for u64 {
    fn from(day: StrategicDayIndex) -> Self {
        day.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StrategicWeekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// Inclusive, ordered days. The final day is emitted once even at `u64::MAX`.
#[derive(Clone, Debug)]
pub struct StrategicDays {
    next: Option<StrategicDayIndex>,
    last: StrategicDayIndex,
}

impl Iterator for StrategicDays {
    type Item = StrategicDayIndex;
    fn next(&mut self) -> Option<Self::Item> {
        let day = self.next?;
        self.next = if day == self.last {
            None
        } else {
            day.checked_following()
        };
        Some(day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_days_floor_minutes_and_preserve_saturating_and_checked_projection() {
        for minute in [0, MINUTES_PER_DAY - 1, MINUTES_PER_DAY, u64::MAX] {
            let at = StrategicMinute::new(minute);
            let day = at.day_index();
            assert_eq!(day.get(), minute / MINUTES_PER_DAY);
            assert_eq!(day.checked_start(), Some(at.day_start()));
            assert!(day.start() <= at);
        }
        let last = StrategicDayIndex::new(u64::MAX / MINUTES_PER_DAY);
        assert!(last.checked_start().is_some());
        assert_eq!(last.following().checked_start(), None);
        assert_eq!(StrategicDayIndex::MAX.start(), StrategicMinute::MAX);
    }

    #[test]
    fn inclusive_days_are_ordered_empty_when_reversed_and_finite_at_maximum() {
        let first = StrategicDayIndex::new(4);
        assert_eq!(
            first.through(first.following()).collect::<Vec<_>>(),
            [first, first.following()]
        );
        assert_eq!(first.following().through(first).next(), None);
        let mut last = StrategicDayIndex::MAX
            .preceding()
            .through(StrategicDayIndex::MAX);
        assert_eq!(last.next(), Some(StrategicDayIndex::MAX.preceding()));
        assert_eq!(last.next(), Some(StrategicDayIndex::MAX));
        assert_eq!(last.next(), None);
        assert_eq!(last.next(), None);
    }

    #[test]
    fn weekday_order_keeps_the_epoch_monday_and_repeats_each_week() {
        let expected = [
            StrategicWeekday::Monday,
            StrategicWeekday::Tuesday,
            StrategicWeekday::Wednesday,
            StrategicWeekday::Thursday,
            StrategicWeekday::Friday,
            StrategicWeekday::Saturday,
            StrategicWeekday::Sunday,
        ];
        for (index, weekday) in expected.into_iter().enumerate() {
            let index = index as u64;
            assert_eq!(StrategicDayIndex::new(index).weekday(), weekday);
            assert_eq!(
                StrategicDayIndex::new(index + DAYS_PER_WEEK).weekday(),
                weekday
            );
        }
    }
}
