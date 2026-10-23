//! Run contact reach, motion disks and terrain-conformed footprint planning.

use super::*;

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
    maximum_reach: ReachLimit,
) -> bool {
    maximum_reach
        .native_metres()
        .is_some_and(|reach| target.distance(upper_root) <= reach + 0.001)
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
    side: FootTrackSide,
    upper_root: Vec3,
    maximum_reach: ReachLimit,
    delta_seconds: AnimationElapsed,
    terrain_height_at: impl Fn(ScenePlanPoint) -> Option<SupportElevation>,
) -> Option<Vec3> {
    let previous_owner = previous_owner?;
    let maximum_reach = maximum_reach.native_metres()?;
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
        let height = terrain_height_at(ScenePlanPoint::from_metres(transported_contact.xz())?)?;
        transported_contact.y = height.metres() + measured_ankle_sole_offset_metres().metres();
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
    let height = terrain_height_at(ScenePlanPoint::from_metres(transported_contact.xz())?)?;
    transported_contact.y = height.metres() + measured_ankle_sole_offset_metres().metres();
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

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(in crate::animation::procedural) fn reachable_run_contact_target(
    mut candidate: Vec3,
    current_upper_root: Vec3,
    velocity: Vec3,
    step_distance: NonnegativeLength,
    phase_to_contact: CycleDelta,
    contact_ready_phase: CycleDelta,
    maximum_reach: ReachLimit,
    terrain_height_at: impl Fn(ScenePlanPoint) -> Option<SupportElevation>,
) -> Option<Vec3> {
    let contact_ready_phase = contact_ready_phase.cycles();
    let direction = velocity.with_y(0.0).try_normalize().unwrap_or(Vec3::NEG_Z);
    let support_radius =
        (contact_ready_phase - ik_tuning().run_contact_chain_settle_phase.fraction()).max(0.0);
    let travel_per_phase = step_distance.metres() * 2.0;
    let current_terrain_height =
        terrain_height_at(ScenePlanPoint::from_metres(current_upper_root.xz())?);
    let [Some(entry), Some(centre), Some(exit)] = [
        phase_to_contact.cycles() - support_radius,
        phase_to_contact.cycles(),
        phase_to_contact.cycles() + support_radius,
    ]
    .map(|remaining_phase| {
        let mut root =
            current_upper_root + direction * (remaining_phase.max(0.0) * travel_per_phase);
        if let (Some(current_height), Some(predicted_height)) = (
            current_terrain_height,
            terrain_height_at(ScenePlanPoint::from_metres(root.xz())?),
        ) {
            root.y += predicted_height.metres() - current_height.metres();
        }
        Some(
            root - Vec3::Y
                * ik_tuning()
                    .run_maximum_planned_reach_pelvis_drop_metres
                    .metres(),
        )
    }) else {
        return None;
    };
    let predicted_upper_roots = [entry, centre, exit];
    // The world footprint must remain reachable for the whole stance, not
    // merely at entry. Project its XZ into the intersection of the predicted
    // entry/center/exit reach disks. Dykstra's deterministic projection keeps
    // an already feasible desired footprint unchanged and finds the closest
    // point in the convex intersection otherwise. Resampling between passes
    // accounts for the changing vertical budget on sloped terrain.
    for _ in 0..4 {
        if let Some(height) = terrain_height_at(ScenePlanPoint::from_metres(candidate.xz())?) {
            candidate.y = height.metres() + measured_ankle_sole_offset_metres().metres();
        }
        candidate = project_run_contact_into_reach_intersection(
            candidate,
            predicted_upper_roots,
            maximum_reach,
        )?;
    }
    if let Some(height) = terrain_height_at(ScenePlanPoint::from_metres(candidate.xz())?) {
        candidate.y = height.metres() + measured_ankle_sole_offset_metres().metres();
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
            )?;
            if let Some(height) = terrain_height_at(ScenePlanPoint::from_metres(candidate.xz())?) {
                candidate.y = height.metres() + measured_ankle_sole_offset_metres().metres();
            }
        }
    }
    Some(candidate)
}

pub(in crate::animation::procedural) fn project_run_contact_into_reach_intersection(
    mut candidate: Vec3,
    predicted_upper_roots: [Vec3; 3],
    maximum_reach: ReachLimit,
) -> Option<Vec3> {
    let maximum_reach = maximum_reach.native_metres()?;
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
    Some(candidate)
}

pub(in crate::animation::procedural) fn run_contact_reachable_through_stance(
    candidate: Vec3,
    predicted_upper_roots: [Vec3; 3],
    maximum_reach: ReachLimit,
) -> bool {
    let Some(maximum_reach) = maximum_reach.native_metres() else {
        return false;
    };
    predicted_upper_roots
        .into_iter()
        .all(|root| candidate.distance(root) <= maximum_reach + 0.001)
}
