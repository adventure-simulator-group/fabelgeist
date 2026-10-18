//! Authoritative foot-contact observations, independent of simulation sample ticks.

use derive_more::{Debug, Display, From};
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    From,
    Display,
)]
#[serde(transparent)]
#[debug("{_0:?}")]
pub struct ContactSequence(u64);

impl ContactSequence {
    pub const ZERO: Self = Self(0);

    pub fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    pub fn events_since(self, previous: Self) -> Option<u64> {
        self.0.checked_sub(previous.0)
    }

    /// Observational continuity permits the authoritative counter to wrap.
    pub fn wrapping_distance_from(self, previous: Self) -> u64 {
        self.0.wrapping_sub(previous.0)
    }

    /// Locate an earlier contact in a proven forward coalescing interval.
    pub fn preceding(self, contacts: u64) -> Option<Self> {
        self.0.checked_sub(contacts).map(Self)
    }

    /// Cadence validation requires a non-wrapping immediate successor.
    pub fn is_successor_of(self, previous: Self) -> bool {
        self.events_since(previous) == Some(1)
    }

    /// Scalar encoding for serialization, tracing and deterministic native seeds.
    pub fn encoded(self) -> u64 {
        self.0
    }
}
