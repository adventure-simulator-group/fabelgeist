//! One calendar rate for shared world time and personal live-play chronology.
//! Fractions survive interval boundaries rather than rounding each receipt.

use adventuresim_world_schema::calendar::{MINUTES_PER_YEAR, StrategicMinute, WORLD_START_MINUTE};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const REAL_SECONDS_PER_STRATEGIC_YEAR: u64 = 7 * 24 * 60 * 60;
const REAL_MICROSECONDS_PER_STRATEGIC_YEAR: u128 =
    REAL_SECONDS_PER_STRATEGIC_YEAR as u128 * 1_000_000;
const REAL_NANOSECONDS_PER_STRATEGIC_YEAR: u64 = REAL_SECONDS_PER_STRATEGIC_YEAR * 1_000_000_000;

pub type ClockResult<Value> = Result<Value, ClockError>;

/// The unspent numerator of one calendar minute. The denominator is the real
/// nanoseconds in a strategic year, after scaling elapsed time by game minutes
/// per year. This integer representation retains even sub-microsecond frames.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
#[serde(try_from = "u64", into = "u64")]
pub struct CalendarMinuteFraction {
    numerator: u64,
}

/// An actor's personal chronology. Whole-minute journey time can change its
/// minute without discarding the fraction earned during preceding local play.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
#[serde(deny_unknown_fields)]
pub struct PersonalClock {
    minute: StrategicMinute,
    fraction: CalendarMinuteFraction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ClockError {
    #[error("Calendar minute fraction must be smaller than one minute")]
    InvalidMinuteFraction,
    #[error("Personal time would exceed its representable calendar minute")]
    PersonalTimeOverflow,
}

impl CalendarMinuteFraction {
    pub const ZERO: Self = Self { numerator: 0 };
}

impl TryFrom<u64> for CalendarMinuteFraction {
    type Error = ClockError;

    fn try_from(numerator: u64) -> ClockResult<Self> {
        if numerator >= REAL_NANOSECONDS_PER_STRATEGIC_YEAR {
            return Err(ClockError::InvalidMinuteFraction);
        }
        Ok(Self { numerator })
    }
}

impl From<CalendarMinuteFraction> for u64 {
    fn from(fraction: CalendarMinuteFraction) -> Self {
        fraction.numerator
    }
}

impl PersonalClock {
    pub const fn new(minute: StrategicMinute, fraction: CalendarMinuteFraction) -> Self {
        Self { minute, fraction }
    }

    pub const fn minute(self) -> StrategicMinute {
        self.minute
    }

    pub const fn fraction(self) -> CalendarMinuteFraction {
        self.fraction
    }

    /// Advance at exactly one game year per real week. Only elapsed local play
    /// belongs here; journey durations remain whole calendar-minute additions.
    pub fn advance_real_time(self, elapsed: Duration) -> ClockResult<Self> {
        let numerator = elapsed
            .as_nanos()
            .checked_mul(u128::from(MINUTES_PER_YEAR))
            .and_then(|value| value.checked_add(u128::from(self.fraction.numerator)))
            .ok_or(ClockError::PersonalTimeOverflow)?;
        let denominator = u128::from(REAL_NANOSECONDS_PER_STRATEGIC_YEAR);
        let minutes =
            u64::try_from(numerator / denominator).map_err(|_| ClockError::PersonalTimeOverflow)?;
        let minute = self
            .minute
            .checked_add_minutes(minutes)
            .ok_or(ClockError::PersonalTimeOverflow)?;
        let remainder =
            u64::try_from(numerator % denominator).map_err(|_| ClockError::PersonalTimeOverflow)?;
        Ok(Self::new(
            minute,
            CalendarMinuteFraction::try_from(remainder)?,
        ))
    }
}

/// Convert real elapsed time to authoritative strategic minutes.
pub fn elapsed_official_minutes(epoch_micros: i64, now_micros: i64) -> u64 {
    let elapsed_micros = now_micros.saturating_sub(epoch_micros).max(0) as u128;
    (elapsed_micros.saturating_mul(u128::from(MINUTES_PER_YEAR))
        / REAL_MICROSECONDS_PER_STRATEGIC_YEAR) as u64
}

/// Convert an official-minute duration to real microseconds, rounding up to
/// preserve the requested tick at the wall-clock timestamp boundary.
pub fn real_micros_for_official_minutes(elapsed_minutes: u64) -> u128 {
    (u128::from(elapsed_minutes) * REAL_MICROSECONDS_PER_STRATEGIC_YEAR)
        .div_ceil(u128::from(MINUTES_PER_YEAR))
}

/// Convert wall-clock timestamps to the shared absolute strategic calendar.
pub fn official_minute(epoch_micros: i64, now_micros: i64) -> StrategicMinute {
    WORLD_START_MINUTE.saturating_add_minutes(elapsed_official_minutes(epoch_micros, now_micros))
}

/// Anchor an isolated developer world's epoch at an exact official minute.
pub fn epoch_micros_for_official_minute(
    now_micros: i64,
    target_minute: StrategicMinute,
) -> Option<i64> {
    let elapsed_minutes = target_minute.elapsed_since(WORLD_START_MINUTE);
    let elapsed_micros = real_micros_for_official_minutes(elapsed_minutes);
    now_micros.checked_sub(i64::try_from(elapsed_micros).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_world_schema::calendar::WORLD_START_DAY_OF_YEAR;

    #[test]
    fn one_real_week_is_one_game_year_for_both_clocks() {
        let one_week_micros = i64::try_from(REAL_MICROSECONDS_PER_STRATEGIC_YEAR).unwrap();
        assert_eq!(
            elapsed_official_minutes(0, one_week_micros),
            MINUTES_PER_YEAR
        );
        let personal = PersonalClock::new(WORLD_START_MINUTE, CalendarMinuteFraction::ZERO)
            .advance_real_time(Duration::from_secs(REAL_SECONDS_PER_STRATEGIC_YEAR))
            .unwrap();
        assert_eq!(personal.minute(), official_minute(0, one_week_micros));
        assert_eq!(personal.fraction(), CalendarMinuteFraction::ZERO);
    }

    #[test]
    fn elapsed_play_is_invariant_under_frame_and_receipt_chunking() {
        let initial = PersonalClock::new(WORLD_START_MINUTE, CalendarMinuteFraction::ZERO);
        let frame = Duration::from_nanos(33_333_333);
        let frames = 30_001;
        let whole = initial.advance_real_time(frame * frames).unwrap();
        let mut chunked = initial;
        for _ in 0..frames {
            chunked = chunked.advance_real_time(frame).unwrap();
        }
        assert_eq!(chunked, whole);
        let document = serde_json::to_string(&chunked).unwrap();
        let restored: PersonalClock = serde_json::from_str(&document).unwrap();
        assert_eq!(restored, whole);
    }

    #[test]
    fn local_journey_local_preserves_unspent_fraction_and_travel_duration() {
        let initial = PersonalClock::new(WORLD_START_MINUTE, CalendarMinuteFraction::ZERO);
        let before = Duration::from_micros(456_789);
        let after = Duration::from_nanos(987_654_321);
        let departure = initial.advance_real_time(before).unwrap();
        assert_ne!(departure.fraction(), CalendarMinuteFraction::ZERO);
        let arrival = PersonalClock::new(
            departure.minute().checked_add_days(3).unwrap(),
            departure.fraction(),
        );
        let resumed = arrival.advance_real_time(after).unwrap();
        let uninterrupted = initial.advance_real_time(before + after).unwrap();
        assert_eq!(resumed.fraction(), uninterrupted.fraction());
        assert_eq!(
            resumed.minute(),
            uninterrupted.minute().checked_add_days(3).unwrap()
        );
    }

    #[test]
    fn invalid_fraction_and_personal_overflow_are_rejected() {
        assert_eq!(
            CalendarMinuteFraction::try_from(REAL_NANOSECONDS_PER_STRATEGIC_YEAR),
            Err(ClockError::InvalidMinuteFraction)
        );
        assert!(
            serde_json::from_str::<CalendarMinuteFraction>(
                &REAL_NANOSECONDS_PER_STRATEGIC_YEAR.to_string()
            )
            .is_err()
        );
        let clock = PersonalClock::new(StrategicMinute::MAX, CalendarMinuteFraction::ZERO);
        assert_eq!(
            clock.advance_real_time(Duration::from_secs(2)),
            Err(ClockError::PersonalTimeOverflow)
        );
    }

    #[test]
    fn future_epoch_has_no_elapsed_official_minutes() {
        assert_eq!(elapsed_official_minutes(2_000_000, 1_000_000), 0);
    }

    #[test]
    fn initialized_world_starts_on_august_twentieth() {
        const DAYS_BEFORE_AUGUST: u64 = 31 + 28 + 31 + 30 + 31 + 30 + 31;
        assert_eq!(WORLD_START_DAY_OF_YEAR, DAYS_BEFORE_AUGUST + 19);
        assert_eq!(official_minute(42, 42), WORLD_START_MINUTE);
        assert_eq!(official_minute(42, 42).calendar_year().get(), 1544);
    }

    #[test]
    fn developer_epoch_round_trips_exact_official_minutes() {
        let now = 2_000_000_000_000_000_i64;
        for target in [
            WORLD_START_MINUTE,
            WORLD_START_MINUTE.saturating_add_minutes(1),
            WORLD_START_MINUTE.saturating_add_days(1),
            WORLD_START_MINUTE.saturating_add_years(16),
        ] {
            let epoch = epoch_micros_for_official_minute(now, target).unwrap();
            assert_eq!(official_minute(epoch, now), target);
        }
    }
}
