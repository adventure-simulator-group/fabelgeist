//! Animation frequencies and checked reciprocal sampling intervals.

use super::{
    AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult,
    PositiveAnimationDuration,
};
use serde::{Deserialize, Serialize};

/// Authored frequency and its once-checked reciprocal sampling interval.
/// The original frequency is retained so scalar serialization preserves its bits.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct AnimationSampleRate {
    frequency: AnimationFrequency,
    interval: PositiveAnimationDuration,
}

/// A finite positive reference multiplier in samples per second. Calibration
/// does not require a representable reciprocal sampling period.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
pub struct AnimationFrequency(f32);

impl AnimationFrequency {
    /// Project at actual per-reference-sample multiplication or native math.
    pub fn hertz(self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for AnimationFrequency {
    type Error = AnimationValueError;
    fn try_from(hertz: f32) -> AnimationValueResult<Self> {
        let quantity = AnimationQuantity::SampleRate;
        if !hertz.is_finite() {
            return Err(AnimationValueError::NonFinite { quantity });
        }
        if hertz <= 0.0 {
            return Err(AnimationValueError::NonPositive { quantity });
        }
        Ok(Self(hertz))
    }
}

impl AnimationSampleRate {
    /// Samples per second at multiplication, ratio or framework clock boundaries.
    pub fn hertz(self) -> f32 {
        self.frequency.hertz()
    }
    /// Native reciprocal duration for fixed-step sampling.
    pub fn interval(self) -> PositiveAnimationDuration {
        self.interval
    }
}

impl TryFrom<f32> for AnimationSampleRate {
    type Error = AnimationValueError;
    fn try_from(hertz: f32) -> AnimationValueResult<Self> {
        let frequency = AnimationFrequency::try_from(hertz)?;
        let quantity = AnimationQuantity::SampleRate;
        let interval = PositiveAnimationDuration::from_elapsed(
            AnimationElapsed::from_seconds(hertz.recip())
                .map_err(|error| AnimationValueError::from(error).for_quantity(quantity))?,
        )?;
        Ok(Self {
            frequency,
            interval,
        })
    }
}

impl From<AnimationSampleRate> for f32 {
    fn from(rate: AnimationSampleRate) -> Self {
        rate.frequency.hertz()
    }
}
