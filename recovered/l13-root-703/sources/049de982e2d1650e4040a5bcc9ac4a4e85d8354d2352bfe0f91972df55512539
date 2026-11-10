//! Tactical iteration-clock coordinates, separate from battle and wall clocks.

use adventuresim_core::elapsed::{AnimationElapsed, ElapsedResult};
use serde::Serialize;

use super::TACTICAL_TICK_SECONDS;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct TacticalElapsed(f32);

impl TacticalElapsed {
    /// Tick counts are promoted and multiplied in the original f32 order. Even
    /// the largest u64 count times this fixed tick remains finite and nonnegative.
    pub(super) fn from_tick_count(ticks: u64) -> Self {
        Self(ticks as f32 * TACTICAL_TICK_SECONDS)
    }

    /// Native seconds for serialization and the report's f64 promotion boundary.
    pub fn seconds(self) -> f32 {
        self.0
    }

    /// Subtract the tactical observations before promoting the interval to f64,
    /// retaining the iteration harness's original rounding and sample order.
    pub fn elapsed_since(self, previous: Self) -> ElapsedResult<AnimationElapsed> {
        AnimationElapsed::from_seconds(self.0 - previous.0)
    }
}
