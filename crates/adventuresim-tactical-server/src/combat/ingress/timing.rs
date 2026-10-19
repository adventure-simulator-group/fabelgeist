//! Authoritative melee continuation and configured seconds-to-sample timing.

use super::*;

pub(super) struct MeleeAnimationTiming {
    pub(super) start: AnimationSampleTick,
    pub(super) contact: AnimationSampleTick,
    pub(super) end: AnimationSampleTick,
}

impl MeleeAnimationTiming {
    pub(super) fn for_attack(
        skeleton: &SkeletonState,
        spec: &AttackSpec,
        input_tick: AnimationSampleTick,
        authored_windup: CombatDuration,
        lunge_delay_seconds: f32,
        recovery: CombatDuration,
    ) -> Self {
        let sequence_start = if spec.continuation {
            skeleton.attack_continuation_tick().unwrap_or(input_tick)
        } else {
            input_tick
        };
        delayed_melee_timing_ticks(
            sequence_start,
            authored_windup,
            lunge_delay_seconds,
            recovery,
        )
    }
}

pub(super) fn animation_tick(time: &Time<()>) -> AnimationSampleTick {
    AnimationSampleTick::from(
        (time.elapsed_secs_f64() * locomotion_sample_rate().hertz() as f64).round() as u64,
    )
}

pub(super) fn delayed_melee_timing_ticks(
    input_tick: AnimationSampleTick,
    authored_windup: CombatDuration,
    lunge_delay_seconds: f32,
    recovery: CombatDuration,
) -> MeleeAnimationTiming {
    let authored_ticks = duration_ticks(authored_windup);
    let arrival_ticks = AnimationSampleSpan::from(
        (lunge_delay_seconds.max(0.0) * locomotion_sample_rate().hertz()).round() as u64,
    );
    let contact = input_tick.saturating_add(authored_ticks.max(arrival_ticks));
    let animation_start = contact.saturating_sub_span(authored_ticks);
    let recovery_end = contact.saturating_add(duration_ticks(recovery));
    MeleeAnimationTiming {
        start: animation_start,
        contact,
        end: recovery_end,
    }
}

pub(super) fn duration_ticks(duration: CombatDuration) -> AnimationSampleSpan {
    AnimationSampleSpan::from(
        (duration.as_secs_f32() * locomotion_sample_rate().hertz())
            .round()
            .max(1.0) as u64,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn lunge_delays_contact_without_stretching_authored_attack_animation() {
        let authored = CombatDuration::from_duration(Duration::from_millis(300));
        let recovery = CombatDuration::from_duration(Duration::from_millis(200));
        let MeleeAnimationTiming {
            start: animation_start,
            contact,
            end: recovery_end,
        } = delayed_melee_timing_ticks(
            adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64),
            authored,
            0.5,
            recovery,
        );
        assert_eq!(animation_start, AnimationSampleTick::from(23u64));
        assert_eq!(contact, AnimationSampleTick::from(42u64));
        assert_eq!(
            contact.saturating_sub(animation_start),
            duration_ticks(authored)
        );
        assert_eq!(
            recovery_end.saturating_sub(contact),
            duration_ticks(recovery)
        );

        let MeleeAnimationTiming {
            start: animation_start,
            contact,
            ..
        } = delayed_melee_timing_ticks(
            adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64),
            authored,
            0.0,
            recovery,
        );
        assert_eq!(animation_start, AnimationSampleTick::from(10u64));
        assert_eq!(
            contact,
            AnimationSampleTick::from(10u64)
                .checked_add(duration_ticks(authored))
                .unwrap()
        );
    }
}
