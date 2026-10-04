//! Shared non-leap strategic calendar and its absolute minute coordinate.
//!
//! Calendar calculations and stored absolute instants use [`CalendarYear`] and
//! [`StrategicMinute`]. Transport adapters convert to raw integers only where
//! their protocols require it.

#[path = "calendar/duration.rs"]
mod duration;
pub use duration::StrategicDuration;
#[path = "calendar/day.rs"]
mod day;
pub use day::{StrategicDayIndex, StrategicDays, StrategicWeekday};
use serde::{Deserialize, Serialize};
use std::fmt;

pub const MINUTES_PER_HOUR: u64 = 60;
pub const MINUTES_PER_DAY: u64 = 24 * MINUTES_PER_HOUR;
pub const DAYS_PER_WEEK: u64 = 7;
pub const DAYS_PER_YEAR: u64 = 365;
pub const MINUTES_PER_YEAR: u64 = DAYS_PER_YEAR * MINUTES_PER_DAY;
/// Calendar year containing strategic minute zero.
pub const WORLD_START_YEAR: CalendarYear = CalendarYear { year: 1544 };
/// August 20 at 00:00 in the shared non-leap strategic calendar.
pub const WORLD_START_DAY_OF_YEAR: u64 = 231;
pub const WORLD_START_MINUTE: StrategicMinute = StrategicMinute {
    minutes: WORLD_START_DAY_OF_YEAR * MINUTES_PER_DAY,
};

/// A positive calendar year. There is no year zero in this calendar.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "i32")]
pub struct CalendarYear {
    year: i32,
}

impl fmt::Display for CalendarYear {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.year.fmt(formatter)
    }
}

impl CalendarYear {
    pub const fn new(year: i32) -> Option<Self> {
        if year > 0 { Some(Self { year }) } else { None }
    }

    pub const fn get(self) -> i32 {
        self.year
    }

    /// Find the birth year implied by a whole-year age at this date.
    pub const fn birth_year_for_age(self, age_years: u16) -> Option<Self> {
        self.checked_sub_years(age_years)
    }

    pub const fn checked_sub_years(self, years: u16) -> Option<Self> {
        Self::new(self.year - years as i32)
    }

    /// Position of this year between two ordered calendar years.
    pub fn interpolation_fraction_between(self, first: Self, last: Self) -> Option<f64> {
        (first < last && first <= self && self <= last)
            .then(|| f64::from(self.year - first.year) / f64::from(last.year - first.year))
    }
}

impl TryFrom<i32> for CalendarYear {
    type Error = InvalidCalendarYear;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(InvalidCalendarYear)
    }
}

/// The strategic calendar has no zero or negative years.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidCalendarYear;

impl fmt::Display for InvalidCalendarYear {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("calendar year must be positive")
    }
}

impl std::error::Error for InvalidCalendarYear {}

#[cfg(feature = "spacetimedb")]
crate::checked_sats::checked_numeric_product!(CalendarYear, year: i32);

impl From<CalendarYear> for i32 {
    fn from(value: CalendarYear) -> Self {
        value.get()
    }
}

/// Absolute minutes since the strategic calendar's zero minute.
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(crate = spacetimedb_lib))]
#[serde(transparent)]
pub struct StrategicMinute {
    minutes: u64,
}

impl fmt::Display for StrategicMinute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.minutes.fmt(formatter)
    }
}

impl StrategicMinute {
    pub const ZERO: Self = Self::new(0);
    pub const MAX: Self = Self::new(u64::MAX);

    pub const fn new(minutes: u64) -> Self {
        Self { minutes }
    }

    pub const fn get(self) -> u64 {
        self.minutes
    }

    pub const fn is_after(self, other: Self) -> bool {
        self.minutes > other.minutes
    }

    pub const fn saturating_add_minutes(self, elapsed_minutes: u64) -> Self {
        Self::new(self.minutes.saturating_add(elapsed_minutes))
    }

    /// Advance by whole calendar days, saturating at the last absolute minute.
    pub const fn saturating_add_days(self, days: u64) -> Self {
        self.saturating_add_minutes(days.saturating_mul(MINUTES_PER_DAY))
    }

    /// Advance by whole calendar days only if the resulting minute fits.
    pub const fn checked_add_days(self, days: u64) -> Option<Self> {
        match days.checked_mul(MINUTES_PER_DAY) {
            Some(minutes) => self.checked_add_minutes(minutes),
            None => None,
        }
    }

    /// Retreat by whole calendar days, saturating at the epoch.
    pub const fn saturating_sub_days(self, days: u64) -> Self {
        self.saturating_sub_minutes(days.saturating_mul(MINUTES_PER_DAY))
    }

    pub const fn checked_add_minutes(self, elapsed_minutes: u64) -> Option<Self> {
        match self.minutes.checked_add(elapsed_minutes) {
            Some(minutes) => Some(Self::new(minutes)),
            None => None,
        }
    }

    pub const fn saturating_sub_minutes(self, elapsed_minutes: u64) -> Self {
        Self::new(self.minutes.saturating_sub(elapsed_minutes))
    }

    pub const fn elapsed_since(self, earlier: Self) -> u64 {
        self.minutes.saturating_sub(earlier.minutes)
    }

    pub const fn checked_elapsed_since(self, earlier: Self) -> Option<u64> {
        self.minutes.checked_sub(earlier.minutes)
    }

    /// Whole or partial calendar days until a later absolute minute.
    pub const fn days_until_ceil(self, later: Self) -> u64 {
        later.elapsed_since(self).div_ceil(MINUTES_PER_DAY)
    }

    /// Midpoint between ordered absolute minutes, rounded toward `self`.
    pub const fn midpoint(self, later: Self) -> Self {
        Self::new(
            self.minutes
                .saturating_add(later.minutes.saturating_sub(self.minutes) / 2),
        )
    }

    /// Inclusive absolute minutes after this instant through `end`.
    pub fn iter_after_through(self, end: Self) -> impl Iterator<Item = Self> {
        (self.minutes.saturating_add(1)..=end.minutes)
            .filter(move |_| self < end)
            .map(Self::new)
    }

    /// Inclusive absolute minutes from this instant through `end`.
    pub fn iter_through(self, end: Self) -> impl Iterator<Item = Self> {
        (self.minutes..=end.minutes).map(Self::new)
    }

    pub const fn saturating_add_years(self, years: u16) -> Self {
        self.saturating_add_minutes((years as u64).saturating_mul(MINUTES_PER_YEAR))
    }

    pub const fn whole_years_since(self, earlier: Self) -> u64 {
        self.elapsed_since(earlier) / MINUTES_PER_YEAR
    }

    /// Age in complete years from the signed birth coordinate used by storage.
    pub fn age_years_since_signed_birth(self, birth_minute: i64) -> u16 {
        let years =
            self.elapsed_minutes_since_signed_birth(birth_minute) / (MINUTES_PER_YEAR as u128);
        years.min(u16::MAX as u128) as u16
    }

    /// Signed birth coordinate for an age at this instant, capped to storage.
    pub fn signed_birth_minute_for_age(self, age_years: u16) -> i64 {
        let birth = i128::from(self.minutes)
            .saturating_sub(i128::from(age_years) * i128::from(MINUTES_PER_YEAR));
        birth.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
    }

    pub fn elapsed_minutes_since_signed_birth(self, birth_minute: i64) -> u128 {
        (i128::from(self.minutes) - i128::from(birth_minute)).max(0) as u128
    }

    /// Progress toward an age threshold in caller-selected integer units.
    pub fn scaled_progress_toward_signed_birth_age(
        self,
        birth_minute: i64,
        target_years: u16,
        whole: u16,
    ) -> u16 {
        if target_years == 0 {
            return whole;
        }
        let elapsed = self.elapsed_minutes_since_signed_birth(birth_minute);
        let target = u128::from(target_years) * u128::from(MINUTES_PER_YEAR);
        elapsed
            .saturating_mul(u128::from(whole))
            .div_euclid(target)
            .min(u128::from(whole)) as u16
    }

    /// Whether a stored signed birth coordinate has occurred by this minute.
    pub const fn is_at_or_after_signed_birth(self, birth_minute: i64) -> bool {
        (self.minutes as i128) >= (birth_minute as i128)
    }

    /// Absolute anniversary of a stored signed birth coordinate.
    pub fn at_signed_birth_age(birth_minute: i64, years: u16) -> Option<Self> {
        let minute = i128::from(birth_minute)
            .checked_add(i128::from(years) * i128::from(MINUTES_PER_YEAR))?;
        u64::try_from(minute).ok().map(Self::new)
    }

    /// First annual birthday after this instant and before an exclusive end.
    pub fn next_signed_birth_anniversary_before(
        self,
        birth_minute: i64,
        end: Self,
    ) -> Option<Self> {
        let completed =
            self.elapsed_minutes_since_signed_birth(birth_minute) / u128::from(MINUTES_PER_YEAR);
        let next = i128::from(birth_minute).checked_add(
            i128::try_from(completed.checked_add(1)?).ok()? * i128::from(MINUTES_PER_YEAR),
        )?;
        let next = Self::new(u64::try_from(next).ok()?);
        (self < next && next < end).then_some(next)
    }

    /// Zero-based day count since the calendar epoch.
    pub const fn day_index(self) -> StrategicDayIndex {
        StrategicDayIndex::new(self.minutes / MINUTES_PER_DAY)
    }

    pub const fn period_index(self, period_minutes: u64) -> Option<u64> {
        self.minutes.checked_div(period_minutes)
    }

    pub const fn offset_within_interval_minutes(self, interval_minutes: u64) -> Option<u64> {
        self.minutes.checked_rem(interval_minutes)
    }

    /// Cumulative integer allocation for a fixed share of each calendar day.
    pub const fn cumulative_daily_share_minutes(self, share_minutes: u64) -> u64 {
        self.minutes.saturating_mul(share_minutes) / MINUTES_PER_DAY
    }

    /// Fractional elapsed days for astronomical presentation.
    pub fn days_since_epoch_f32(self) -> f32 {
        self.minutes as f32 / MINUTES_PER_DAY as f32
    }

    pub const fn day_start(self) -> Self {
        self.day_index().start()
    }

    /// Absolute start of a fixed-width period, if the width and result fit.
    pub const fn checked_period_start_for_index(
        period_index: u64,
        period_minutes: u64,
    ) -> Option<Self> {
        if period_minutes == 0 {
            return None;
        }
        match period_index.checked_mul(period_minutes) {
            Some(minutes) => Some(Self::new(minutes)),
            None => None,
        }
    }

    /// Advance the time of day while keeping this minute's calendar day.
    pub const fn wrapping_day_offset(self, elapsed_minutes: u64) -> Self {
        let minute_of_day =
            (self.minute_of_day() as u64 + elapsed_minutes % MINUTES_PER_DAY) % MINUTES_PER_DAY;
        self.day_start().saturating_add_minutes(minute_of_day)
    }

    /// Place a local clock reading on this minute's calendar day, wrapping
    /// elapsed journey time within that day.
    pub const fn with_wrapped_time_of_day(
        self,
        starting_minute_of_day: u16,
        elapsed_minutes: u64,
    ) -> Self {
        let minute_of_day = (starting_minute_of_day as u64 % MINUTES_PER_DAY
            + elapsed_minutes % MINUTES_PER_DAY)
            % MINUTES_PER_DAY;
        self.day_start().saturating_add_minutes(minute_of_day)
    }

    pub const fn minute_of_day(self) -> u16 {
        (self.minutes % MINUTES_PER_DAY) as u16
    }

    /// Minutes until a clock position later in the current or next day.
    pub const fn minutes_until_time_of_day(self, target: u16) -> Option<u16> {
        if target as u64 > MINUTES_PER_DAY {
            return None;
        }
        let current = self.minute_of_day() as u64;
        Some(((target as u64 + MINUTES_PER_DAY - current) % MINUTES_PER_DAY) as u16)
    }

    /// Whether this instant lies in a half-open daily presence window.
    pub const fn contains_daily_window(self, start: u16, end: u16) -> bool {
        if start as u64 > MINUTES_PER_DAY || end as u64 > MINUTES_PER_DAY || start == end {
            return false;
        }
        let minute = self.minute_of_day();
        if start < end {
            start <= minute && minute < end
        } else {
            minute >= start || minute < end
        }
    }

    /// Remaining elapsed minutes in a daily window, including overnight.
    pub const fn remaining_daily_window_minutes(self, start: u16, end: u16) -> Option<u64> {
        if !self.contains_daily_window(start, end) {
            return None;
        }
        let minute = self.minute_of_day() as u64;
        let end = end as u64;
        if (start as u64) < end {
            Some(end - minute)
        } else if minute >= start as u64 {
            Some(MINUTES_PER_DAY - minute + end)
        } else {
            Some(end - minute)
        }
    }

    /// One-based day since the epoch, followed by clock hour and minute.
    pub const fn day_hour_minute(self) -> (u64, u64, u64) {
        let minute_of_day = self.minute_of_day() as u64;
        (
            self.day_index().get() + 1,
            minute_of_day / 60,
            minute_of_day % 60,
        )
    }

    /// Start of the fixed-width interval containing this minute.
    pub const fn floor_to_interval_minutes(self, interval_minutes: u64) -> Option<Self> {
        match self.minutes.checked_div(interval_minutes) {
            Some(period) => Some(Self::new(period * interval_minutes)),
            None => None,
        }
    }

    /// One-based day within the non-leap calendar year.
    pub const fn day_of_year(self) -> u16 {
        ((self.minutes / MINUTES_PER_DAY) % DAYS_PER_YEAR + 1) as u16
    }

    /// Zero-based calendar day, including its fractional time of day.
    pub fn day_of_year_fraction(self) -> f64 {
        (self.minutes % MINUTES_PER_YEAR) as f64 / MINUTES_PER_DAY as f64
    }

    /// Calendar year containing this minute, capped at the representable year.
    pub const fn calendar_year(self) -> CalendarYear {
        let elapsed_years = self.minutes / MINUTES_PER_YEAR;
        let year = if elapsed_years > (i32::MAX - WORLD_START_YEAR.year) as u64 {
            i32::MAX
        } else {
            WORLD_START_YEAR.year + elapsed_years as i32
        };
        CalendarYear { year }
    }

    pub const fn birth_year_for_age(self, age_years: u16) -> Option<CalendarYear> {
        self.calendar_year().birth_year_for_age(age_years)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "spacetimedb")]
    #[test]
    fn database_year_decoding_rejects_year_zero_and_preserves_integer_bytes() {
        use spacetimedb_lib::bsatn;
        for year in [i32::MIN, -1, 0] {
            let bytes = year.to_le_bytes();
            assert!(bsatn::from_slice::<super::CalendarYear>(&bytes).is_err());
            let mut input = bytes.as_slice();
            assert!(
                <super::CalendarYear as spacetimedb_lib::de::Deserialize>::validate(
                    bsatn::Deserializer::new(&mut input)
                )
                .is_err()
            );
        }
        for year in [1, 1544, i32::MAX] {
            let value = super::CalendarYear::new(year).unwrap();
            let bytes = bsatn::to_vec(&value).unwrap();
            assert_eq!(bytes, year.to_le_bytes());
            assert_eq!(
                bsatn::from_slice::<super::CalendarYear>(&bytes).unwrap(),
                value
            );
        }
    }

    use super::*;

    #[test]
    fn epoch_and_year_boundaries() {
        assert_eq!(StrategicMinute::ZERO.calendar_year(), WORLD_START_YEAR);
        assert_eq!(
            StrategicMinute::new(MINUTES_PER_YEAR - 1).calendar_year(),
            WORLD_START_YEAR
        );
        assert_eq!(
            StrategicMinute::new(MINUTES_PER_YEAR).calendar_year().get(),
            1545
        );
        assert_eq!(WORLD_START_MINUTE.get(), 231 * MINUTES_PER_DAY);
        assert_eq!(
            StrategicMinute::new(u64::MAX).calendar_year().get(),
            i32::MAX
        );
        assert_eq!(
            WORLD_START_MINUTE
                .saturating_add_minutes(MINUTES_PER_YEAR)
                .calendar_year()
                .get(),
            1545
        );
        assert_eq!(StrategicMinute::ZERO.elapsed_since(WORLD_START_MINUTE), 0);
        assert_eq!(
            StrategicMinute::new(5).checked_elapsed_since(StrategicMinute::new(3)),
            Some(2)
        );
        assert_eq!(
            StrategicMinute::new(3).checked_elapsed_since(StrategicMinute::new(5)),
            None
        );
        assert_eq!(
            StrategicMinute::ZERO.days_until_ceil(StrategicMinute::ZERO),
            0
        );
        assert_eq!(
            StrategicMinute::ZERO.days_until_ceil(StrategicMinute::new(1)),
            1
        );
        assert_eq!(
            StrategicMinute::new(2).days_until_ceil(StrategicMinute::new(1)),
            0
        );
        assert_eq!(
            StrategicMinute::new(4).midpoint(StrategicMinute::new(7)),
            StrategicMinute::new(5)
        );
        assert_eq!(StrategicMinute::ZERO.day_of_year(), 1);
        assert_eq!(StrategicMinute::new(1_500).day_hour_minute(), (2, 1, 0));
        assert_eq!(
            StrategicMinute::new(1_430).wrapping_day_offset(20),
            StrategicMinute::new(10)
        );
        assert_eq!(
            StrategicMinute::new(3 * MINUTES_PER_DAY + 900).with_wrapped_time_of_day(1_430, 20),
            StrategicMinute::new(3 * MINUTES_PER_DAY + 10)
        );
        assert_eq!(
            StrategicMinute::new(3 * MINUTES_PER_DAY).with_wrapped_time_of_day(1_430, u64::MAX),
            StrategicMinute::new(3 * MINUTES_PER_DAY + 725)
        );
        assert_eq!(
            StrategicMinute::new(359).floor_to_interval_minutes(360),
            Some(StrategicMinute::ZERO)
        );
        assert_eq!(StrategicMinute::ZERO.floor_to_interval_minutes(0), None);
        assert_eq!(
            StrategicMinute::new(1_500).offset_within_interval_minutes(MINUTES_PER_DAY),
            Some(60)
        );
        assert_eq!(
            StrategicMinute::new(1_380).minutes_until_time_of_day(60),
            Some(120)
        );
        assert_eq!(
            StrategicMinute::new(60).minutes_until_time_of_day(60),
            Some(0)
        );
        assert_eq!(StrategicMinute::ZERO.minutes_until_time_of_day(1_441), None);
        assert!(StrategicMinute::new(1_380).contains_daily_window(1_200, 120));
        assert!(StrategicMinute::new(60).contains_daily_window(1_200, 120));
        assert!(!StrategicMinute::new(600).contains_daily_window(1_200, 120));
        assert!(!StrategicMinute::new(60).contains_daily_window(1_441, 120));
        assert_eq!(
            StrategicMinute::new(1_380).remaining_daily_window_minutes(1_200, 120),
            Some(180)
        );
        assert_eq!(
            StrategicMinute::new(60).remaining_daily_window_minutes(1_200, 120),
            Some(60)
        );
        assert_eq!(
            StrategicMinute::new(600).remaining_daily_window_minutes(1_200, 120),
            None
        );
        assert_eq!(
            StrategicMinute::new(1_500).offset_within_interval_minutes(0),
            None
        );
        assert_eq!(
            StrategicMinute::new(1_500).cumulative_daily_share_minutes(720),
            750
        );
        assert_eq!(
            StrategicDayIndex::new(2).checked_start(),
            Some(StrategicMinute::new(2 * MINUTES_PER_DAY))
        );
        assert_eq!(StrategicDayIndex::MAX.checked_start(), None);
        assert_eq!(
            StrategicMinute::checked_period_start_for_index(2, 360),
            Some(StrategicMinute::new(720))
        );
        assert_eq!(StrategicMinute::checked_period_start_for_index(2, 0), None);
        assert_eq!(
            StrategicMinute::checked_period_start_for_index(u64::MAX, 360),
            None
        );
        assert_eq!(
            StrategicMinute::new(MINUTES_PER_YEAR - 1).day_of_year(),
            365
        );
        assert_eq!(WORLD_START_MINUTE.day_of_year(), 232);
        assert!(StrategicDayIndex::FIRST.weekday() != StrategicWeekday::Sunday);
        assert!(StrategicDayIndex::new(6).weekday() == StrategicWeekday::Sunday);
        assert!(StrategicDayIndex::new(13).weekday() == StrategicWeekday::Sunday);
        assert_eq!(
            StrategicMinute::ZERO
                .saturating_add_years(16)
                .whole_years_since(StrategicMinute::ZERO),
            16
        );
        assert_eq!(
            StrategicMinute::new(u64::MAX).saturating_add_minutes(1),
            StrategicMinute::new(u64::MAX)
        );
        assert_eq!(
            StrategicMinute::ZERO.saturating_add_days(2),
            StrategicMinute::new(2 * MINUTES_PER_DAY)
        );
        assert_eq!(
            StrategicMinute::new(u64::MAX - 1).saturating_add_days(1),
            StrategicMinute::MAX
        );
        assert_eq!(
            StrategicMinute::ZERO.checked_add_days(1),
            Some(StrategicMinute::new(MINUTES_PER_DAY))
        );
        assert_eq!(StrategicMinute::MAX.checked_add_days(1), None);
        assert_eq!(StrategicMinute::ZERO.checked_add_days(u64::MAX), None);
        assert_eq!(
            StrategicMinute::new(3 * MINUTES_PER_DAY).saturating_sub_days(2),
            StrategicMinute::new(MINUTES_PER_DAY)
        );
        assert_eq!(
            StrategicMinute::ZERO.saturating_sub_days(1),
            StrategicMinute::ZERO
        );
        assert_eq!(StrategicMinute::new(u64::MAX).checked_add_minutes(1), None);
        assert_eq!(
            StrategicMinute::new(4)
                .iter_after_through(StrategicMinute::new(6))
                .collect::<Vec<_>>(),
            vec![StrategicMinute::new(5), StrategicMinute::new(6)]
        );
        assert_eq!(
            StrategicMinute::MAX
                .iter_after_through(StrategicMinute::MAX)
                .count(),
            0
        );
        assert_eq!(
            StrategicMinute::new(4)
                .iter_through(StrategicMinute::new(6))
                .collect::<Vec<_>>(),
            vec![
                StrategicMinute::new(4),
                StrategicMinute::new(5),
                StrategicMinute::new(6)
            ]
        );
        assert_eq!(
            StrategicMinute::MAX
                .iter_through(StrategicMinute::MAX)
                .count(),
            1
        );
        assert_eq!(
            StrategicMinute::MAX
                .iter_through(StrategicMinute::ZERO)
                .count(),
            0
        );
        assert_eq!(
            StrategicMinute::new(u64::MAX - 1).checked_add_minutes(1),
            Some(StrategicMinute::new(u64::MAX))
        );
    }

    #[test]
    fn age_derived_birth_years_and_bounds() {
        assert_eq!(
            StrategicMinute::ZERO.birth_year_for_age(25).unwrap().get(),
            1519
        );
        assert_eq!(CalendarYear::new(1).unwrap().birth_year_for_age(1), None);
        assert_eq!(
            CalendarYear::new(20).unwrap().checked_sub_years(19),
            CalendarYear::new(1)
        );
        assert_eq!(CalendarYear::new(20).unwrap().checked_sub_years(20), None);
        assert_eq!(CalendarYear::new(0), None);
        assert_eq!(CalendarYear::new(-1), None);
        assert_eq!(CalendarYear::new(i32::MAX).unwrap().get(), i32::MAX);
        let first = CalendarYear::new(1500).unwrap();
        let last = CalendarYear::new(1600).unwrap();
        assert_eq!(
            CalendarYear::new(1550)
                .unwrap()
                .interpolation_fraction_between(first, last),
            Some(0.5)
        );
        assert_eq!(first.interpolation_fraction_between(last, first), None);
        assert_eq!(
            CalendarYear::new(1499)
                .unwrap()
                .interpolation_fraction_between(first, last),
            None
        );
        assert_eq!(StrategicMinute::ZERO.age_years_since_signed_birth(1), 0);
        assert_eq!(
            StrategicMinute::ZERO.signed_birth_minute_for_age(25),
            -(25 * MINUTES_PER_YEAR as i64)
        );
        assert_eq!(
            StrategicMinute::MAX.signed_birth_minute_for_age(0),
            i64::MAX
        );
        assert_eq!(
            StrategicMinute::new(MINUTES_PER_YEAR).age_years_since_signed_birth(0),
            1
        );
        assert_eq!(
            StrategicMinute::new(MINUTES_PER_YEAR).age_years_since_signed_birth(-1),
            1
        );
        assert!(!StrategicMinute::ZERO.is_at_or_after_signed_birth(1));
        assert!(StrategicMinute::ZERO.is_at_or_after_signed_birth(-1));
        assert_eq!(
            StrategicMinute::at_signed_birth_age(-1, 1),
            Some(StrategicMinute::new(MINUTES_PER_YEAR - 1))
        );
        assert_eq!(StrategicMinute::at_signed_birth_age(-1, 0), None);
        assert_eq!(
            StrategicMinute::ZERO.elapsed_minutes_since_signed_birth(-1),
            1
        );
        assert_eq!(
            StrategicMinute::new(MINUTES_PER_YEAR / 2)
                .scaled_progress_toward_signed_birth_age(0, 1, 10_000),
            5_000
        );
        assert_eq!(
            StrategicMinute::new(MINUTES_PER_YEAR * 2)
                .scaled_progress_toward_signed_birth_age(0, 1, 10_000),
            10_000
        );
    }

    #[test]
    fn serialization_validates_year_and_preserves_minute_units() {
        let year = CalendarYear::new(1544).unwrap();
        assert_eq!(serde_json::to_string(&year).unwrap(), "1544");
        assert_eq!(serde_json::from_str::<CalendarYear>("1544").unwrap(), year);
        assert!(serde_json::from_str::<CalendarYear>("0").is_err());
        assert!(serde_json::from_str::<CalendarYear>("-1").is_err());
        assert!(serde_json::from_str::<CalendarYear>("1544.0").is_err());
        let minute = StrategicMinute::new(MINUTES_PER_YEAR);
        assert_eq!(
            serde_json::to_string(&minute).unwrap(),
            MINUTES_PER_YEAR.to_string()
        );
        assert_eq!(
            serde_json::from_str::<StrategicMinute>(&MINUTES_PER_YEAR.to_string()).unwrap(),
            minute
        );
        assert!(serde_json::from_str::<StrategicMinute>("-1").is_err());
    }
}
