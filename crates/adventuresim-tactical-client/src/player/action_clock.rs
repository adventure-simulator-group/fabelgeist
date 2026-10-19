//! Native combat seconds and the client prediction sample clock.

use super::*;

#[derive(Clone, Copy)]
pub(super) struct MeleePlaybackTiming {
    pub(super) start: AnimationSampleTick,
    pub(super) contact: AnimationSampleTick,
    pub(super) end: AnimationSampleTick,
    /// Native seconds consumed by the predicted AttackState timers.
    pub(super) contact_from_input_seconds: f32,
}

impl MeleePlaybackTiming {
    pub(super) fn delayed(
        skeleton: &SkeletonState,
        spec: &AttackSpec,
        input: AnimationSampleTick,
        delay: f32,
        contact_seconds: f32,
        recovery: f32,
    ) -> Self {
        let sequence_start = if spec.continuation {
            skeleton.attack_continuation_tick().unwrap_or(input)
        } else {
            input
        };
        let start = sequence_start + delay_ticks(delay);
        let contact = sequence_start + animation_ticks(contact_seconds);
        let end = sequence_start + animation_ticks(contact_seconds + recovery);
        let contact_from_input_seconds =
            melee_contact_delay_from_input(skeleton, spec.continuation, contact, contact_seconds);
        Self {
            start,
            contact,
            end,
            contact_from_input_seconds,
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

#[cfg(test)]
mod tests {
    use super::melee_contact_delay_from_input;
    use adventuresim_tactical_core::animation::{AnimationSampleTick, SkeletonState};

    #[test]
    fn continuation_cooldown_uses_the_replicated_animation_clock() {
        let skeleton = SkeletonState::default().with_locomotion_sample_tick(
            adventuresim_tactical_core::animation::AnimationSampleTick::from(700u64),
        );

        assert_eq!(
            melee_contact_delay_from_input(&skeleton, true, AnimationSampleTick::from(732u64), 0.3),
            0.5
        );
        assert_eq!(
            melee_contact_delay_from_input(
                &skeleton,
                false,
                AnimationSampleTick::from(732u64),
                0.3
            ),
            0.3
        );
    }
}
