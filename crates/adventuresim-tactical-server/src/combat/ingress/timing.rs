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
        arrival: MeleeLungeArrival,
        recovery: CombatDuration,
    ) -> adventuresim_core::elapsed::ElapsedResult<Self> {
        let sequence_start = if spec.continuation {
            skeleton.attack_continuation_tick().unwrap_or(input_tick)
        } else {
            input_tick
        };
        delayed_melee_timing_ticks(sequence_start, authored_windup, arrival, recovery)
    }
    pub(super) fn ranged(
        input: AnimationSampleTick,
        windup: CombatDuration,
        recovery: CombatDuration,
    ) -> adventuresim_core::elapsed::ElapsedResult<Self> {
        let windup = duration_ticks(windup)?;
        let recovery = duration_ticks(recovery)?;
        let contact = input.saturating_add(windup);
        Ok(Self {
            start: input,
            contact,
            end: contact.saturating_add(recovery),
        })
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
    arrival: MeleeLungeArrival,
    recovery: CombatDuration,
) -> adventuresim_core::elapsed::ElapsedResult<MeleeAnimationTiming> {
    let authored_ticks = duration_ticks(authored_windup)?;
    let arrival_ticks = match arrival {
        MeleeLungeArrival::ArrivesIn(elapsed) => rounded_samples(
            elapsed.seconds().max(0.0) * locomotion_sample_rate().hertz(),
            false,
        )?,
        MeleeLungeArrival::NotNeeded | MeleeLungeArrival::CannotArrive => AnimationSampleSpan::ZERO,
    };
    let contact = input_tick.saturating_add(authored_ticks.max(arrival_ticks));
    let animation_start = contact.saturating_sub_span(authored_ticks);
    let recovery_end = contact.saturating_add(duration_ticks(recovery)?);
    Ok(MeleeAnimationTiming {
        start: animation_start,
        contact,
        end: recovery_end,
    })
}

pub(super) fn duration_ticks(
    duration: CombatDuration,
) -> adventuresim_core::elapsed::ElapsedResult<AnimationSampleSpan> {
    rounded_samples(
        duration.as_secs_f32() * locomotion_sample_rate().hertz(),
        true,
    )
}

fn rounded_samples(
    samples: f32,
    minimum_one: bool,
) -> adventuresim_core::elapsed::ElapsedResult<AnimationSampleSpan> {
    if !samples.is_finite() {
        return Err(adventuresim_core::elapsed::ElapsedError::Overflow);
    }
    let rounded = samples.round();
    Ok(AnimationSampleSpan::from(if minimum_one {
        rounded.max(1.0)
    } else {
        rounded
    } as u64))
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
            MeleeLungeArrival::ArrivesIn(
                adventuresim_core::elapsed::AnimationElapsed::from_seconds(0.5).unwrap(),
            ),
            recovery,
        )
        .unwrap();
        assert_eq!(animation_start, AnimationSampleTick::from(23u64));
        assert_eq!(contact, AnimationSampleTick::from(42u64));
        assert_eq!(
            contact.saturating_sub(animation_start),
            duration_ticks(authored).unwrap()
        );
        assert_eq!(
            recovery_end.saturating_sub(contact),
            duration_ticks(recovery).unwrap()
        );

        let MeleeAnimationTiming {
            start: animation_start,
            contact,
            ..
        } = delayed_melee_timing_ticks(
            adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64),
            authored,
            MeleeLungeArrival::NotNeeded,
            recovery,
        )
        .unwrap();
        assert_eq!(animation_start, AnimationSampleTick::from(10u64));
        assert_eq!(
            contact,
            AnimationSampleTick::from(10u64)
                .checked_add(duration_ticks(authored).unwrap())
                .unwrap()
        );
    }
}
