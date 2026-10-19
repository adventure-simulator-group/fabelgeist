//! Action timing coordinates and nonnegative phase intervals.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(super) struct ActionTimeline {
    pub(super) start_tick: AnimationSampleTick,
    pub(super) preparation_ticks: AnimationSampleSpan,
    pub(super) recovery_ticks: AnimationSampleSpan,
    pub(super) phase: f32,
}

impl ActionTimeline {
    pub(super) fn new(start_tick: AnimationSampleTick, contact_tick: AnimationSampleTick) -> Self {
        let preparation_ticks = contact_tick
            .saturating_sub(start_tick)
            .max(AnimationSampleSpan::ONE);
        Self {
            start_tick,
            preparation_ticks,
            recovery_ticks: preparation_ticks,
            phase: 0.0,
        }
    }

    pub(super) fn with_recovery(
        start_tick: AnimationSampleTick,
        contact_tick: AnimationSampleTick,
        end_tick: AnimationSampleTick,
    ) -> Self {
        Self {
            start_tick,
            preparation_ticks: contact_tick
                .saturating_sub(start_tick)
                .max(AnimationSampleSpan::ONE),
            recovery_ticks: end_tick
                .saturating_sub(contact_tick)
                .max(AnimationSampleSpan::ONE),
            phase: 0.0,
        }
    }

    pub(super) fn normalized(mut self) -> Self {
        self.preparation_ticks = self.preparation_ticks.max(AnimationSampleSpan::ONE);
        self.recovery_ticks = self.recovery_ticks.max(AnimationSampleSpan::ONE);
        self.phase = if self.phase.is_finite() {
            self.phase.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self
    }

    pub(super) fn contact_tick(self) -> AnimationSampleTick {
        self.start_tick.saturating_add(self.preparation_ticks)
    }

    pub(super) fn end_tick(self) -> AnimationSampleTick {
        self.contact_tick().saturating_add(self.recovery_ticks)
    }
}
