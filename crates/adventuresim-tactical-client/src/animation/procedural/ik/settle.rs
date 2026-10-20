//! Stop/restart balance capture and terminal contact convergence.

use super::*;
pub(in crate::animation::procedural) use planning::{
    balance_recovery_direction, plan_settle_landing, projected_capture_point, settle_swing_side,
};
mod planning;

// Returning the raised pelvis consumes about 2 cm of the knee's 10 cm frame
// budget. Reserve that motion only for the raised-to-settle handoff; ordinary
// swing and settle targets retain the faster general cap.

pub(in crate::animation::procedural) fn preserve_raised_handoff_targets(
    memory: &mut LegIkMemory,
    raised: RaisedFootworkState,
    rig_origin: Vec3,
    rig_rotation: Quat,
) {
    let retained = raised.step.retained_targets();
    let left = raised
        .left_solve_target
        .or_else(|| retained.map(|targets| targets.0));
    let right = raised
        .right_solve_target
        .or_else(|| retained.map(|targets| targets.1));
    let (Some(left), Some(right)) = (left, right) else {
        return;
    };
    memory.left_foot_world_target = Some(left);
    memory.right_foot_world_target = Some(right);
    memory.left_foot_target = Some(rig_rotation.inverse() * (left - rig_origin));
    memory.right_foot_target = Some(rig_rotation.inverse() * (right - rig_origin));
    memory.left_release_active = true;
    memory.right_release_active = true;
    memory.left_release_target = None;
    memory.right_release_target = None;
}

pub(in crate::animation::procedural) fn terrain_ik_is_required(
    mode: crate::animation::TerrainIkMode,
    settle_active: bool,
    raised_handoff: bool,
) -> bool {
    mode.is_enabled() || settle_active || raised_handoff
}

pub(in crate::animation::procedural) fn advance_settle_state(
    mut settle: LocomotionSettleState,
    delta_seconds: AnimationElapsed,
) -> AnimationValueResult<LocomotionSettleState> {
    settle.elapsed_seconds = settle.elapsed_seconds.checked_add(delta_seconds)?;
    settle.progress = ik_tuning()
        .settle_step_seconds
        .advance_settle(settle.progress, delta_seconds)?;
    Ok(settle)
}

pub(in crate::animation::procedural) fn cancel_settle_for_restart(
    memory: &mut LegIkMemory,
    planar_velocity: Vec3,
) {
    // A stop can be cancelled while its selected support is still airborne.
    // In that case the retained terrain plant is only a future landing goal,
    // not support ownership. Resume gait from the last propagated ankle the
    // player actually saw; otherwise the first restart sample can snap from
    // the clearance follower directly to the stale settle contact.
    for left in [true, false] {
        let acquired = if left {
            memory.left_foot_plant.is_acquired()
        } else {
            memory.right_foot_plant.is_acquired()
        };
        if acquired {
            continue;
        }
        let (rendered_world, rendered_owner) = if left {
            (
                memory.left_last_rendered_world,
                memory.left_last_rendered_owner,
            )
        } else {
            (
                memory.right_last_rendered_world,
                memory.right_last_rendered_owner,
            )
        };
        if left {
            memory.left_foot_plant = FootPlant::Absent;
            memory.left_foot_world_target = rendered_world.or(memory.left_foot_world_target);
            memory.left_foot_target = rendered_owner.or(memory.left_foot_target);
            memory.left_support_weight = Some(UnitInterval::ZERO);
            memory.left_transition_support_weight = Some(UnitInterval::ZERO);
            memory.left_release_active = true;
            memory.left_release_target = None;
        } else {
            memory.right_foot_plant = FootPlant::Absent;
            memory.right_foot_world_target = rendered_world.or(memory.right_foot_world_target);
            memory.right_foot_target = rendered_owner.or(memory.right_foot_target);
            memory.right_support_weight = Some(UnitInterval::ZERO);
            memory.right_transition_support_weight = Some(UnitInterval::ZERO);
            memory.right_release_active = true;
            memory.right_release_target = None;
        }
    }
    memory.settle = None;
    reset_terminal_settle_reach(memory);
    memory.recent_movement_velocity = planar_velocity.with_y(0.0);
}

pub(in crate::animation::procedural) fn reset_terminal_settle_reach(memory: &mut LegIkMemory) {
    memory.terminal_contacts = TerminalContacts::Inactive;
}

pub(in crate::animation::procedural) fn finish_settle_for_idle(memory: &mut LegIkMemory) {
    let terminal_reach_shift = memory.terminal_contacts.shift();
    memory.settle = None;
    reset_terminal_settle_reach(memory);
    // The next sparse authored-idle evaluation restores the uncorrected rig
    // root. Transfer the converged terminal reach offset to ordinary retained
    // pelvis ownership so both frozen contacts remain reachable instead of
    // dropping one support and starting another settle loop.
    memory.pelvis_shift = terminal_reach_shift;
    memory.recent_movement_velocity = Vec3::ZERO;
    // Promote both final solve targets to a stable idle stance. Clearing them
    // here made the next authored-idle evaluation pull a wide settled step
    // half a metre under the body in one frame. Movement restart still releases
    // these plants through the ordinary bounded gait handoff.
    memory.left_foot_plant = FootPlant::retain(memory.left_foot_world_target, true);
    memory.right_foot_plant = FootPlant::retain(memory.right_foot_world_target, true);
    memory.left_planned_contact = None;
    memory.right_planned_contact = None;
    memory.left_planned_contact_start = None;
    memory.right_planned_contact_start = None;
    memory.left_planned_contact_phase_start = None;
    memory.right_planned_contact_phase_start = None;
    memory.left_support_weight = Some(UnitInterval::ONE);
    memory.right_support_weight = Some(UnitInterval::ONE);
    memory.left_transition_support_weight = Some(UnitInterval::ONE);
    memory.right_transition_support_weight = Some(UnitInterval::ONE);
    memory.left_support_exhausted_until_flight = false;
    memory.right_support_exhausted_until_flight = false;
    memory.left_release_active = false;
    memory.right_release_active = false;
    memory.left_release_target = None;
    memory.right_release_target = None;
}

pub(in crate::animation::procedural) fn settle_is_terminal(memory: &LegIkMemory) -> bool {
    memory
        .settle
        .is_some_and(|settle| settle.progress.is_complete())
        && !memory.left_release_active
        && !memory.right_release_active
}

pub(in crate::animation::procedural) fn prepare_terminal_settle_contacts(
    memory: &mut LegIkMemory,
    rig_origin: Vec3,
    rig_rotation: Quat,
    terrain_height_at: impl Fn(ScenePlanPoint) -> Option<SupportElevation>,
) -> bool {
    if !memory.terminal_contacts.is_prepared() {
        // Terminal solve changes from ordinary pelvis ownership to an
        // absolute rig-root correction. Seed it from the correction already
        // visible on the final settle sample; starting from zero restores the
        // authored root for one frame and lifts the whole hierarchy by the
        // accumulated settle drop.
        memory.terminal_contacts.seed(memory.pelvis_shift);
    }
    let left_seed = if memory.terminal_contacts.is_prepared() {
        memory.left_foot_world_target
    } else {
        memory
            .left_last_rendered_world
            .filter(|target| target.is_finite())
            .or(memory.left_foot_world_target)
    };
    let Some(mut left) = left_seed else {
        return false;
    };
    let right_seed = if memory.terminal_contacts.is_prepared() {
        memory.right_foot_world_target
    } else {
        memory
            .right_last_rendered_world
            .filter(|target| target.is_finite())
            .or(memory.right_foot_world_target)
    };
    let Some(mut right) = right_seed else {
        return false;
    };
    let (Some(left_point), Some(right_point)) = (
        ScenePlanPoint::from_metres(left.xz()),
        ScenePlanPoint::from_metres(right.xz()),
    ) else {
        return false;
    };
    let (Some(left_height), Some(right_height)) = (
        terrain_height_at(left_point),
        terrain_height_at(right_point),
    ) else {
        return false;
    };
    left.y = left_height.metres() + measured_ankle_sole_offset_metres().metres();
    right.y = right_height.metres() + measured_ankle_sole_offset_metres().metres();
    memory.left_foot_world_target = Some(left);
    memory.right_foot_world_target = Some(right);
    memory.left_foot_target = Some(rig_rotation.inverse() * (left - rig_origin));
    memory.right_foot_target = Some(rig_rotation.inverse() * (right - rig_origin));
    memory.left_foot_plant = FootPlant::Acquiring(left);
    memory.right_foot_plant = FootPlant::Acquiring(right);
    if let Some(settle) = memory.settle.as_mut() {
        settle.landing_target = if settle.support_foot == LeadFoot::Left {
            right
        } else {
            left
        };
    }
    memory.terminal_contacts.prepare();
    true
}

pub(in crate::animation::procedural) fn terminal_settle_contacts_are_rendered(
    memory: &LegIkMemory,
    terrain_height_at: impl Fn(ScenePlanPoint) -> Option<SupportElevation>,
) -> bool {
    [
        (
            memory.left_last_rendered_world,
            memory.left_foot_world_target,
            memory.left_last_rendered_toe_world,
        ),
        (
            memory.right_last_rendered_world,
            memory.right_foot_world_target,
            memory.right_last_rendered_toe_world,
        ),
    ]
    .into_iter()
    .all(|(rendered, target, toe)| {
        rendered
            .zip(target)
            .zip(toe)
            .is_some_and(|((rendered, target), toe)| {
                let Some(ankle_point) = ScenePlanPoint::from_metres(rendered.xz()) else {
                    return false;
                };
                let Some(toe_point) = ScenePlanPoint::from_metres(toe.xz()) else {
                    return false;
                };
                rendered.xz().distance(target.xz()) <= 0.01
                    && terrain_height_at(ankle_point).is_some_and(|height| {
                        (rendered.y
                            - height.metres()
                            - measured_ankle_sole_offset_metres().metres())
                        .abs()
                            <= 0.01
                    })
                    && terrain_height_at(toe_point).is_some_and(|height| {
                        let clearance = toe.y - height.metres();
                        (-0.01..=0.10).contains(&clearance)
                    })
            })
    })
}

pub(in crate::animation::procedural) fn terminal_contact_solve_ownership(
    terminal_prepared: bool,
    nominal_weight: UnitInterval,
    retained_plant: Option<Vec3>,
) -> (UnitInterval, Option<Vec3>) {
    if terminal_prepared && retained_plant.is_some() {
        (UnitInterval::ONE, retained_plant)
    } else {
        (nominal_weight, retained_plant)
    }
}

pub(in crate::animation::procedural) fn seed_settle_from_rendered_feet(
    memory: &mut LegIkMemory,
    left: Option<Vec3>,
    right: Option<Vec3>,
    rig_origin: Vec3,
    rig_rotation: Quat,
) {
    reset_terminal_settle_reach(memory);
    if let Some(left) = left.filter(|target| target.is_finite()) {
        memory.left_foot_world_target = Some(left);
        memory.left_foot_target = Some(rig_rotation.inverse() * (left - rig_origin));
        memory.left_foot_plant = FootPlant::Absent;
    }
    if let Some(right) = right.filter(|target| target.is_finite()) {
        memory.right_foot_world_target = Some(right);
        memory.right_foot_target = Some(rig_rotation.inverse() * (right - rig_origin));
        memory.right_foot_plant = FootPlant::Absent;
    }
    // Stop capture owns both legs. A gait plan retained from the preceding run
    // is not a valid landing goal for either the stationary support or the
    // balance-recovery swing.
    memory.left_planned_contact = None;
    memory.right_planned_contact = None;
    memory.left_planned_contact_start = None;
    memory.right_planned_contact_start = None;
    memory.left_planned_contact_phase_start = None;
    memory.right_planned_contact_phase_start = None;
}

pub(in crate::animation::procedural) fn settle_visible_foot(
    last_rendered_world: Option<Vec3>,
    current_authored_world: Option<Vec3>,
) -> Option<Vec3> {
    last_rendered_world
        .filter(|target| target.is_finite())
        .or_else(|| current_authored_world.filter(|target| target.is_finite()))
}

pub(in crate::animation::procedural) fn retain_settle_support(
    memory: &mut LegIkMemory,
    support_foot: LeadFoot,
    left: Option<Vec3>,
    right: Option<Vec3>,
    acquired: bool,
) {
    if support_foot == LeadFoot::Left {
        memory.left_foot_plant = FootPlant::retain(left, acquired);
        memory.left_transition_support_weight = Some(UnitInterval::ONE);
    } else {
        memory.right_foot_plant = FootPlant::retain(right, acquired);
        memory.right_transition_support_weight = Some(UnitInterval::ONE);
    }
}

pub(in crate::animation::procedural) fn settle_stance_is_safe(
    projected_com: Vec3,
    left_foot: Option<Vec3>,
    right_foot: Option<Vec3>,
    terrain: &SceneTerrain,
) -> bool {
    let (Some(left), Some(right)) = (left_foot, right_foot) else {
        return false;
    };
    let at_contact = |foot: Vec3| {
        ScenePlanPoint::from_metres(foot.xz())
            .and_then(|point| terrain.height_at(point))
            .is_some_and(|height| sole_is_at_contact(foot.y, height))
    };
    if !at_contact(left) || !at_contact(right) {
        return false;
    }
    let segment = right.xz() - left.xz();
    let progress = if segment.length_squared() > 0.000001 {
        ((projected_com.xz() - left.xz()).dot(segment) / segment.length_squared()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    projected_com.xz().distance(left.xz() + segment * progress) <= 0.18
}

pub(in crate::animation::procedural) fn sole_is_at_contact(
    ankle_y: f32,
    terrain_height: SupportElevation,
) -> bool {
    (ankle_y - terrain_height.metres() - measured_ankle_sole_offset_metres().metres()).abs()
        <= sole_contact_tolerance_metres().metres()
}

pub(in crate::animation::procedural) fn raised_support_is_actual(
    nominal_support: bool,
    ankle_y: f32,
    terrain_height: SupportElevation,
) -> bool {
    nominal_support && sole_is_at_contact(ankle_y, terrain_height)
}
