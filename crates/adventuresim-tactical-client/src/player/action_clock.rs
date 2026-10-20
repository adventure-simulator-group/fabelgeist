//! Native combat seconds and the client prediction sample clock.

use super::*;
use adventuresim_core::linear_speed::NonnegativeSpeed;
use adventuresim_tactical_core::animation::{
    AnimationElapsed, AnimationValueResult, PositiveAnimationDuration,
};

/// Timing available only to melee surface-range acquisition. Ranged targeting
/// has no movement-assisted timing input.
#[derive(Clone, Copy)]
pub(super) struct MeleeAcquisitionTiming {
    forward_speed: NonnegativeSpeed,
    acceleration: ForwardLungeAcceleration,
    quickstep_duration: PositiveAnimationDuration,
}

#[derive(Clone, Copy)]
pub(super) struct MeleePlaybackTiming {
    pub(super) start: AnimationSampleTick,
    pub(super) contact: AnimationSampleTick,
    pub(super) end: AnimationSampleTick,
    /// Native seconds consumed by the predicted AttackState timers.
    pub(super) contact_from_input_seconds: AnimationElapsed,
}

impl TryFrom<&adventuresim_tactical_core::combat_config::TacticalMovementConfig>
    for MeleeAcquisitionTiming
{
    type Error = adventuresim_tactical_core::animation::AnimationValueError;

    fn try_from(
        movement: &adventuresim_tactical_core::combat_config::TacticalMovementConfig,
    ) -> AnimationValueResult<Self> {
        Ok(Self {
            forward_speed: movement.speeds_metres_per_second.run,
            acceleration: conservative_forward_lunge_acceleration(&movement.motor)?,
            quickstep_duration: movement.maneuvers.quickstep_duration_seconds,
        })
    }
}

impl MeleeAcquisitionTiming {
    /// A reachable melee surface target owns its movement-assisted delay.
    pub(super) fn contact_arrival(
        self,
        surface_measure: f32,
        reach: f32,
        maximum_travel: f32,
    ) -> AnimationValueResult<Option<MeleeLungeArrival>> {
        let closure = (surface_measure - reach).max(0.0);
        if closure > maximum_travel + 1.0e-5 {
            return Ok(None);
        }
        let arrival = melee_lunge_arrival(
            melee_lunge(surface_measure, reach, 0.0, maximum_travel),
            self.forward_speed,
            self.acceleration,
            self.quickstep_duration,
        )?;
        Ok(match arrival {
            MeleeLungeArrival::CannotArrive => None,
            admitted => Some(admitted),
        })
    }
}

impl MeleePlaybackTiming {
    pub(super) fn delayed(
        skeleton: &SkeletonState,
        spec: &AttackSpec,
        input: AnimationSampleTick,
        delay: AnimationElapsed,
        contact_seconds: AnimationElapsed,
        recovery: AnimationElapsed,
    ) -> AnimationValueResult<Self> {
        let sequence_start = if spec.continuation {
            skeleton.attack_continuation_tick().unwrap_or(input)
        } else {
            input
        };
        let start = sequence_start.saturating_add(delay_ticks(delay)?);
        let contact = sequence_start.saturating_add(animation_ticks(contact_seconds)?);
        let end =
            sequence_start.saturating_add(animation_ticks(contact_seconds.checked_add(recovery)?)?);
        let contact_from_input_seconds =
            melee_contact_delay_from_input(skeleton, spec.continuation, contact, contact_seconds)?;
        Ok(Self {
            start,
            contact,
            end,
            contact_from_input_seconds,
        })
    }
}
pub(super) fn on_attack_fired_hook(
    event: On<Fire<Attack>>,
    mut cmd: Commands,
    mut q_character: Query<(Has<AttackState>, &mut SkeletonState)>,
    viewer: TacticalPlayerViewer,
    time: Res<Time>,
    combat_config: Res<TacticalCombatConfig>,
    targeting: CombatTargeting,
) {
    if let Err(error) = try_start_attack(
        AttackStartRequest {
            entity: event.context,
            alternate_attack: false,
            hand: AttackHand::Main,
        },
        &mut cmd,
        &mut q_character,
        &viewer,
        &time,
        &combat_config,
        &targeting,
    ) {
        warn!(%error, "cannot schedule attack");
    }
}

pub(super) fn animation_ticks(
    seconds: AnimationElapsed,
) -> AnimationValueResult<AnimationSampleSpan> {
    let samples = seconds
        .seconds()
        .max(locomotion_sample_rate().interval().seconds())
        * locomotion_sample_rate().hertz();
    rounded_samples(samples)
}

pub(super) fn delay_ticks(seconds: AnimationElapsed) -> AnimationValueResult<AnimationSampleSpan> {
    rounded_samples(seconds.seconds().max(0.0) * locomotion_sample_rate().hertz())
}

fn rounded_samples(samples: f32) -> AnimationValueResult<AnimationSampleSpan> {
    if !samples.is_finite() {
        return Err(
            adventuresim_tactical_core::animation::AnimationValueError::Overflow {
                quantity: adventuresim_tactical_core::animation::AnimationQuantity::Elapsed,
            },
        );
    }
    // Finite values retain Rust's saturating integer conversion at this native
    // sample-count boundary; overflow of the seconds product is not a deadline.
    Ok(AnimationSampleSpan::from(samples.round() as u64))
}

pub(super) fn melee_contact_delay_from_input(
    skeleton: &SkeletonState,
    continuation: bool,
    contact_tick: AnimationSampleTick,
    ordinary_contact_seconds: AnimationElapsed,
) -> AnimationValueResult<AnimationElapsed> {
    if continuation {
        Ok(AnimationElapsed::from_seconds(
            contact_tick
                .saturating_sub(skeleton.locomotion_sample_tick)
                .samples() as f32
                / locomotion_sample_rate().hertz(),
        )?)
    } else {
        Ok(ordinary_contact_seconds)
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
            melee_contact_delay_from_input(
                &skeleton,
                true,
                AnimationSampleTick::from(732u64),
                adventuresim_core::elapsed::AnimationElapsed::from_seconds(0.3).unwrap()
            )
            .unwrap()
            .seconds(),
            0.5
        );
        assert_eq!(
            melee_contact_delay_from_input(
                &skeleton,
                false,
                AnimationSampleTick::from(732u64),
                adventuresim_core::elapsed::AnimationElapsed::from_seconds(0.3).unwrap()
            )
            .unwrap()
            .seconds(),
            0.3
        );
    }
}
