//! Elapsed strategic time, distinct from an absolute strategic minute.
//!
//! ```compile_fail
//! use adventuresim_world_schema::calendar::{StrategicDuration, StrategicMinute};
//! let elapsed = StrategicDuration::new(60);
//! let at: StrategicMinute = elapsed;
//! ```
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct StrategicDuration(u64);
impl StrategicDuration {
    pub const ZERO: Self = Self(0);
    pub const fn new(minutes: u64) -> Self {
        Self(minutes)
    }
    pub const fn get(self) -> u64 {
        self.0
    }
}
impl std::fmt::Display for StrategicDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
