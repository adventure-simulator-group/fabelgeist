//! Local quickstep animation and actuator transition diagnostics.

use adventuresim_tactical_core::prelude::{
    AnimationSampleTick, CharacterMotionSnapshot, ControlledPlayer, SkeletonAction, SkeletonState,
};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, Default)]
pub(super) struct LocalQuickstepTrace {
    observation: Option<QuickstepTraceObservation>,
}

#[derive(Clone, Copy, Debug)]
struct QuickstepTraceObservation {
    action: SkeletonAction,
    actuator: ObservedQuickstepActuator,
}

#[derive(Clone, Copy, Debug)]
enum ObservedQuickstepActuator {
    Inactive {
        retained_start_tick: AnimationSampleTick,
    },
    Active {
        start_tick: AnimationSampleTick,
    },
}

impl QuickstepTraceObservation {
    fn has_changed_since(self, previous: Self) -> bool {
        (self.action == SkeletonAction::Dodge) != (previous.action == SkeletonAction::Dodge)
            || self.actuator.is_active() != previous.actuator.is_active()
            || self.actuator.start_tick() != previous.actuator.start_tick()
    }
}

impl ObservedQuickstepActuator {
    fn is_active(self) -> bool {
        matches!(self, Self::Active { .. })
    }

    fn start_tick(self) -> AnimationSampleTick {
        match self {
            Self::Inactive {
                retained_start_tick,
            } => retained_start_tick,
            Self::Active { start_tick } => start_tick,
        }
    }
}

pub(super) fn trace_local_quickstep_state(
    mut players: Query<
        (
            Entity,
            &SkeletonState,
            &CharacterMotionSnapshot,
            &Transform,
            &mut LocalQuickstepTrace,
        ),
        With<ControlledPlayer>,
    >,
) {
    for (entity, skeleton, snapshot, transform, mut trace) in &mut players {
        let action = skeleton.action_kind();
        let dodge_action = action == SkeletonAction::Dodge;
        let push = snapshot.quickstep_push;
        let observation = QuickstepTraceObservation {
            action,
            actuator: if push.active {
                ObservedQuickstepActuator::Active {
                    start_tick: push.start_tick,
                }
            } else {
                ObservedQuickstepActuator::Inactive {
                    retained_start_tick: push.start_tick,
                }
            },
        };
        let changed = trace
            .observation
            .is_none_or(|previous| observation.has_changed_since(previous));
        if !changed {
            continue;
        }

        let attack_lunge = snapshot.melee_lunge.is_some_and(|lunge| lunge.quickstep);
        if (dodge_action || attack_lunge) != push.active {
            warn!(
                target: "quickstep_trace",
                ?entity,
                dodge_action,
                action_direction = ?skeleton.action_direction(),
                skeleton_tick = skeleton.locomotion_sample_tick.encoded(),
                push_active = push.active,
                push_start_tick = push.start_tick.encoded(),
                push_direction = ?push.direction,
                acknowledged_input_tick = snapshot.acknowledged_input_tick,
                render_translation = ?transform.translation,
                snapshot_translation = ?snapshot.translation,
                "[quickstep][client-state] animation/actuator discrepancy"
            );
        } else if dodge_action
            || push.active
            || trace.observation.is_some_and(|previous| {
                previous.action == SkeletonAction::Dodge || previous.actuator.is_active()
            })
        {
            info!(
                target: "quickstep_trace",
                ?entity,
                dodge_action,
                action_direction = ?skeleton.action_direction(),
                skeleton_tick = skeleton.locomotion_sample_tick.encoded(),
                push_active = push.active,
                push_start_tick = push.start_tick.encoded(),
                push_direction = ?push.direction,
                acknowledged_input_tick = snapshot.acknowledged_input_tick,
                render_translation = ?transform.translation,
                snapshot_translation = ?snapshot.translation,
                "[quickstep][client-state] transition"
            );
        }

        trace.observation = Some(observation);
    }
}
