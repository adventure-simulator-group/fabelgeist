//! Native authority coordinates and checked timer-duration refinement.

use adventuresim_core::elapsed::{AnimationElapsed, ElapsedResult};
use bevy::prelude::Time;
use std::{ops::Add, time::Duration};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CombatInstant(Duration);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CombatDuration(Duration);

impl CombatInstant {
    #[cfg(test)]
    pub(crate) const fn from_duration(duration: Duration) -> Self {
        Self(duration)
    }

    pub(crate) fn from_elapsed(time: &Time<()>) -> Self {
        Self(time.elapsed())
    }

    pub(super) fn native_nanos(self) -> u128 {
        self.0.as_nanos()
    }

    pub(crate) fn elapsed_since(self, earlier: Self) -> Duration {
        self.0.saturating_sub(earlier.0)
    }
}

impl CombatDuration {
    #[cfg(test)]
    pub(crate) const fn from_duration(duration: Duration) -> Self {
        Self(duration)
    }

    /// Native authority clock refinement, after checked seconds admission.
    pub(crate) fn from_elapsed(elapsed: AnimationElapsed) -> ElapsedResult<Self> {
        Ok(Self(elapsed.native_duration()?))
    }

    pub(crate) fn elapsed(self) -> AnimationElapsed {
        AnimationElapsed::from(self.0)
    }

    pub(crate) fn as_secs_f32(self) -> f32 {
        self.0.as_secs_f32()
    }

    pub(crate) fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl Add<CombatDuration> for CombatInstant {
    type Output = Self;

    fn add(self, rhs: CombatDuration) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}
