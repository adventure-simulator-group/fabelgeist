//! Local quickstep animation and actuator transition diagnostics.

use adventuresim_tactical_core::prelude::{
    AnimationSampleTick, CharacterMotionSnapshot, ControlledPlayer, SkeletonAction, SkeletonState,
};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, Default)]
pub(super) struct LocalQuickstepTrace {
    observation: Option<QuickstepTraceObservation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct QuickstepTraceObservation {
    dodge_action: bool,
    push_active: bool,
    push_start_tick: AnimationSampleTick,
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
        let dodge_action = skeleton.action_kind() == SkeletonAction::Dodge;
        let push = snapshot.quickstep_push;
        let observation = QuickstepTraceObservation {
            dodge_action,
            push_active: push.active,
            push_start_tick: push.start_tick,
        };
        let changed = trace.observation != Some(observation);
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
            || trace
                .observation
                .is_some_and(|previous| previous.dodge_action || previous.push_active)
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
