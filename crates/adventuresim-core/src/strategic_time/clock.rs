//! Typed wall-clock coordinates and the shared official strategic-clock rate.
//!
//! Storage adapters admit native Unix microseconds. An official epoch anchors
//! the shared world-start minute; it is distinct from the observed instant.
//!
//! ```compile_fail
//! use adventuresim_core::strategic_time::clock::{OfficialClockEpoch, UnixMicrosecondInstant};
//! let epoch = OfficialClockEpoch::from(0);
//! epoch.at(epoch); // an epoch is not an observed Unix instant
//! ```

use adventuresim_world_schema::calendar::{
    MINUTES_PER_YEAR, StrategicDuration, StrategicMinute, WORLD_START_MINUTE,
};

const REAL_MICROSECONDS_PER_STRATEGIC_YEAR: u128 = 7 * 24 * 60 * 60 * 1_000_000;

/// Signed Unix time at the database and system-clock boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnixMicrosecondInstant(i64);

impl From<i64> for UnixMicrosecondInstant {
    fn from(micros: i64) -> Self {
        Self(micros)
    }
}

impl From<UnixMicrosecondInstant> for i64 {
    fn from(instant: UnixMicrosecondInstant) -> Self {
        instant.0
    }
}

impl UnixMicrosecondInstant {
    /// Preserve signed subtraction saturation before clamping future epochs.
    fn elapsed_since(self, earlier: Self) -> RealMicrosecondDuration {
        RealMicrosecondDuration(self.0.saturating_sub(earlier.0).max(0) as u128)
    }
}

/// Real elapsed time used by the official clock, including wide inverse shifts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RealMicrosecondDuration(u128);

impl From<StrategicDuration> for RealMicrosecondDuration {
    fn from(elapsed: StrategicDuration) -> Self {
        Self(
            (u128::from(elapsed.get()) * REAL_MICROSECONDS_PER_STRATEGIC_YEAR)
                .div_ceil(u128::from(MINUTES_PER_YEAR)),
        )
    }
}

impl From<RealMicrosecondDuration> for u128 {
    fn from(elapsed: RealMicrosecondDuration) -> Self {
        elapsed.0
    }
}

impl RealMicrosecondDuration {
    fn official_elapsed(self) -> StrategicDuration {
        StrategicDuration::new(
            (self.0.saturating_mul(u128::from(MINUTES_PER_YEAR))
                / REAL_MICROSECONDS_PER_STRATEGIC_YEAR) as u64,
        )
    }
}

/// The Unix anchor corresponding to the shared absolute world-start minute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OfficialClockEpoch(UnixMicrosecondInstant);

impl From<i64> for OfficialClockEpoch {
    fn from(micros: i64) -> Self {
        Self(UnixMicrosecondInstant::from(micros))
    }
}

impl From<UnixMicrosecondInstant> for OfficialClockEpoch {
    fn from(instant: UnixMicrosecondInstant) -> Self {
        Self(instant)
    }
}

impl From<OfficialClockEpoch> for i64 {
    fn from(epoch: OfficialClockEpoch) -> Self {
        epoch.0.into()
    }
}

impl OfficialClockEpoch {
    pub fn elapsed_at(self, now: UnixMicrosecondInstant) -> StrategicDuration {
        now.elapsed_since(self.0).official_elapsed()
    }

    pub fn at(self, now: UnixMicrosecondInstant) -> StrategicMinute {
        StrategicMinute::new(
            WORLD_START_MINUTE
                .get()
                .saturating_add(self.elapsed_at(now).get()),
        )
    }

    /// Choose the inverse epoch, rounding up to preserve the requested tick.
    /// Targets before world start retain the existing clamp to world start.
    pub fn for_target_minute(
        now: UnixMicrosecondInstant,
        target: StrategicMinute,
    ) -> Result<Self, ClockEpochShiftError> {
        let elapsed = StrategicDuration::new(target.get().saturating_sub(WORLD_START_MINUTE.get()));
        Self::from(now).advance_by(elapsed)
    }

    /// Move an epoch earlier by the inverse-rate duration. This does not write
    /// any clock or grant authority to advance a database.
    pub fn advance_by(self, elapsed: StrategicDuration) -> Result<Self, ClockEpochShiftError> {
        let real_elapsed = RealMicrosecondDuration::from(elapsed);
        let delta = i64::try_from(real_elapsed.0).map_err(|source| {
            ClockEpochShiftError::DurationOutOfRange {
                epoch: self,
                elapsed,
                real_elapsed,
                source,
            }
        })?;
        let earlier = self
            .0
            .0
            .checked_sub(delta)
            .ok_or(ClockEpochShiftError::EpochUnderflow {
                epoch: self,
                elapsed,
                real_elapsed,
            })?;
        Ok(Self::from(earlier))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockEpochShiftError {
    DurationOutOfRange {
        epoch: OfficialClockEpoch,
        elapsed: StrategicDuration,
        real_elapsed: RealMicrosecondDuration,
        source: std::num::TryFromIntError,
    },
    EpochUnderflow {
        epoch: OfficialClockEpoch,
        elapsed: StrategicDuration,
        real_elapsed: RealMicrosecondDuration,
    },
}

impl std::fmt::Display for ClockEpochShiftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DurationOutOfRange { elapsed, .. } => write!(
                f,
                "official clock shift of {elapsed} minutes exceeds signed microseconds"
            ),
            Self::EpochUnderflow { epoch, elapsed, .. } => write!(
                f,
                "official clock epoch {} underflows after a shift of {elapsed} minutes",
                i64::from(*epoch)
            ),
        }
    }
}

impl std::error::Error for ClockEpochShiftError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DurationOutOfRange { source, .. } => Some(source),
            Self::EpochUnderflow { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn one_real_week_is_one_game_year() {
        let epoch = OfficialClockEpoch::from(0);
        let now = UnixMicrosecondInstant::from(604_800_000_000);
        assert_eq!(
            epoch.elapsed_at(now),
            StrategicDuration::new(MINUTES_PER_YEAR)
        );
        assert_eq!(
            epoch.at(now),
            WORLD_START_MINUTE.saturating_add_minutes(MINUTES_PER_YEAR)
        );
    }

    #[test]
    fn future_epoch_has_no_elapsed_official_minutes() {
        let epoch = OfficialClockEpoch::from(2_000_000);
        let now = UnixMicrosecondInstant::from(1_000_000);
        assert_eq!(epoch.elapsed_at(now), StrategicDuration::ZERO);
        assert_eq!(epoch.at(now), WORLD_START_MINUTE);
    }

    #[test]
    fn initialized_world_starts_on_august_twentieth() {
        use adventuresim_world_schema::calendar::WORLD_START_DAY_OF_YEAR;
        const DAYS_BEFORE_AUGUST: u64 = 31 + 28 + 31 + 30 + 31 + 30 + 31;
        assert_eq!(WORLD_START_DAY_OF_YEAR, DAYS_BEFORE_AUGUST + 19);
        let now = UnixMicrosecondInstant::from(42);
        assert_eq!(OfficialClockEpoch::from(now).at(now), WORLD_START_MINUTE);
        assert_eq!(
            OfficialClockEpoch::from(now).at(now).calendar_year().get(),
            1544
        );
    }

    #[test]
    fn inverse_epochs_preserve_exact_requested_ticks_and_pre_world_clamping() {
        let now = UnixMicrosecondInstant::from(2_000_000_000_000_000);
        for target in [
            WORLD_START_MINUTE,
            WORLD_START_MINUTE.saturating_add_minutes(1),
            WORLD_START_MINUTE.saturating_add_days(1),
            WORLD_START_MINUTE.saturating_add_years(16),
        ] {
            let epoch = OfficialClockEpoch::for_target_minute(now, target).unwrap();
            assert_eq!(epoch.at(now), target);
        }
        let before_world =
            OfficialClockEpoch::for_target_minute(now, StrategicMinute::ZERO).unwrap();
        assert_eq!(i64::from(before_world), i64::from(now));
        assert_eq!(before_world.at(now), WORLD_START_MINUTE);
    }

    #[test]
    fn each_inverse_shift_rounds_up_at_the_first_real_microsecond_of_a_tick() {
        let epoch = OfficialClockEpoch::from(0);
        for minute in [1, 2, 60, 1_440, 525_600] {
            let elapsed = StrategicDuration::new(minute);
            let micros = i64::try_from(u128::from(RealMicrosecondDuration::from(elapsed))).unwrap();
            assert_eq!(
                epoch.elapsed_at(UnixMicrosecondInstant::from(micros)),
                elapsed
            );
            assert_eq!(
                epoch.elapsed_at(UnixMicrosecondInstant::from(micros - 1)),
                StrategicDuration::new(minute - 1)
            );
        }
    }

    #[test]
    fn full_width_signed_elapsed_time_saturates_before_scaling() {
        let earliest = OfficialClockEpoch::from(i64::MIN);
        let latest = UnixMicrosecondInstant::from(i64::MAX);
        assert_eq!(
            earliest.elapsed_at(latest),
            OfficialClockEpoch::from(0).elapsed_at(latest)
        );
        assert_eq!(
            OfficialClockEpoch::from(i64::MAX).elapsed_at(UnixMicrosecondInstant::from(i64::MIN)),
            StrategicDuration::ZERO
        );
    }

    #[test]
    fn inverse_shift_errors_keep_the_failed_coordinates_and_narrowing_cause() {
        let epoch = OfficialClockEpoch::from(i64::MIN);
        let elapsed = StrategicDuration::new(1);
        let error = epoch.advance_by(elapsed).unwrap_err();
        assert!(
            matches!(error, ClockEpochShiftError::EpochUnderflow { epoch: failed, elapsed: requested, .. } if failed==epoch && requested==elapsed)
        );
        assert!(error.source().is_none());
        let elapsed = StrategicDuration::new(u64::MAX);
        let error = OfficialClockEpoch::from(0).advance_by(elapsed).unwrap_err();
        assert!(
            matches!(error, ClockEpochShiftError::DurationOutOfRange { elapsed: requested, real_elapsed, .. } if requested==elapsed && real_elapsed==RealMicrosecondDuration::from(elapsed))
        );
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<std::num::TryFromIntError>()
                .is_some()
        );
    }
}
