//! Identity of a sampled authoritative animation observation.

use bevy::prelude::Reflect;
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
    Reflect,
    Serialize,
    Deserialize,
    From,
    Display,
)]
#[serde(transparent)]
#[debug("{_0:?}")]
pub struct AnimationSampleTick(u64);

/// Nonnegative count of animation samples; not an absolute clock coordinate.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Reflect,
    Serialize,
    Deserialize,
    From,
    Display,
)]
#[serde(transparent)]
#[debug("{_0:?}")]
pub struct AnimationSampleSpan(u64);

impl AnimationSampleTick {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u64::MAX);
    /// Checked coordinate advance; unlike deadlines, fixture/required coordinates may fail.
    pub fn checked_add(self, span: AnimationSampleSpan) -> Option<Self> {
        self.0.checked_add(span.0).map(Self)
    }
    pub fn checked_sub_span(self, span: AnimationSampleSpan) -> Option<Self> {
        self.0.checked_sub(span.0).map(Self)
    }
    pub fn saturating_sub_span(self, span: AnimationSampleSpan) -> Self {
        Self(self.0.saturating_sub(span.0))
    }
    /// A deadline never wraps into the past at the end of the clock.
    pub fn saturating_add(self, span: AnimationSampleSpan) -> Self {
        Self(self.0.saturating_add(span.0))
    }
    pub fn saturating_sub(self, previous: Self) -> AnimationSampleSpan {
        AnimationSampleSpan(self.0.saturating_sub(previous.0))
    }
    /// Capture and consecutive-observation clocks intentionally admit MAX → 0.
    pub fn wrapping_next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
    pub fn wrapping_delta(self, previous: Self) -> AnimationSampleSpan {
        AnimationSampleSpan(self.0.wrapping_sub(previous.0))
    }
    pub fn checked_delta(self, previous: Self) -> Option<AnimationSampleSpan> {
        self.0.checked_sub(previous.0).map(AnimationSampleSpan)
    }
    /// Native simulation clock, event/report or deterministic-seed encoding.
    pub fn encoded(self) -> u64 {
        self.0
    }
}

impl AnimationSampleSpan {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    pub fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }
    pub fn saturating_mul(self, count: u64) -> Self {
        Self(self.0.saturating_mul(count))
    }
    pub fn half(self) -> Self {
        Self(self.0 / 2)
    }
    /// Native sample-count arithmetic, diagnostics or duration conversion boundary.
    pub fn samples(self) -> u64 {
        self.0
    }
}
