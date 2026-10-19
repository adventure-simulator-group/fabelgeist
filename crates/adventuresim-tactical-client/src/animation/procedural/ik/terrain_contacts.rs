//! Ordinary and run contact acquisition, release, reach, and clearance policy.

use super::*;
use adventuresim_core::linear_speed::NonnegativeSpeed;
use adventuresim_tactical_core::animation::STATIONARY_ANIMATION_SPEED_LIMIT;
pub(in crate::animation::procedural) use approach::run_contact_approach_progress;
pub(in crate::animation::procedural) use foot_clearance::{
    ordinary_terrain_target, run_airborne_clearance, run_airborne_clearance_for_sample,
    run_clearance_target_height, run_swing_clearance, settle_swing_target,
    toe_aware_minimum_ankle_y, transition_toe_clearance_with_rotation_margin,
};

mod approach;
mod foot_clearance;

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
    minimum_world_y: impl Fn(Vec2) -> Option<f32>,
) -> Vec3 {
    if run_budget && !has_plan && !acquired && release_owned {
        advance_run_airborne_world_target(
            previous_owner_target,
            desired_world_target,
            rig_origin,
            rig_rotation,
            delta_seconds,
            target_speed,
            minimum_world_y,
        )
    } else {
        desired_world_target
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
    support_radius: f32,
) -> bool {
    let post_contact = phase.since_contact(foot_side).cycles();
    // Release on the first sampled phase beyond the nominal lobe, not on its
    // decaying shoulder. The half-cycle bound distinguishes this foot's
    // post-contact side from its next rising shoulder after wrap.
    post_contact >= support_radius && post_contact < 0.5
}

pub(in crate::animation::procedural) fn exhausted_latch_after_raw_cadence(
    exhausted: bool,
    raw_nominal_support: f32,
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
    support_radius: f32,
    raw_nominal_support: f32,
    planned_contact: Option<Vec3>,
    acquired: bool,
) -> bool {
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
    previous_support: Option<f32>,
    current_support: f32,
) -> bool {
    planned_contact.is_some()
        && !acquired
        && previous_support.is_some_and(terrain_leg_has_support)
        && !terrain_leg_has_support(current_support)
}

pub(in crate::animation::procedural) fn support_after_exhausted_lobe(
    exhausted: bool,
    nominal_weight: f32,
) -> (bool, f32) {
    if !exhausted {
        (false, nominal_weight)
    } else if terrain_leg_has_support(nominal_weight) {
        (true, 0.0)
    } else {
        (false, nominal_weight)
    }
}

pub(in crate::animation::procedural) fn run_planned_contact_allowed(
    support_lobe_exhausted: bool,
    phase_to_contact: CycleDelta,
    approach_window: f32,
) -> bool {
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
    phase_to_contact: CycleDelta,
    contact_ready_phase: f32,
) -> Vec3 {
    let remaining_phase = (phase_to_contact.cycles() - contact_ready_phase).max(0.0);
    let root_travel = remaining_phase * ordinary_step_distance(speed) * 2.0;
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
    nominal_support: f32,
    acquired: bool,
    planned_contact: Option<Vec3>,
) -> bool {
    gait == LocomotionGait::Run
        && speed > STATIONARY_ANIMATION_SPEED_LIMIT
        && terrain_leg_has_support(nominal_support)
        && !acquired
        && planned_contact.is_none()
}

pub(in crate::animation::procedural) fn run_support_eligible_for_descent(
    gait: LocomotionGait,
    phase: CyclePhase,
    foot_side: LeadFoot,
    support_radius: f32,
    raw_nominal_support: f32,
    contact_reachable: bool,
) -> bool {
    gait == LocomotionGait::Run
        && contact_reachable
        && terrain_leg_has_support(raw_nominal_support)
        // Only the rising shoulder approaches this foot's contact center.
        // The symmetric raw weight on the post-contact shoulder belongs to
        // stance/toe-off and must not pull an unacquired next-cycle plan down.
        && phase.until_contact(foot_side).cycles() <= support_radius + 0.001
}

pub(in crate::animation::procedural) fn run_contact_within_follower_step(
    previous_owner: Option<Vec3>,
    desired_world: Vec3,
    rig_origin: Vec3,
    rig_rotation: Quat,
    delta_seconds: AnimationElapsed,
) -> bool {
    let Some(previous_owner) = previous_owner else {
        return true;
    };
    let desired_owner = rig_rotation.inverse() * (desired_world - rig_origin);
    previous_owner.distance(desired_owner)
        <= (ik_tuning().run_airborne_owner_step_metres.metres()
            * ik_tuning().continuity_sample_hz.hertz())
            * delta_seconds.seconds()
            + sole_contact_tolerance_metres().metres()
}

pub(in crate::animation::procedural) fn run_contact_within_leg_reach(
    target: Vec3,
    upper_root: Vec3,
    maximum_reach: f32,
) -> bool {
    target.distance(upper_root) <= maximum_reach + 0.001
}

pub(in crate::animation::procedural) fn run_contact_within_follower_motion_step(
    previous_owner: Option<Vec3>,
    desired_world: Vec3,
    rig_origin: Vec3,
    rig_rotation: Quat,
    delta_seconds: AnimationElapsed,
) -> bool {
    let Some(previous_owner) = previous_owner else {
        return true;
    };
    let desired_owner = rig_rotation.inverse() * (desired_world - rig_origin);
    previous_owner.distance(desired_owner)
        <= (ik_tuning().run_airborne_owner_step_metres.metres()
            * ik_tuning().continuity_sample_hz.hertz())
            * delta_seconds.seconds()
            + 0.0001
}

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(in crate::animation::procedural) fn retarget_unacquired_run_contact_for_descent(
    previous_owner: Option<Vec3>,
    fixed_contact: Vec3,
    rig_origin: Vec3,
    rig_rotation: Quat,
    side: f32,
    upper_root: Vec3,
    maximum_reach: f32,
    delta_seconds: AnimationElapsed,
    terrain_height_at: impl Fn(Vec2) -> Option<f32>,
) -> Option<Vec3> {
    let previous_owner = previous_owner?;
    let fixed_within_motion = run_contact_within_follower_motion_step(
        Some(previous_owner),
        fixed_contact,
        rig_origin,
        rig_rotation,
        delta_seconds,
    );
    let fixed_within_reach = fixed_contact.distance(upper_root) <= maximum_reach + 0.001;
    if fixed_within_motion && fixed_within_reach {
        return None;
    }

    // At 5.5 m/s a fixed world contact recedes 8.6 cm in owner space per
    // sample. Combining that with the final 5 cm descent exceeds the 9 cm
    // continuity budget. A downhill target may instead be inside that motion
    // budget but a few millimetres beyond the analytic leg reach. Start from
    // the visible owner XZ for the first case or the frozen contact for the
    // second, then terrain-resample and project into current reach before
    // freezing the final acquired footprint.
    let start_world = rig_origin + rig_rotation * previous_owner;
    let maximum_motion = (ik_tuning().run_airborne_owner_step_metres.metres()
        * ik_tuning().continuity_sample_hz.hertz())
        * delta_seconds.seconds();
    let mut transported_contact = if fixed_within_motion {
        fixed_contact
    } else {
        start_world
    };
    for _ in 0..4 {
        transported_contact =
            constrain_foot_to_track(transported_contact, rig_origin, rig_rotation, side);
        let height = terrain_height_at(transported_contact.xz())?;
        transported_contact.y = height + measured_ankle_sole_offset_metres().metres();
        let leg_vertical = transported_contact.y - upper_root.y;
        let leg_horizontal = (maximum_reach * maximum_reach - leg_vertical * leg_vertical)
            .max(0.0)
            .sqrt();
        let motion_vertical = transported_contact.y - start_world.y;
        let motion_horizontal = (maximum_motion * maximum_motion
            - motion_vertical * motion_vertical)
            .max(0.0)
            .sqrt();
        let projected = project_point_into_two_disks(
            transported_contact.xz(),
            [
                (upper_root.xz(), leg_horizontal),
                (start_world.xz(), motion_horizontal),
            ],
        );
        transported_contact.x = projected.x;
        transported_contact.z = projected.y;
    }
    transported_contact =
        constrain_foot_to_track(transported_contact, rig_origin, rig_rotation, side);
    let height = terrain_height_at(transported_contact.xz())?;
    transported_contact.y = height + measured_ankle_sole_offset_metres().metres();
    let accepted = transported_contact.distance(upper_root) <= maximum_reach + 0.001
        && run_contact_within_follower_motion_step(
            Some(previous_owner),
            transported_contact,
            rig_origin,
            rig_rotation,
            delta_seconds,
        );
    if !accepted {
        return None;
    }
    Some(transported_contact)
}

pub(in crate::animation::procedural) fn project_point_into_two_disks(
    mut point: Vec2,
    disks: [(Vec2, f32); 2],
) -> Vec2 {
    let mut corrections = [Vec2::ZERO; 2];
    for _ in 0..24 {
        for (index, (center, radius)) in disks.into_iter().enumerate() {
            let corrected = point + corrections[index];
            let projected = center + (corrected - center).clamp_length_max(radius.max(0.0));
            corrections[index] = corrected - projected;
            point = projected;
        }
    }
    point
}

pub(in crate::animation::procedural) fn ordinary_contact_target(
    rig_origin: Vec3,
    rig_rotation: Quat,
    projected_com: Vec3,
    velocity: Vec3,
    speed: NonnegativeSpeed,
    phase_to_contact: CycleDelta,
    side: f32,
) -> Vec3 {
    let direction = velocity
        .with_y(0.0)
        .try_normalize()
        .unwrap_or(rig_rotation * Vec3::NEG_Z);
    // One complete phase contains two ordinary steps. Predicting by the
    // remaining phase makes the world landing nearly stationary as the root
    // advances, instead of recomputing a target that chases the body.
    let remaining_travel = phase_to_contact.cycles() * ordinary_step_distance(speed) * 2.0;
    plan_settle_landing(
        rig_origin,
        rig_rotation,
        projected_com + direction * remaining_travel,
        direction,
        side,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(in crate::animation::procedural) fn reachable_run_contact_target(
    mut candidate: Vec3,
    current_upper_root: Vec3,
    velocity: Vec3,
    speed: NonnegativeSpeed,
    phase_to_contact: CycleDelta,
    contact_ready_phase: f32,
    maximum_reach: f32,
    terrain_height_at: impl Fn(Vec2) -> Option<f32>,
) -> Vec3 {
    let direction = velocity.with_y(0.0).try_normalize().unwrap_or(Vec3::NEG_Z);
    let support_radius =
        (contact_ready_phase - ik_tuning().run_contact_chain_settle_phase.fraction()).max(0.0);
    let travel_per_phase = ordinary_step_distance(speed) * 2.0;
    let current_terrain_height = terrain_height_at(current_upper_root.xz());
    let predicted_upper_roots = [
        phase_to_contact.cycles() - support_radius,
        phase_to_contact.cycles(),
        phase_to_contact.cycles() + support_radius,
    ]
    .map(|remaining_phase| {
        let mut root =
            current_upper_root + direction * (remaining_phase.max(0.0) * travel_per_phase);
        if let (Some(current_height), Some(predicted_height)) =
            (current_terrain_height, terrain_height_at(root.xz()))
        {
            root.y += predicted_height - current_height;
        }
        root - Vec3::Y
            * ik_tuning()
                .run_maximum_planned_reach_pelvis_drop_metres
                .metres()
    });
    // The world footprint must remain reachable for the whole stance, not
    // merely at entry. Project its XZ into the intersection of the predicted
    // entry/center/exit reach disks. Dykstra's deterministic projection keeps
    // an already feasible desired footprint unchanged and finds the closest
    // point in the convex intersection otherwise. Resampling between passes
    // accounts for the changing vertical budget on sloped terrain.
    for _ in 0..4 {
        if let Some(height) = terrain_height_at(candidate.xz()) {
            candidate.y = height + measured_ankle_sole_offset_metres().metres();
        }
        candidate = project_run_contact_into_reach_intersection(
            candidate,
            predicted_upper_roots,
            maximum_reach,
        );
    }
    if let Some(height) = terrain_height_at(candidate.xz()) {
        candidate.y = height + measured_ankle_sole_offset_metres().metres();
    }
    if !run_contact_reachable_through_stance(candidate, predicted_upper_roots, maximum_reach) {
        // A cliff-like sample can have no shared stance footprint even with
        // the bounded pelvis allowance. Keep support entry continuous; the
        // reach-release latch will end the stance later without reacquiring
        // the same lobe if the hip path still diverges.
        for _ in 0..3 {
            candidate = project_run_contact_into_reach_intersection(
                candidate,
                [predicted_upper_roots[0]; 3],
                maximum_reach,
            );
            if let Some(height) = terrain_height_at(candidate.xz()) {
                candidate.y = height + measured_ankle_sole_offset_metres().metres();
            }
        }
    }
    candidate
}

pub(in crate::animation::procedural) fn project_run_contact_into_reach_intersection(
    mut candidate: Vec3,
    predicted_upper_roots: [Vec3; 3],
    maximum_reach: f32,
) -> Vec3 {
    let horizontal_reaches = predicted_upper_roots.map(|root| {
        let vertical_delta = candidate.y - root.y;
        (maximum_reach * maximum_reach - vertical_delta * vertical_delta)
            .max(0.0)
            .sqrt()
    });
    let mut point = candidate.xz();
    let mut corrections = [Vec2::ZERO; 3];
    for _ in 0..24 {
        for (index, root) in predicted_upper_roots.iter().enumerate() {
            let corrected = point + corrections[index];
            let offset = corrected - root.xz();
            let projected = root.xz() + offset.clamp_length_max(horizontal_reaches[index]);
            corrections[index] = corrected - projected;
            point = projected;
        }
    }
    candidate.x = point.x;
    candidate.z = point.y;
    candidate
}

pub(in crate::animation::procedural) fn run_contact_reachable_through_stance(
    candidate: Vec3,
    predicted_upper_roots: [Vec3; 3],
    maximum_reach: f32,
) -> bool {
    predicted_upper_roots
        .into_iter()
        .all(|root| candidate.distance(root) <= maximum_reach + 0.001)
}

pub(in crate::animation::procedural) fn acquisition_planted_target(
    planted_target: Vec3,
    upper_root: Vec3,
    maximum_reach: f32,
    gait: LocomotionGait,
    acquired: bool,
) -> Vec3 {
    if acquired || gait == LocomotionGait::Run {
        planted_target
    } else {
        constrain_target_to_reach(planted_target, upper_root, maximum_reach)
    }
}

pub(in crate::animation::procedural) fn advance_scalar_at_speed(
    current: f32,
    desired: f32,
    delta_seconds: AnimationElapsed,
    speed: adventuresim_core::linear_speed::SignedSpeed,
) -> f32 {
    let maximum_step = speed.nonnegative_part().metres_per_second() * delta_seconds.seconds();
    current + (desired - current).clamp(-maximum_step, maximum_step)
}

pub(in crate::animation::procedural) fn advance_run_airborne_world_target(
    previous_owner: Option<Vec3>,
    desired_world: Vec3,
    rig_origin: Vec3,
    rig_rotation: Quat,
    delta_seconds: AnimationElapsed,
    speed: NonnegativeSpeed,
    minimum_ankle_y_at: impl Fn(Vec2) -> Option<f32>,
) -> Vec3 {
    let Some(previous_owner) = previous_owner.filter(|target| target.is_finite()) else {
        let mut target = desired_world;
        if let Some(minimum_y) = minimum_ankle_y_at(target.xz()) {
            target.y = target.y.max(minimum_y);
        }
        return target;
    };
    let start_world = rig_origin + rig_rotation * previous_owner;
    let maximum_step = speed.metres_per_second() * delta_seconds.seconds();
    let cleared_at = |progress: f32| {
        let mut target = start_world.lerp(desired_world, progress.clamp(0.0, 1.0));
        if let Some(minimum_y) = minimum_ankle_y_at(target.xz()) {
            target.y = target.y.max(minimum_y);
        }
        target
    };
    let desired = cleared_at(1.0);
    if desired.distance(start_world) <= maximum_step {
        return desired;
    }

    // Clearance and the 3D budget are solved jointly. On an uphill release,
    // both full owner transport (terrain rise) and literal world hold (root
    // displacement) can be outside the sphere while an intermediate XZ is
    // feasible. That set is not monotone from either endpoint, so first scan
    // deterministically for its farthest feasible interval, then refine only
    // the local exit boundary.
    const SEARCH_SAMPLES: usize = 64;
    let mut best = None;
    let mut best_progress = 0.0;
    for index in 0..=SEARCH_SAMPLES {
        let progress = index as f32 / SEARCH_SAMPLES as f32;
        let candidate = cleared_at(progress);
        if candidate.distance(start_world) <= maximum_step + 0.000001 {
            best = Some(candidate);
            best_progress = progress;
        }
    }
    let Some(mut best) = best else {
        // The straight owner-transport-to-world-plant segment can cross a
        // locally high triangle even when a small lateral/fore-aft detour is
        // inside the same 3D motion sphere. Search that complete horizontal
        // disk before accepting an over-budget fallback. This matters at
        // toe-off: the semantic sole floor and a terrain rise can make both
        // line endpoints invalid while a nearby downhill point remains valid.
        if let Some(feasible) = terrain_feasible_target_in_step(
            start_world,
            desired_world,
            maximum_step,
            &minimum_ankle_y_at,
        ) {
            return feasible;
        }
        // A true vertical discontinuity can leave no valid point in the full
        // disk. Keep clearance truthful and choose the least-discontinuous
        // line sample; support/reach ownership handles that rare fallback.
        let fallback = (0..=SEARCH_SAMPLES)
            .map(|index| cleared_at(index as f32 / SEARCH_SAMPLES as f32))
            .min_by(|left, right| {
                left.distance_squared(start_world)
                    .total_cmp(&right.distance_squared(start_world))
            })
            .unwrap_or_else(|| cleared_at(0.0));
        return fallback;
    };
    let mut low = best_progress;
    let mut high = (best_progress + 1.0 / SEARCH_SAMPLES as f32).min(1.0);
    for _ in 0..12 {
        let middle = (low + high) * 0.5;
        let candidate = cleared_at(middle);
        if candidate.distance(start_world) <= maximum_step + 0.000001 {
            low = middle;
            best = candidate;
        } else {
            high = middle;
        }
    }
    best
}

pub(in crate::animation::procedural) fn terrain_feasible_target_in_step(
    start_world: Vec3,
    desired_world: Vec3,
    maximum_step: f32,
    minimum_ankle_y_at: &impl Fn(Vec2) -> Option<f32>,
) -> Option<Vec3> {
    if maximum_step <= 0.0 {
        return None;
    }
    let mut best: Option<(f32, Vec3)> = None;
    let mut center = start_world.xz();
    let mut radius = maximum_step;
    // A deterministic square-disk search followed by local refinements finds
    // terrain-feasible points off the direct swing chord without introducing
    // frame-rate-dependent iteration or lateral drift.
    for refinement in 0..4 {
        const HALF_GRID: i32 = 12;
        let spacing = radius / HALF_GRID as f32;
        for x in -HALF_GRID..=HALF_GRID {
            for z in -HALF_GRID..=HALF_GRID {
                let xz = center + Vec2::new(x as f32 * spacing, z as f32 * spacing);
                let offset = xz - start_world.xz();
                if offset.length_squared() > maximum_step * maximum_step + 0.000001 {
                    continue;
                }
                let chord = desired_world.xz() - start_world.xz();
                let chord_progress = if chord.length_squared() > 0.0 {
                    offset.dot(chord) / chord.length_squared()
                } else {
                    0.0
                }
                .clamp(0.0, 1.0);
                let nearest_chord = start_world.xz() + chord * chord_progress;
                if xz.distance(nearest_chord) > 0.04 {
                    continue;
                }
                let minimum_y = minimum_ankle_y_at(xz)?;
                let candidate = Vec3::new(xz.x, desired_world.y.max(minimum_y), xz.y);
                if candidate.distance(start_world) > maximum_step + 0.000001 {
                    continue;
                }
                let score = candidate.distance_squared(desired_world);
                if best.is_none_or(|(best_score, _)| score < best_score) {
                    best = Some((score, candidate));
                }
            }
        }
        let Some((_, candidate)) = best else {
            break;
        };
        if refinement < 3 {
            center = candidate.xz();
            radius = spacing * 2.0;
        }
    }
    best.map(|(_, candidate)| candidate)
}

pub(in crate::animation::procedural) fn terrain_maximum_reach(
    upper_length: f32,
    lower_length: f32,
) -> f32 {
    (upper_length * upper_length
        + lower_length * lower_length
        + 2.0
            * upper_length
            * lower_length
            * ik_tuning()
                .minimum_terrain_knee_flexion_degrees
                .to_radians()
                .cos())
    .sqrt()
}

pub(in crate::animation::procedural) fn exclusive_ground_support(
    left: f32,
    right: f32,
    phase: CyclePhase,
) -> (f32, f32) {
    if left <= 0.0 {
        return (0.0, contact_support_weight(right));
    }
    if right <= 0.0 {
        return (contact_support_weight(left), 0.0);
    }
    if left > right || ((left - right) == 0.0 && phase.cycles().rem_euclid(1.0) >= 0.75) {
        (contact_support_weight(left), 0.0)
    } else {
        (0.0, contact_support_weight(right))
    }
}

pub(in crate::animation::procedural) fn contact_support_weight(weight: f32) -> f32 {
    // Preserve the complete profile-owned support window. Thresholding this
    // confidence delayed the effective contact edge and lengthened a 5.5 m/s
    // run flight from about 100 ms to roughly 140 ms.
    weight.clamp(0.0, 1.0)
}

pub(in crate::animation::procedural) fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(in crate::animation::procedural) fn smootherstep01(value: f32) -> f32 {
    let t = value.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}
