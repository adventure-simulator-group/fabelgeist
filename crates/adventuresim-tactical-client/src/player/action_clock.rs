//! Native combat seconds and the client prediction sample clock.

use super::*;
use adventuresim_core::linear_speed::NonnegativeSpeed;
use adventuresim_tactical_core::animation::PositiveAnimationDuration;

/// Timing available only to melee surface-range acquisition. Ranged targeting
/// has no movement-assisted timing input.
#[derive(Clone, Copy)]
pub(super) struct MeleeAcquisitionTiming {
    forward_speed: NonnegativeSpeed,
    acceleration: f32,
    quickstep_duration: PositiveAnimationDuration,
}

#[derive(Clone, Copy)]
pub(super) struct MeleePlaybackTiming {
    pub(super) start: AnimationSampleTick,
    pub(super) contact: AnimationSampleTick,
    pub(super) end: AnimationSampleTick,
    /// Native seconds consumed by the predicted AttackState timers.
    pub(super) contact_from_input_seconds: f32,
}

impl From<&adventuresim_tactical_core::combat_config::TacticalMovementConfig>
    for MeleeAcquisitionTiming
{
    fn from(movement: &adventuresim_tactical_core::combat_config::TacticalMovementConfig) -> Self {
        Self {
            forward_speed: movement.speeds_metres_per_second.run,
            acceleration: conservative_forward_lunge_acceleration(&movement.motor),
            quickstep_duration: movement.maneuvers.quickstep_duration_seconds,
        }
    }
}

impl MeleeAcquisitionTiming {
    /// A reachable melee surface target owns its movement-assisted delay.
    pub(super) fn contact_delay(
        self,
        surface_measure: f32,
        reach: f32,
        maximum_travel: f32,
    ) -> Option<f32> {
        let closure = (surface_measure - reach).max(0.0);
        (closure <= maximum_travel + 1.0e-5).then(|| {
            let lunge = melee_lunge(surface_measure, reach, 0.0, maximum_travel);
            melee_lunge_delay_seconds(
                lunge,
                self.forward_speed,
                self.acceleration,
                self.quickstep_duration,
            )
        })
    }
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
        let start = sequence_start.saturating_add(delay_ticks(delay));
        let contact = sequence_start.saturating_add(animation_ticks(contact_seconds));
        let end = sequence_start.saturating_add(animation_ticks(contact_seconds + recovery));
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
pub(super) fn on_attack_fired_hook(
    event: On<Fire<Attack>>,
    mut cmd: Commands,
    mut q_character: Query<(Has<AttackState>, &mut SkeletonState)>,
    viewer: TacticalPlayerViewer,
    time: Res<Time>,
    combat_config: Res<TacticalCombatConfig>,
    targeting: CombatTargeting,
) {
    try_start_attack(
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
    );
}

pub(super) fn animation_ticks(seconds: f32) -> AnimationSampleSpan {
    AnimationSampleSpan::from(
        (seconds.max(locomotion_sample_rate().interval().seconds())
            * locomotion_sample_rate().hertz())
        .round() as u64,
    )
}

pub(super) fn delay_ticks(seconds: f32) -> AnimationSampleSpan {
    AnimationSampleSpan::from((seconds.max(0.0) * locomotion_sample_rate().hertz()).round() as u64)
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
            / locomotion_sample_rate().hertz()
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
