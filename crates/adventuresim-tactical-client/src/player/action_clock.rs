//! Native combat seconds and the client prediction sample clock.

use super::*;

#[derive(Clone, Copy)]
pub(super) struct MeleePlaybackTiming {
    pub(super) start: AnimationSampleTick,
    pub(super) contact: AnimationSampleTick,
    pub(super) end: AnimationSampleTick,
}

impl MeleePlaybackTiming {
    pub(super) fn delayed(
        input: AnimationSampleTick,
        delay: f32,
        contact_seconds: f32,
        recovery: f32,
    ) -> Self {
        Self {
            start: input + delay_ticks(delay),
            contact: input + animation_ticks(contact_seconds),
            end: input + animation_ticks(contact_seconds + recovery),
        }
    }
}
pub(super) fn animation_ticks(seconds: f32) -> AnimationSampleSpan {
    AnimationSampleSpan::from(
        (seconds.max(1.0 / locomotion_sample_hz()) * locomotion_sample_hz()).round() as u64,
    )
}

pub(super) fn delay_ticks(seconds: f32) -> AnimationSampleSpan {
    AnimationSampleSpan::from((seconds.max(0.0) * locomotion_sample_hz()).round() as u64)
}

pub(super) fn delayed_melee_contact_seconds(
    authored_windup: f32,
    predicted_arrival: f32,
) -> (f32, f32) {
    let contact = authored_windup.max(predicted_arrival).max(0.0);
    ((contact - authored_windup.max(0.0)).max(0.0), contact)
}

pub(super) fn melee_contact_delay_from_input(
    skeleton: &SkeletonState,
    continuation: bool,
    contact_tick: AnimationSampleTick,
    ordinary_contact_seconds: f32,
) -> f32 {
    if continuation {
        contact_tick
            .saturating_sub(skeleton.locomotion_sample_tick)
            .samples() as f32
            / locomotion_sample_hz()
    } else {
        ordinary_contact_seconds
    }
}
