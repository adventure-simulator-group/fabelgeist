//! Identity of a sampled authoritative animation observation.

use bevy::prelude::Reflect;
use derive_more::{Debug, Display, From};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

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
    pub fn saturating_sub_span(self, span: AnimationSampleSpan) -> Self {
        Self(self.0.saturating_sub(span.0))
    }
    pub fn saturating_add(self, span: AnimationSampleSpan) -> Self {
        Self(self.0.saturating_add(span.0))
    }
    pub fn saturating_sub(self, previous: Self) -> AnimationSampleSpan {
        AnimationSampleSpan(self.0.saturating_sub(previous.0))
    }
    pub fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
    pub fn wrapping_delta(self, previous: Self) -> u64 {
        self.0.wrapping_sub(previous.0)
    }
    pub fn checked_delta(self, previous: Self) -> Option<u64> {
        self.0.checked_sub(previous.0)
    }
    /// Native simulation clock, event/report or deterministic-seed encoding.
    pub fn encoded(self) -> u64 {
        self.0
    }
}

impl AnimationSampleSpan {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);
    /// Native sample-count arithmetic, diagnostics or duration conversion boundary.
    pub fn samples(self) -> u64 {
        self.0
    }
}
impl Add<AnimationSampleSpan> for AnimationSampleTick {
    type Output = Self;
    fn add(self, span: AnimationSampleSpan) -> Self {
        Self(self.0 + span.0)
    }
}

impl Sub<AnimationSampleSpan> for AnimationSampleTick {
    type Output = Self;
    fn sub(self, span: AnimationSampleSpan) -> Self {
        Self(self.0 - span.0)
    }
}

impl Sub<AnimationSampleTick> for AnimationSampleTick {
    type Output = AnimationSampleSpan;
    fn sub(self, previous: Self) -> AnimationSampleSpan {
        AnimationSampleSpan(self.0 - previous.0)
    }
}
