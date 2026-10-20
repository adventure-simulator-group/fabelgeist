//! Authored maneuver durations and their get-up sample-clock conversion.

use crate::animation::{AnimationSampleRate, PositiveAnimationDuration};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManeuverTimingConfig {
    pub get_up_seconds: PositiveAnimationDuration,
    pub roll_seconds: f32,
    pub dive_seconds: f32,
    pub backward_dive_seconds: f32,
    /// Full sprint-slide duration. Its authored midpoint is body contact;
    /// only the second half is subject to slide drag.
    pub slide_seconds: f32,
    /// Complete input-to-recovery quickstep duration. `ActionTimeline` places
    /// semantic contact halfway through this interval.
    pub quickstep_duration_seconds: PositiveAnimationDuration,
}

impl ManeuverTimingConfig {
    /// Preserve configured get-up seconds -> samples multiplication, rounding,
    /// and native saturating count conversion, with a minimum one-sample pose.
    pub fn get_up_sample_span(
        &self,
        rate: AnimationSampleRate,
    ) -> crate::animation::AnimationSampleSpan {
        crate::animation::AnimationSampleSpan::from(
            (self.get_up_seconds.seconds() * rate.hertz())
                .round()
                .max(1.0) as u64,
        )
    }
}
