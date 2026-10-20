//! Ordinary and run contact acquisition, release, reach, and clearance policy.

use super::*;
use adventuresim_core::linear_speed::NonnegativeSpeed;
use adventuresim_tactical_core::animation::STATIONARY_ANIMATION_SPEED_LIMIT;
pub(in crate::animation::procedural) use approach::run_contact_approach_progress;
pub(in crate::animation::procedural) use flight_projection::advance_run_airborne_world_target;
pub(in crate::animation::procedural) use run_reach::{
    reachable_run_contact_target, retarget_unacquired_run_contact_for_descent,
    run_contact_within_follower_motion_step, run_contact_within_follower_step,
    run_contact_within_leg_reach,
};
pub(in crate::animation::procedural) use foot_clearance::{
    FootClearanceFloor, FootDescent, checked_foot_target, checked_scene_point_metres,
    conform_run_swing_target, conform_settle_swing_target, ordinary_terrain_target,
    run_airborne_clearance_for_sample,
    settle_swing_target, terrain_clearance_floor, toe_aware_clearance_floor,
    transition_clearance_floor,
};

#[cfg(test)]
pub(in crate::animation::procedural) use foot_clearance::{
    checked_scene_elevation, run_airborne_clearance, run_swing_clearance, transition_toe_clearance_with_rotation_margin,
};

mod approach;
mod flight_projection;
mod foot_clearance;
mod run_reach;

// A two-bone knee can travel slightly more than twice as far as its ankle
// target near extension. Derive the release cap from that conservative bound
// and retain two percent of numerical margin below the viewer's 0.10 m
// contract at 64 Hz.
// Measured vertical distance from the Cascadeur ankle bone to its sole.
// Maximum rendered ankle-to-terrain residual that still represents sole
// contact after the complete analytic and scene-hierarchy solve.
// A late-created plan must not compress a full stride into the few samples
// left before support entry. Keep target motion relative to the advancing body
// below the measured knee-singularity budget; ordinary full-swing plans retain
// their desired footprint because their available budget is larger.

pub(in crate::animation::procedural) fn uses_run_airborne_motion_budget(
    gait: LocomotionGait,
    planar_speed: NonnegativeSpeed,
) -> bool {
    gait == LocomotionGait::Run
        || planar_speed.metres_per_second()
            >= (walk_locomotion_profile()
                .reference_speed
                .metres_per_second()
                + run_locomotion_profile().reference_speed.metres_per_second())
                * 0.5
}

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(in crate::animation::procedural) fn bound_unacquired_run_support_release_target(
    run_budget: bool,
    has_plan: bool,
    acquired: bool,
    release_owned: bool,
    previous_owner_target: Option<Vec3>,
    desired_world_target: Vec3,
    rig_origin: Vec3,
    rig_rotation: Quat,
    delta_seconds: AnimationElapsed,
    target_speed: NonnegativeSpeed,
    clearance_floor_at: impl Fn(ScenePlanPoint) -> AnimationValueResult<Option<FootClearanceFloor>>,
) -> AnimationValueResult<Vec3> {
    if run_budget && !has_plan && !acquired && release_owned {
        advance_run_airborne_world_target(
            previous_owner_target,
            desired_world_target,
            rig_origin,
            rig_rotation,
            delta_seconds,
            target_speed,
            clearance_floor_at,
        )
    } else {
        checked_foot_target(desired_world_target)
    }
}

pub(in crate::animation::procedural) fn support_release_diagnostic_goal(
    release_active: bool,
    bounded_unplanned_release: bool,
    bounded_owner_target: Vec3,
    desired_owner_target: Vec3,
) -> Option<Vec3> {
    release_active.then_some(if bounded_unplanned_release {
        // The terrain-feasible bounded waypoint is the target this release
        // actually owns for the current sample. Reporting the unreachable
        // final contact as a frozen goal makes a necessary uphill projection
        // appear to move away from its owner even though rendered continuity
        // is valid and the waypoint advances monotonically.
        bounded_owner_target
    } else {
        desired_owner_target
    })
}

pub(in crate::animation::procedural) fn resolved_unacquired_support_release_ownership(
    bounded_unplanned_release: bool,
    resolved_end: Vec3,
    rig_origin: Vec3,
    rig_rotation: Quat,
) -> Option<(Vec3, Vec3)> {
    bounded_unplanned_release.then(|| {
        let resolved_owner = rig_rotation.inverse() * (resolved_end - rig_origin);
        (resolved_end, resolved_owner)
    })
}

pub(in crate::animation::procedural) fn airborne_unplanned_release_uses_resolved_end(
    run_airborne_budget: bool,
    planned_contact: Option<Vec3>,
    release_active: bool,
) -> bool {
    run_airborne_budget && planned_contact.is_none() && release_active
}

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(in crate::animation::procedural) fn commit_resolved_unplanned_airborne_release(
    memory: &mut LegIkMemory,
    foot_side: LeadFoot,
    run_airborne_budget: bool,
    planned_contact: Option<Vec3>,
    release_active: bool,
    resolved_end: Vec3,
    rig_origin: Vec3,
    rig_rotation: Quat,
) {
    commit_resolved_unacquired_support_release(
        memory,
        foot_side,
        airborne_unplanned_release_uses_resolved_end(
            run_airborne_budget,
            planned_contact,
            release_active,
        ),
        resolved_end,
        rig_origin,
        rig_rotation,
    );
}

pub(in crate::animation::procedural) fn commit_resolved_unacquired_support_release(
    memory: &mut LegIkMemory,
    foot_side: LeadFoot,
    bounded_unplanned_release: bool,
    resolved_end: Vec3,
    rig_origin: Vec3,
    rig_rotation: Quat,
) {
    let Some((resolved_world, resolved_owner)) = resolved_unacquired_support_release_ownership(
        bounded_unplanned_release,
        resolved_end,
        rig_origin,
        rig_rotation,
    ) else {
        return;
    };
    if foot_side == LeadFoot::Left {
        memory.left_foot_world_target = Some(resolved_world);
        memory.left_foot_target = Some(resolved_owner);
        memory.left_release_target = Some(resolved_owner);
    } else {
        memory.right_foot_world_target = Some(resolved_world);
        memory.right_foot_target = Some(resolved_owner);
        memory.right_release_target = Some(resolved_owner);
    }
}

pub(in crate::animation::procedural) fn update_measured_owner_planar_speed(
    retained_speed: Option<NonnegativeSpeed>,
    previous_origin: Option<Vec3>,
    current_origin: Vec3,
    delta_seconds: AnimationElapsed,
    evaluation_advances: bool,
    owner_discontinuous: bool,
) -> AnimationValueResult<Option<NonnegativeSpeed>> {
    if !evaluation_advances {
        Ok(retained_speed)
    } else if owner_discontinuous || delta_seconds.is_zero() {
        Ok(None)
    } else {
        previous_origin
            .map(|previous| {
                NonnegativeSpeed::from_metres_per_second(
                    current_origin.xz().distance(previous.xz()) / delta_seconds.seconds(),
                )
                .map_err(|_| AnimationValueError::Overflow {
                    quantity: AnimationQuantity::Speed,
                })
            })
            .transpose()
    }
}

pub(in crate::animation::procedural) fn run_is_at_support_exit(
    phase: CyclePhase,
    foot_side: LeadFoot,
    support_radius: CycleDelta,
) -> bool {
    let support_radius = support_radius.cycles();
    let post_contact = phase.since_contact(foot_side).cycles();
    // Release on the first sampled phase beyond the nominal lobe, not on its
    // decaying shoulder. The half-cycle bound distinguishes this foot's
    // post-contact side from its next rising shoulder after wrap.
    post_contact >= support_radius && post_contact < 0.5
}

pub(in crate::animation::procedural) fn exhausted_latch_after_raw_cadence(
    exhausted: bool,
    raw_nominal_support: UnitInterval,
) -> bool {
    // Exhaustion suppresses only the remainder of the current support lobe.
    // Consult the unsuppressed gait cadence here: reported/effective support
    // may be zero precisely because this latch is active and therefore cannot
    // prove that the foot has crossed true flight into its next cycle.
    exhausted && terrain_leg_has_support(raw_nominal_support)
}

pub(in crate::animation::procedural) fn run_plan_is_on_rising_support(
    gait: LocomotionGait,
    phase: CyclePhase,
    foot_side: LeadFoot,
    support_radius: CycleDelta,
    raw_nominal_support: UnitInterval,
    planned_contact: Option<Vec3>,
    acquired: bool,
) -> bool {
    let support_radius = support_radius.cycles();
    gait == LocomotionGait::Run
        && planned_contact.is_some()
        && !acquired
        && terrain_leg_has_support(raw_nominal_support)
        // A rising shoulder approaches this foot's contact center. The
        // post-contact shoulder has almost a complete cycle remaining and
        // must not reopen a just-exhausted lobe.
        && phase.until_contact(foot_side).cycles() <= support_radius + 0.001
}

pub(in crate::animation::procedural) fn acquired_plan_can_clear(acquired: bool) -> bool {
    acquired
}

pub(in crate::animation::procedural) fn clear_planned_contact_metadata(
    contact: &mut Option<Vec3>,
    start: &mut Option<Vec3>,
    phase_start: &mut Option<CycleDelta>,
) {
    *contact = None;
    *start = None;
    *phase_start = None;
}

pub(in crate::animation::procedural) fn clear_all_planned_contact_metadata(
    memory: &mut LegIkMemory,
) {
    clear_planned_contact_metadata(
        &mut memory.left_planned_contact,
        &mut memory.left_planned_contact_start,
        &mut memory.left_planned_contact_phase_start,
    );
    clear_planned_contact_metadata(
        &mut memory.right_planned_contact,
        &mut memory.right_planned_contact_start,
        &mut memory.right_planned_contact_phase_start,
    );
}

pub(in crate::animation::procedural) fn acquisition_lobe_exited_without_contact(
    planned_contact: Option<Vec3>,
    acquired: bool,
    previous_support: Option<UnitInterval>,
    current_support: UnitInterval,
) -> bool {
    planned_contact.is_some()
        && !acquired
        && previous_support.is_some_and(terrain_leg_has_support)
        && !terrain_leg_has_support(current_support)
}

pub(in crate::animation::procedural) fn support_after_exhausted_lobe(
    exhausted: bool,
    nominal_weight: UnitInterval,
) -> (bool, UnitInterval) {
    if !exhausted {
        (false, nominal_weight)
    } else if terrain_leg_has_support(nominal_weight) {
        (true, UnitInterval::ZERO)
    } else {
        (false, nominal_weight)
    }
}

pub(in crate::animation::procedural) fn run_planned_contact_allowed(
    support_lobe_exhausted: bool,
    phase_to_contact: CycleDelta,
    approach_window: CycleDelta,
) -> bool {
    let approach_window = approach_window.cycles();
    !support_lobe_exhausted && phase_to_contact.cycles() <= approach_window
}

pub(in crate::animation::procedural) fn planned_contact_start(
    retained_start: Option<Vec3>,
    prior_visible_target: Option<Vec3>,
    authored_foot: Vec3,
) -> Vec3 {
    retained_start
        .or(prior_visible_target)
        .unwrap_or(authored_foot)
}

pub(in crate::animation::procedural) fn run_previous_owner_target(
    gait: LocomotionGait,
    last_rendered_owner: Option<Vec3>,
    analytic_owner_target: Option<Vec3>,
) -> Option<Vec3> {
    if gait == LocomotionGait::Run {
        // The analytic target can be centimetres ahead of a reach-constrained
        // rendered ankle. Continuity and final acquisition must advance from
        // the pose the player saw, not from that invisible goal.
        last_rendered_owner.or(analytic_owner_target)
    } else {
        analytic_owner_target
    }
}

pub(in crate::animation::procedural) fn run_plan_visible_start(
    gait: LocomotionGait,
    starts_new_plan: bool,
    was_releasing: bool,
    previous_owner_target: Option<Vec3>,
    rig_origin: Vec3,
    rig_rotation: Quat,
    propagated_visible_target: Option<Vec3>,
) -> Option<Vec3> {
    if gait == LocomotionGait::Run && starts_new_plan && was_releasing {
        // Hermite progress is zero on the creation sample. Reusing the prior
        // world ankle therefore holds it still while the controller advances
        // 8.6 cm, forcing the nearly extended knee to rearrange in one frame.
        // Preserve its owner-local position across this release-to-plan seam;
        // the new endpoint remains frozen in world space after the seed.
        previous_owner_target
            .map(|owner| rig_origin + rig_rotation * owner)
            .or(propagated_visible_target)
    } else {
        propagated_visible_target
    }
}

pub(in crate::animation::procedural) fn release_start_owner_target(
    gait: LocomotionGait,
    previous_owner_target: Option<Vec3>,
    previous_world_target: Option<Vec3>,
    rig_origin: Vec3,
    rig_rotation: Quat,
    fallback: Vec3,
) -> Vec3 {
    if gait == LocomotionGait::Run {
        // The first aerial sample follows controller travel in owner space and
        // adds only the clearance floor. Holding the old world plant for this
        // sample adds the full root displacement to the visible foot step.
        previous_owner_target
            .or_else(|| {
                previous_world_target.map(|world| rig_rotation.inverse() * (world - rig_origin))
            })
            .unwrap_or(fallback)
    } else {
        previous_world_target
            .map(|world| rig_rotation.inverse() * (world - rig_origin))
            .or(previous_owner_target)
            .unwrap_or(fallback)
    }
}

pub(in crate::animation::procedural) fn bound_late_run_contact(
    visible_start: Vec3,
    desired_contact: Vec3,
    speed: NonnegativeSpeed,
    step_distance: NonnegativeLength,
    phase_to_contact: CycleDelta,
    contact_ready_phase: CycleDelta,
) -> Vec3 {
    let contact_ready_phase = contact_ready_phase.cycles();
    let remaining_phase = (phase_to_contact.cycles() - contact_ready_phase).max(0.0);
    let root_travel = remaining_phase * step_distance.metres() * 2.0;
    let remaining_seconds = if speed > STATIONARY_ANIMATION_SPEED_LIMIT {
        root_travel / speed.metres_per_second()
    } else {
        0.0
    };
    let relative_travel = ik_tuning()
        .maximum_run_swing_root_relative_step_metres
        .metres()
        * ik_tuning().continuity_sample_hz.hertz()
        * remaining_seconds;
    let maximum_horizontal_travel = root_travel + relative_travel;
    let horizontal = desired_contact.xz() - visible_start.xz();
    let bounded = visible_start.xz() + horizontal.clamp_length_max(maximum_horizontal_travel);
    Vec3::new(bounded.x, desired_contact.y, bounded.y)
}

pub(in crate::animation::procedural) fn late_run_plan_requires_bound(
    retained_contact: Option<Vec3>,
    phase_to_contact: CycleDelta,
) -> bool {
    retained_contact.is_none()
        && phase_to_contact.cycles() < ik_tuning().late_run_contact_plan_phase.fraction()
}

pub(in crate::animation::procedural) fn unplanned_run_support_requires_flight(
    gait: LocomotionGait,
    speed: NonnegativeSpeed,
    nominal_support: UnitInterval,
    acquired: bool,
    planned_contact: Option<Vec3>,
) -> bool {
    gait == LocomotionGait::Run
        && speed > STATIONARY_ANIMATION_SPEED_LIMIT
        && terrain_leg_has_support(nominal_support)
        && !acquired
        && planned_contact.is_none()
}

pub(in crate::animation::procedural) fn run_support_descent(
    gait: LocomotionGait,
    phase: CyclePhase,
    foot_side: LeadFoot,
    support_radius: CycleDelta,
    raw_nominal_support: UnitInterval,
    contact_reachable: bool,
) -> FootDescent {
    let support_radius = support_radius.cycles();
    if gait == LocomotionGait::Run
        && contact_reachable
        && terrain_leg_has_support(raw_nominal_support)
        // Only the rising shoulder approaches this foot's contact center.
        // The symmetric raw weight on the post-contact shoulder belongs to
        // stance/toe-off and must not pull an unacquired next-cycle plan down.
        && phase.until_contact(foot_side).cycles() <= support_radius + 0.001
    {
        FootDescent::FollowFloor
    } else {
        FootDescent::RaiseOnly
    }
}

pub(in crate::animation::procedural) fn ordinary_contact_target(
    rig_origin: Vec3,
    rig_rotation: Quat,
    projected_com: Vec3,
    velocity: Vec3,
    step_distance: NonnegativeLength,
    phase_to_contact: CycleDelta,
    side: FootTrackSide,
) -> Vec3 {
    let direction = velocity
        .with_y(0.0)
        .try_normalize()
        .unwrap_or(rig_rotation * Vec3::NEG_Z);
    // One complete phase contains two ordinary steps. Predicting by the
    // remaining phase makes the world landing nearly stationary as the root
    // advances, instead of recomputing a target that chases the body.
    let remaining_travel = phase_to_contact.cycles() * step_distance.metres() * 2.0;
    plan_settle_landing(
        rig_origin,
        rig_rotation,
        projected_com + direction * remaining_travel,
        direction,
        side,
    )
}

pub(in crate::animation::procedural) fn acquisition_planted_target(
    planted_target: Vec3,
    upper_root: Vec3,
    maximum_reach: ReachLimit,
    gait: LocomotionGait,
    acquired: bool,
) -> Option<Vec3> {
    if acquired || gait == LocomotionGait::Run {
        Some(planted_target)
    } else {
        constrain_target_to_reach(planted_target, upper_root, maximum_reach)
    }
}

pub(in crate::animation::procedural) fn advance_scalar_at_speed(
    current: SignedLength,
    desired: SignedLength,
    delta_seconds: AnimationElapsed,
    speed: adventuresim_core::linear_speed::SignedSpeed,
) -> AnimationValueResult<SignedLength> {
    let maximum_step = speed.nonnegative_part().metres_per_second() * delta_seconds.seconds();
    SignedLength::from_metres(
        current.metres() + (desired.metres() - current.metres()).clamp(-maximum_step, maximum_step),
    )
    .map_err(|_| AnimationValueError::Overflow {
        quantity: AnimationQuantity::HeightOffset,
    })
}

pub(in crate::animation::procedural) fn terrain_maximum_reach(
    upper_length: NonnegativeLength,
    lower_length: NonnegativeLength,
) -> ReachLimit {
    ReachLimit::Flexed {
        upper: upper_length,
        lower: lower_length,
        minimum_flexion: ik_tuning().minimum_terrain_knee_flexion_degrees,
    }
}

pub(in crate::animation::procedural) fn exclusive_ground_support(
    left: UnitInterval,
    right: UnitInterval,
    phase: CyclePhase,
) -> (UnitInterval, UnitInterval) {
    if left == UnitInterval::ZERO {
        return (UnitInterval::ZERO, contact_support_weight(right));
    }
    if right == UnitInterval::ZERO {
        return (contact_support_weight(left), UnitInterval::ZERO);
    }
    if left > right
        || ((left.fraction() - right.fraction()) == 0.0 && phase.cycles().rem_euclid(1.0) >= 0.75)
    {
        (contact_support_weight(left), UnitInterval::ZERO)
    } else {
        (UnitInterval::ZERO, contact_support_weight(right))
    }
}

pub(in crate::animation::procedural) fn contact_support_weight(
    weight: UnitInterval,
) -> UnitInterval {
    // Preserve the complete profile-owned support window. Thresholding this
    // confidence delayed the effective contact edge and lengthened a 5.5 m/s
    // run flight from about 100 ms to roughly 140 ms.
    weight
}

pub(in crate::animation::procedural) fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(in crate::animation::procedural) fn smootherstep01(value: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}
