//! Procedural lower-body coordination.
//!
//! This facade owns Bevy query routing, presentation-tick sequencing, owner
//! handoffs, and the single ordered terrain solve. Retained component state,
//! raised footwork, stop settlement, terrain-contact policy, orientation, and
//! propagated-pose observation live in responsibility-named implementation
//! fragments below. Hand constraints, authored locomotion conformity, and the
//! analytic two-bone solver remain ordinary cohesive child modules.

use super::*;
use adventuresim_building_generator::spatial_geometry::{PositiveLength, SignedLength};
use adventuresim_core::linear_speed::NonnegativeSpeed;
use adventuresim_tactical_core::animation::STATIONARY_ANIMATION_SPEED_LIMIT;
use adventuresim_tactical_core::{
    city_layout::grounding::SupportElevation, scene_coordinates::ScenePlanPoint,
};
use adventuresim_tactical_core::animation::{
    AngularMagnitudeDegrees, HeightDistance, NonnegativeLength, SettleProgress, SignedAngleDegrees,
    SignedAngularSpeedDegrees, UnitInterval,
};

use frame::ObservedRigFrame;
pub(in crate::animation) use hands::apply_arm_and_weapon_constraints;
#[cfg(test)]
pub(super) use hands::secondary_grip_world;
pub(crate) use hands::{HandIkTarget, HandSide, HeldWeaponConstraint, HumanoidIkTargets};
pub(crate) use locomotion::locomotion_support_weights;
pub(super) use locomotion::owns as authored_locomotion_owns;
pub(in crate::animation) use observation::refresh_raised_support_after_propagation;
use observation::*;
#[cfg(test)]
pub(super) use observation::{
    raised_footwork_posture_is_valid, retained_plant_requires_release, terrain_ik_posture_is_valid,
    terrain_leg_has_support,
};
pub(super) use orientation::constrain_rendered_leg_pole;
pub(in crate::animation) use orientation::enforce_anatomical_knee_yaw;
use orientation::*;
#[cfg(test)]
pub(super) use orientation::{
    anatomical_knee_yaw_posture_is_valid, authored_knee_pole_world, slope_aligned_world_rotation,
};
use plant::FootPlant;
pub(crate) use raised_footwork::RaisedFootworkState;
use raised_footwork::*;
#[cfg(test)]
pub(super) use raised_footwork::{constrain_foot_to_track, terrain_conformed_guard_target};
pub(in crate::animation::procedural) use reach::{LimbSegments, ReachLimit};
use settle::*;
#[cfg(test)]
pub(super) use settle::{
    balance_recovery_direction, plan_settle_landing, projected_capture_point, settle_swing_side,
    sole_is_at_contact,
};
pub(super) use solver::*;
use state::publication::FootSolveRequest;
use state::*;
pub(crate) use state::{ArmIkState, LegIkDiagnostics, LegIkState};
use target_speed::FootTargetSpeeds;
use terminal::TerminalContacts;
pub(super) use terrain_contacts::smoothstep;
use terrain_contacts::*;
pub(in crate::animation::procedural) use track::FootTrackSide;

mod evaluation_clock;
mod foot_roll;
mod frame;
mod hands;
mod locomotion;
mod observation;
mod orientation;
mod plant;
mod raised_footwork;
mod reach;
mod settle;
mod solver;
mod state;
mod target_speed;
mod terminal;
mod terrain_contacts;
mod track;

pub(super) fn minimum_inter_foot_separation_metres() -> PositiveLength {
    ik_tuning().minimum_inter_foot_separation_metres
}

pub(super) fn foot_track_inner_metres() -> NonnegativeLength {
    NonnegativeLength::from(minimum_inter_foot_separation_metres()).half()
}

pub(super) fn maximum_pelvis_correction_step_metres() -> NonnegativeLength {
    ik_tuning().maximum_pelvis_correction_step_metres
}

pub(crate) fn measured_ankle_sole_offset_metres() -> PositiveLength {
    ik_tuning().measured_ankle_sole_offset_metres
}

pub(crate) fn sole_contact_tolerance_metres() -> PositiveLength {
    ik_tuning().sole_contact_tolerance_metres
}

/// Places the planted foot on the terrain with an analytic two-bone solve,
/// then lowers the hips by the bounded residual. Existing weapon/hand
/// constraints run at the same final-pose seam.
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects each independently borrowed terrain IK resource and query as a system parameter"
)]
pub(in crate::animation) fn apply_terrain_leg_ik(
    enabled: Res<super::super::TerrainIkMode>,
    time: Res<Time>,
    clock: Res<ProceduralAnimationClock>,
    terrain: Query<&SceneTerrain>,
    owners: Query<&PresentedSkeleton>,
    rigs: Query<(Entity, &HumanoidRig)>,
    parents: Query<&ChildOf>,
    mut ik_states: Query<&mut LegIkState>,
    mut raised_states: Query<&mut RaisedFootworkState>,
    mut transforms: ParamSet<(TransformHelper, Query<&mut Transform>)>,
    mut commands: Commands,
) {
    let _spike = crate::animation::diagnostics::SpikeGuard::new("apply_terrain_leg_ik");
    let terrain = terrain.single().ok();
    let target_speeds = match FootTargetSpeeds::configured() {
        Ok(speeds) => speeds,
        Err(error) => {
            warn!(%error, "Rejected terrain target speeds");
            return;
        }
    };
    for (owner, rig) in &rigs {
        let Ok(skeleton) = owners.get(owner) else {
            continue;
        };
        if locomotion::owns(skeleton) {
            // The authored pose remains untouched, but allocate observational
            // memory so the post-propagation pass can retain its final feet
            // for a continuous transfer to stationary procedural footwork.
            if let Ok(mut state) = ik_states.get_mut(owner) {
                // Authored locomotion has already accepted the post-dodge
                // lower body. A still-pending landing handoff must not survive
                // that owner and reappear later when the character stops.
                release_raised_state_for_authored_locomotion(&mut state.0);
            } else {
                commands
                    .entity(owner)
                    .insert(LegIkState(LegIkMemory::default()));
            }
            if let Ok(mut state) = raised_states.get_mut(owner) {
                let step_sequence = retain_monotonic_contact_sequence(
                    state.step_sequence,
                    skeleton.contact_sequence,
                );
                *state = RaisedFootworkState {
                    step_sequence,
                    ..default()
                };
            } else {
                commands.entity(owner).insert(RaisedFootworkState {
                    step_sequence: skeleton.contact_sequence,
                    ..default()
                });
            }
            continue;
        }
        if !terrain_ik_posture_is_valid(skeleton) {
            if let Ok(mut state) = ik_states.get_mut(owner) {
                state.0 = LegIkMemory::default();
            } else {
                // Quicksteps bypass every leg modifier, yet need a component
                // on which the later propagated-pose observer can record the
                // authored landing endpoints.
                commands
                    .entity(owner)
                    .insert(LegIkState(LegIkMemory::default()));
            }
            if let Ok(mut state) = raised_states.get_mut(owner) {
                let step_sequence = retain_monotonic_contact_sequence(
                    state.step_sequence,
                    skeleton.contact_sequence,
                );
                *state = RaisedFootworkState {
                    step_sequence,
                    ..default()
                };
            }
            continue;
        }
        let speed = match skeleton.animation_speed() {
            Ok(speed) => speed,
            Err(error) => {
                warn!(%error, ?owner, "Rejected terrain IK animation speed");
                continue;
            }
        };
        let raised_guard_follower = raised_footwork_posture_is_valid(skeleton)
            && skeleton.weapon_guard() == WeaponGuardState::Raised
            && matches!(
                skeleton.action_kind(),
                SkeletonAction::None | SkeletonAction::Attack
            );
        let raised_footwork_was_active = raised_states
            .get(owner)
            .is_ok_and(|state| state.step.initialized());
        let raised_footwork_handoff = !raised_guard_follower && raised_footwork_was_active;
        let (mut left_weight, mut right_weight) = match locomotion_support_weights(skeleton, speed)
        {
            Ok(weights) => weights,
            Err(error) => {
                warn!(%error, ?owner, "Rejected locomotion support");
                continue;
            }
        };
        // Preserve the profile-owned cadence before settle, ownership, and
        // exhausted-lobe state can suppress effective support. Run touchdown
        // descent is a phase fact and must never be derived from the mutable
        // solver/reporting weights below.
        let profile = match locomotion_profile(skeleton, speed) {
            Ok(profile) => profile,
            Err(error) => {
                warn!(%error, ?owner, "Rejected locomotion profile");
                continue;
            }
        };
        let contact_ready_phase =
            match profile
                .support_phase_radius
                .checked_add(CycleDelta::from_unit_cycles(
                    ik_tuning().run_contact_chain_settle_phase,
                )) {
                Ok(phase) => phase,
                Err(error) => {
                    warn!(%error, "contact-ready phase rejected");
                    continue;
                }
            };
        let raw_run_support = match (profile.gait == LocomotionGait::Run)
            .then(|| gait_support_weights(profile, skeleton.gait_phase.wrapped()))
            .transpose()
        {
            Ok(support) => support,
            Err(error) => {
                warn!(%error, "raw cadence support rejected");
                continue;
            }
        };
        let mut legs = [
            (
                BoneRole::ThighLeft,
                BoneRole::ShinLeft,
                BoneRole::FootLeft,
                left_weight,
                LeadFoot::Left,
            ),
            (
                BoneRole::ThighRight,
                BoneRole::ShinRight,
                BoneRole::FootRight,
                right_weight,
                LeadFoot::Right,
            ),
        ];
        let (mut memory, memory_was_missing) = match ik_states.get_mut(owner) {
            Ok(state) => (state.0, false),
            Err(_) => (
                // Startup is not a toggle transition: establish the configured
                // mode immediately so the first supported frame can plant.
                LegIkMemory {
                    terrain_blend: if enabled.is_enabled() {
                        UnitInterval::ONE
                    } else {
                        UnitInterval::ZERO
                    },
                    ..default()
                },
                true,
            ),
        };
        let observation = memory.sample_elapsed(&clock, &time);
        let evaluation_advances = observation.is_some();
        let state_delta_seconds = observation.unwrap_or(AnimationElapsed::ZERO);
        if repeated_fixed_tick_skips_ik(clock.fixed_tick.is_some(), evaluation_advances) {
            // Multi-view tools restore the first complete local pose at the
            // end of the procedural chain. Re-running IK from memory already
            // advanced by that first view can re-enter a decaying support
            // branch and commit a next-cycle plant into the same logical tick.
            // Skip both evaluation and state mutation for repeated fixed ticks.
            continue;
        }
        if !state_delta_seconds.is_zero() {
            let desired = if terrain_ik_is_required(
                *enabled,
                memory.settle.is_some(),
                raised_footwork_handoff,
            ) {
                1.0
            } else {
                0.0
            };
            memory.terrain_blend = match UnitInterval::from_fraction(
                memory.terrain_blend.fraction()
                    + (desired - memory.terrain_blend.fraction()).clamp(
                        -ik_tuning().terrain_blend_speed_per_second.per_second()
                            * state_delta_seconds.seconds(),
                        ik_tuning().terrain_blend_speed_per_second.per_second()
                            * state_delta_seconds.seconds(),
                    ),
            ) {
                Ok(blend) => blend,
                Err(error) => {
                    warn!(%error, "terrain blend rejected");
                    continue;
                }
            };
        }
        let terrain_blend = memory.terrain_blend;
        // Plant and pelvis reach belong to the server-owned authored-body
        // frame. Terrain knee poles retain their world bend plane separately
        // so a sharp owner turn cannot corkscrew a planted knee.
        let frame = match ObservedRigFrame::resolve(rig, &transforms.p0()) {
            Ok(Some(frame)) => frame,
            Ok(None) => continue,
            Err(error) => {
                warn!(%error, ?owner, "Rejected terrain IK rig frame");
                continue;
            }
        };
        let (rig_origin, rig_rotation) = (frame.origin(), frame.rotation());
        if let Some((left, right)) = memory.quickstep_handoff.targets() {
            memory.left_foot_world_target = Some(rig_origin + rig_rotation * left);
            memory.right_foot_world_target = Some(rig_origin + rig_rotation * right);
        }
        if let Err(error) =
            memory.observe_owner_frame(frame, state_delta_seconds, evaluation_advances)
        {
            warn!(%error, ?owner, "Rejected owner-motion measurement");
            continue;
        }
        if raised_footwork_handoff {
            // A mode that owns the full lower body supersedes guard footwork.
            // Preserve both last visible targets as the beginning of a bounded
            // balance capture instead of reacquiring authored gait feet.
            if let Ok(mut raised) = raised_states.get_mut(owner) {
                preserve_raised_handoff_targets(&mut memory, *raised, rig_origin, rig_rotation);
                *raised = RaisedFootworkState::default();
            }
        }
        let ordinary_lowered = skeleton.weapon_guard() == WeaponGuardState::Lowered
            && skeleton.action_kind() == SkeletonAction::None;
        let planar_velocity = skeleton.world_velocity.with_y(0.0);
        let planar_speed = match NonnegativeSpeed::from_metres_per_second(planar_velocity.length())
        {
            Ok(speed) => speed,
            Err(error) => {
                warn!(%error, ?owner, "Rejected terrain planar speed");
                continue;
            }
        };
        if memory.observe_movement_for_settle(
            ordinary_lowered,
            raised_footwork_handoff,
            speed,
            planar_velocity,
        ) && let Err(error) = memory.begin_settle_if_needed(
            rig,
            &transforms.p0(),
            frame,
            terrain,
            raised_footwork_handoff,
        ) {
            warn!(%error, "settle planning rejected");
            continue;
        }

        let settle_cancelled_for_restart =
            ordinary_lowered && speed > STATIONARY_ANIMATION_SPEED_LIMIT && memory.settle.is_some();
        if settle_cancelled_for_restart {
            // A restart invalidates the balance-capture trajectory immediately.
            // Keeping a cancelled settle alive until both release targets
            // converged could starve ordinary gait acquisition indefinitely:
            // the authored swing kept moving, so neither release ever became
            // idle. Retain the already bounded visible targets, but return
            // ownership to ordinary phase/contact planning on this tick.
            cancel_settle_for_restart(&mut memory, planar_velocity);
        }
        let mut settle_ready_for_contact = false;
        if let Some(mut settle) = memory.settle {
            if !state_delta_seconds.is_zero() {
                settle = match advance_settle_state(settle, state_delta_seconds) {
                    Ok(settle) => settle,
                    Err(error) => {
                        warn!(%error, "Rejected IK settlement interval");
                        continue;
                    }
                };
            }
            settle_ready_for_contact = settle.progress.is_complete();
            if settle.support_foot == LeadFoot::Left {
                left_weight = UnitInterval::ONE;
                right_weight = UnitInterval::ZERO;
            } else {
                left_weight = UnitInterval::ZERO;
                right_weight = UnitInterval::ONE;
            }
            legs[0].3 = left_weight;
            legs[1].3 = right_weight;
            memory.settle = Some(settle);
        }
        // A compact combat stance needs real leg reach reserve. Without this
        // shared root drop the authored near-straight legs exhaust their reach
        // after only a few centimetres of lateral root travel, forcing the
        // support foot to slide. This moves the body, never either contact.
        if !state_delta_seconds.is_zero() {
            let desired = if raised_guard_follower {
                -ik_tuning().guard_reach_pelvis_drop_metres.metres()
            } else {
                0.0
            };
            memory.raised_pelvis_shift = match SignedLength::from_metres(desired)
                .map_err(|_| AnimationValueError::Overflow {
                    quantity: AnimationQuantity::HeightOffset,
                })
                .and_then(|desired| {
                    advance_pelvis_shift(memory.raised_pelvis_shift, desired, state_delta_seconds)
                }) {
                Ok(shift) => shift,
                Err(error) => {
                    warn!(%error, "raised pelvis correction rejected");
                    continue;
                }
            };
        }
        if raised_guard_follower {
            prepare_slope_rotation_cache(&mut memory, SlopeAlignmentMode::Raised);
            if memory.raised_pelvis_shift.metres() < -0.001
                && let Some(&root) = rig.get(&BoneRole::Root)
            {
                let local_delta = parents
                    .get(root)
                    .ok()
                    .and_then(|parent| {
                        transforms
                            .p0()
                            .compute_global_transform(parent.parent())
                            .ok()
                    })
                    .map(|parent| {
                        parent
                            .affine()
                            .inverse()
                            .transform_vector3(Vec3::Y * memory.raised_pelvis_shift.metres())
                    })
                    .unwrap_or(Vec3::Y * memory.raised_pelvis_shift.metres());
                if local_delta.is_finite()
                    && let Ok(mut transform) = transforms.p1().get_mut(root)
                {
                    transform.translation += local_delta;
                }
            }
            // Retained plants may request the minimum pelvis correction needed
            // for reach, but the authored guard stance owns the resting height.
            let left = (
                rig.get(&BoneRole::ThighLeft),
                rig.get(&BoneRole::ShinLeft),
                rig.get(&BoneRole::FootLeft),
            );
            let right = (
                rig.get(&BoneRole::ThighRight),
                rig.get(&BoneRole::ShinRight),
                rig.get(&BoneRole::FootRight),
            );
            let (Some(&left_upper), Some(&left_lower), Some(&left_foot)) = left else {
                continue;
            };
            let (Some(&right_upper), Some(&right_lower), Some(&right_foot)) = right else {
                continue;
            };
            let Some((left_upper_snapshot, left_lower_snapshot, left_foot_snapshot)) =
                snapshot_chain(
                    left_upper,
                    left_lower,
                    left_foot,
                    &parents,
                    &transforms.p0(),
                )
            else {
                continue;
            };
            let Some((right_upper_snapshot, right_lower_snapshot, right_foot_snapshot)) =
                snapshot_chain(
                    right_upper,
                    right_lower,
                    right_foot,
                    &parents,
                    &transforms.p0(),
                )
            else {
                continue;
            };
            let LimbSegments {
                upper: left_upper_length,
                lower: left_lower_length,
            } = match LimbSegments::measure(
                left_upper_snapshot.global.translation(),
                left_lower_snapshot.global.translation(),
                left_foot_snapshot.global.translation(),
            ) {
                Ok(segments) => segments,
                Err(error) => {
                    warn!(%error, "IK segment measurement rejected");
                    continue;
                }
            };
            let LimbSegments {
                upper: right_upper_length,
                lower: right_lower_length,
            } = match LimbSegments::measure(
                right_upper_snapshot.global.translation(),
                right_lower_snapshot.global.translation(),
                right_foot_snapshot.global.translation(),
            ) {
                Ok(segments) => segments,
                Err(error) => {
                    warn!(%error, "IK segment measurement rejected");
                    continue;
                }
            };
            let mut footwork = raised_states
                .get_mut(owner)
                .map(|state| *state)
                .unwrap_or_default();
            let previous_swing = footwork.step.swing_foot();
            let left_authored = left_foot_snapshot.global.translation();
            let right_authored = right_foot_snapshot.global.translation();
            let live_speed = planar_speed;
            let handoff_targets = memory.quickstep_handoff.targets();
            let visible_left = if let Some((landing_local, _)) = handoff_targets {
                let authored_local = rig_rotation.inverse() * (left_authored - rig_origin);
                let landing_local =
                    landing_local + (authored_local - landing_local).clamp_length_max(0.015);
                rig_origin + rig_rotation * landing_local
            } else {
                memory.left_foot_world_target.unwrap_or(left_authored)
            };
            let visible_right = if let Some((_, landing_local)) = handoff_targets {
                let authored_local = rig_rotation.inverse() * (right_authored - rig_origin);
                let landing_local =
                    landing_local + (authored_local - landing_local).clamp_length_max(0.015);
                rig_origin + rig_rotation * landing_local
            } else {
                memory.right_foot_world_target.unwrap_or(right_authored)
            };
            if memory.quickstep_handoff.is_pending() {
                memory.quickstep_handoff.update_targets(
                    rig_rotation.inverse() * (visible_left - rig_origin),
                    rig_rotation.inverse() * (visible_right - rig_origin),
                );
                memory.left_foot_world_target = Some(visible_left);
                memory.right_foot_world_target = Some(visible_right);
                memory.left_foot_plant = FootPlant::Acquired(visible_left);
                memory.right_foot_plant = FootPlant::Acquired(visible_right);
            }
            let quickstep_handoff_active = memory.quickstep_handoff.is_pending();
            if memory.quickstep_handoff.is_held() && live_speed > STATIONARY_ANIMATION_SPEED_LIMIT {
                memory.quickstep_handoff = QuickstepContactHandoff::None;
            }
            if quickstep_handoff_active {
                // Preserve the airborne pose through impact while the legs
                // converge to their authored guard length. Once converged, the
                // ordinary follower is free to step for any residual velocity.
                footwork.step = GuardStepState::Stationary {
                    left: visible_left,
                    right: visible_right,
                    next: opposite_guard_foot(skeleton.contact_foot),
                };
            }

            if !quickstep_handoff_active {
                let contact_world = |contact: Vec2, authored: Vec3| {
                    let authored_local = rig_rotation.inverse() * (authored - rig_origin);
                    let target = rig_origin
                        + rig_rotation * Vec3::new(contact.x, authored_local.y, contact.y);
                    if enabled.is_enabled() {
                        terrain.map_or(target, |terrain| {
                            terrain_conformed_guard_target(target, ScenePlanPoint::from_metres(target.xz()).and_then(|point| terrain.height_at(point)))
                        })
                    } else {
                        target
                    }
                };
                let requested = match skeleton.raised_footwork() {
                    GuardFootworkPlan::Uninitialized => GuardStepState::Stationary {
                        left: visible_left,
                        right: visible_right,
                        next: opposite_guard_foot(skeleton.contact_foot),
                    },
                    GuardFootworkPlan::Planted {
                        contacts,
                        next_swing,
                    } => {
                        let authored_contact = |authored: Vec3| {
                            let local = rig_rotation.inverse() * (authored - rig_origin);
                            Vec2::new(local.x, local.z)
                        };
                        let (left_contact, right_contact) =
                            if live_speed <= STATIONARY_ANIMATION_SPEED_LIMIT {
                                // Once translation has stopped, the prepared combat
                                // stance is the canonical footprint. Replicated tap
                                // contacts describe the last moving sample and can
                                // otherwise leave both feet collapsed on one side.
                                (
                                    authored_contact(left_authored),
                                    authored_contact(right_authored),
                                )
                            } else {
                                (contacts.left(), contacts.right())
                            };
                        GuardStepState::Stationary {
                            left: contact_world(left_contact, left_authored),
                            right: contact_world(right_contact, right_authored),
                            next: next_swing,
                        }
                    }
                    GuardFootworkPlan::Stepping(step) => {
                        let contacts = step.contacts();
                        let progress = match step.swing_foot() {
                            LeadFoot::Left => {
                                (skeleton.gait_phase.cycles() - 0.5).rem_euclid(1.0) * 2.0
                            }
                            LeadFoot::Right => skeleton.gait_phase.cycles() * 2.0,
                        }
                        .clamp(0.0, 1.0);
                        let progress = match UnitInterval::from_fraction(progress) {
                            Ok(progress) => progress,
                            Err(error) => {
                                warn!(%error, "guard swing phase rejected");
                                continue;
                            }
                        };
                        match step.swing_foot() {
                            LeadFoot::Left => GuardStepState::LeftSwing {
                                right_support: contact_world(contacts.right(), right_authored),
                                left: GuardSwing {
                                    start: contact_world(step.swing_start(), left_authored),
                                    end: contact_world(step.landing(), left_authored),
                                    progress,
                                },
                            },
                            LeadFoot::Right => GuardStepState::RightSwing {
                                left_support: contact_world(contacts.left(), left_authored),
                                right: GuardSwing {
                                    start: contact_world(step.swing_start(), right_authored),
                                    end: contact_world(step.landing(), right_authored),
                                    progress,
                                },
                            },
                        }
                    }
                };
                if live_speed <= STATIONARY_ANIMATION_SPEED_LIMIT
                    && !footwork.step.initialized()
                    && let (Some(left), Some(right)) = (
                        memory.left_last_rendered_world,
                        memory.right_last_rendered_world,
                    )
                {
                    let fallback = opposite_guard_foot(skeleton.contact_foot);
                    let next = match requested {
                        GuardStepState::Stationary {
                            left: desired_left,
                            right: desired_right,
                            ..
                        } => safer_guard_reacquire_foot(
                            left,
                            right,
                            desired_left,
                            desired_right,
                            fallback,
                        ),
                        _ => fallback,
                    };
                    // Reacquire from the final authored locomotion pose.
                    // The stationary stepper can then move one foot at a
                    // time toward the guard contacts without a one-frame
                    // ownership snap or contact-identity reset.
                    footwork.step = GuardStepState::Stationary { left, right, next };
                }
                footwork.step = if live_speed <= STATIONARY_ANIMATION_SPEED_LIMIT
                    && matches!(requested, GuardStepState::Stationary { .. })
                {
                    match advance_stationary_turn_step(
                        footwork.step,
                        requested,
                        state_delta_seconds,
                    ) {
                        Ok(step) => step,
                        Err(error) => {
                            warn!(%error, "stationary footwork rejected");
                            continue;
                        }
                    }
                } else {
                    requested
                };
            }
            let next_swing = footwork.step.swing_foot();
            if previous_swing.is_some() && previous_swing != next_swing {
                // Match the authoritative contact-sequence convention: the
                // sequence advances when the airborne foot lands (or a
                // skipped presentation sample lands and launches the other
                // foot), not when toe-off begins.
                footwork.step_sequence = footwork.step_sequence.next();
            }

            let Some(mut requested_targets) = footwork.step.visual_targets() else {
                continue;
            };
            // Contact safety outranks swing interpolation when the body catches up.
            let hip_center = (left_upper_snapshot.global.translation()
                + right_upper_snapshot.global.translation())
                * 0.5;
            requested_targets.keep_swing_ahead(skeleton, frame, hip_center);
            let Some(validated) = validate_guard_frame_targets(
                requested_targets,
                [
                    GuardLegGeometry {
                        hip: left_upper_snapshot.global.translation(),
                        maximum_reach: ReachLimit::Extended {
                            upper: left_upper_length,
                            lower: left_lower_length,
                        },
                    },
                    GuardLegGeometry {
                        hip: right_upper_snapshot.global.translation(),
                        maximum_reach: ReachLimit::Extended {
                            upper: right_upper_length,
                            lower: right_lower_length,
                        },
                    },
                ],
                footwork.step.swing_foot(),
            ) else {
                // The local candidate may already contain transformed contacts.
                // Reject it without replacing either prior retained component.
                continue;
            };
            let left_target = validated.left();
            let right_target = validated.right();
            let _adjusted_for_reach = validated.adjusted_for_reach();
            let (left_nominal_support, right_nominal_support) = match footwork.step {
                GuardStepState::LeftSwing { .. } => (false, true),
                GuardStepState::RightSwing { .. } => (true, false),
                GuardStepState::Uninitialized | GuardStepState::Stationary { .. } => (true, true),
            };
            let quickstep_handoff_converged =
                memory
                    .quickstep_handoff
                    .targets()
                    .is_some_and(|(left, right)| {
                        let authored_left = rig_rotation.inverse() * (left_authored - rig_origin);
                        let authored_right = rig_rotation.inverse() * (right_authored - rig_origin);
                        left.distance(authored_left) <= 0.001
                            && right.distance(authored_right) <= 0.001
                    });
            if memory.quickstep_handoff.is_pending() && quickstep_handoff_converged {
                memory.quickstep_handoff.hold();
            }

            let mut airborne_orientation_owned = [true; 2];
            for (leg_index, (upper, lower, foot, target, foot_side, support)) in [
                (
                    left_upper,
                    left_lower,
                    left_foot,
                    left_target,
                    LeadFoot::Left,
                    left_nominal_support,
                ),
                (
                    right_upper,
                    right_lower,
                    right_foot,
                    right_target,
                    LeadFoot::Right,
                    right_nominal_support,
                ),
            ]
            .into_iter()
            .enumerate()
            {
                let left = foot_side == LeadFoot::Left;
                let Some((upper_snapshot, lower_snapshot, foot_snapshot)) =
                    snapshot_chain(upper, lower, foot, &parents, &transforms.p0())
                else {
                    continue;
                };
                let LimbSegments {
                    upper: upper_length,
                    lower: lower_length,
                } = match LimbSegments::measure(
                    upper_snapshot.global.translation(),
                    lower_snapshot.global.translation(),
                    foot_snapshot.global.translation(),
                ) {
                    Ok(segments) => segments,
                    Err(error) => {
                        warn!(%error, "IK segment measurement rejected");
                        continue;
                    }
                };
                let side = anatomical_side(
                    rig_rotation,
                    rig_origin,
                    upper_snapshot.global.translation(),
                    foot_side,
                );
                let quickstep_bend = if left {
                    memory.left_leg
                } else {
                    memory.right_leg
                }
                .map(|bend| pole_to_world(rig_rotation, bend));
                let remembered = if quickstep_handoff_active {
                    quickstep_bend
                } else if left {
                    footwork.left_knee_bend_world.or(quickstep_bend)
                } else {
                    footwork.right_knee_bend_world.or(quickstep_bend)
                };
                let previous_end_direction = if left {
                    footwork.left_end_direction
                } else {
                    footwork.right_end_direction
                };
                let canonical_pole = canonical_knee_pole(side);
                let canonical_world = pole_to_world(rig_rotation, canonical_pole);
                let foot_facing = rendered_foot_facing(
                    rig,
                    foot_side,
                    foot_snapshot.global.translation(),
                    &parents,
                    &transforms.p0(),
                );
                let pole = stabilized_knee_pole(
                    remembered,
                    previous_end_direction,
                    upper_snapshot.global.translation(),
                    lower_snapshot.global.translation(),
                    target,
                    canonical_world,
                    foot_facing,
                )
                .unwrap_or(canonical_world);
                let pole = constrain_rendered_leg_pole(
                    rig,
                    foot_side,
                    upper_snapshot.global.translation(),
                    foot_snapshot.global.translation(),
                    target,
                    pole,
                    &parents,
                    &transforms.p0(),
                );
                if let Some(solution) = solve_two_bone_with_reach(
                    TwoBoneChain::new(
                        upper_snapshot.global.translation(),
                        lower_snapshot.global.translation(),
                        foot_snapshot.global.translation(),
                        upper_length,
                        lower_length,
                        pole,
                    ),
                    target,
                    if support {
                        ReachLimit::Extended {
                            upper: upper_length,
                            lower: lower_length,
                        }
                    } else {
                        maximum_reach(upper_length, lower_length)
                    },
                ) {
                    apply_two_bone_solution(
                        upper,
                        lower,
                        foot,
                        solution,
                        &parents,
                        &mut transforms,
                    );
                    let bend = (solution.knee - upper_snapshot.global.translation())
                        .reject_from_normalized(solution.end_direction);
                    if !state_delta_seconds.is_zero()
                        && let Some(valid) = bend.try_normalize()
                    {
                        if left {
                            memory.left_leg = Some(pole_to_owner(rig_rotation, valid));
                            footwork.left_knee_bend_world = Some(valid);
                            footwork.left_end_direction = Some(solution.end_direction);
                        } else {
                            memory.right_leg = Some(pole_to_owner(rig_rotation, valid));
                            footwork.right_knee_bend_world = Some(valid);
                            footwork.right_end_direction = Some(solution.end_direction);
                        }
                    }
                }
                let rendered_ankle = snapshot(foot, &parents, &transforms.p0())
                    .map(|rendered| rendered.global.translation());
                let support_observation =
                    observed_support_weight(support, rendered_ankle, terrain, *enabled);
                let reported_support = support_observation.is_some_and(terrain_leg_has_support);
                airborne_orientation_owned[leg_index] = !reported_support;
                if enabled.is_enabled()
                    && reported_support
                    && let Some(terrain) = terrain
                    && let Some(normal) = ScenePlanPoint::from_metres(target.xz()).and_then(|point| terrain.normal_at(point))
                    && let Some(sole_axis) = rig.sole_axis(foot_side)
                {
                    let cached_chain = if left {
                        memory.left_rotation_chain
                    } else {
                        memory.right_rotation_chain
                    };
                    if evaluation_advances || cached_chain.is_none() {
                        align_foot_to_slope(foot, sole_axis, normal, &parents, &mut transforms);
                    }
                }
                if left {
                    if evaluation_advances && let Some(observed) = support_observation {
                        memory.left_contact_orientation_blend_active =
                            update_contact_orientation_blend(
                                memory.left_contact_orientation_blend_active,
                                memory.left_support_weight,
                                observed,
                            );
                    }
                    let visible_target = rendered_ankle.unwrap_or(target);
                    footwork.left_solve_target = Some(visible_target);
                    memory.left_foot_world_target = Some(visible_target);
                    memory.left_support_weight = support_observation;
                    if let Some(observed) = support_observation {
                        footwork.left_support_weight = observed;
                    }
                } else {
                    if evaluation_advances && let Some(observed) = support_observation {
                        memory.right_contact_orientation_blend_active =
                            update_contact_orientation_blend(
                                memory.right_contact_orientation_blend_active,
                                memory.right_support_weight,
                                observed,
                            );
                    }
                    let visible_target = rendered_ankle.unwrap_or(target);
                    footwork.right_solve_target = Some(visible_target);
                    memory.right_foot_world_target = Some(visible_target);
                    memory.right_support_weight = support_observation;
                    if let Some(observed) = support_observation {
                        footwork.right_support_weight = observed;
                    }
                }
            }
            let raised_orientation_handoff = [
                memory.left_foot_orientation_world.is_none()
                    && memory.left_last_rendered_foot_rotation_world.is_some(),
                memory.right_foot_orientation_world.is_none()
                    && memory.right_last_rendered_foot_rotation_world.is_some(),
            ];
            if let Err(error) = finalize_leg_rotation_chains(
                rig,
                skeleton,
                speed,
                rig_rotation,
                &mut memory,
                evaluation_advances,
                state_delta_seconds,
                airborne_orientation_owned,
                raised_orientation_handoff,
                &parents,
                &mut transforms,
            ) {
                warn!(%error, ?owner, "Rejected final leg rotation");
                continue;
            }
            // Classify support and retain handoff targets only after the final
            // cached-chain/orientation seam. This is the same local-transform
            // state that transform propagation exposes to viewer telemetry.
            for (foot, left, nominal_support) in [
                (left_foot, true, left_nominal_support),
                (right_foot, false, right_nominal_support),
            ] {
                let Some(rendered) = snapshot(foot, &parents, &transforms.p0()) else {
                    continue;
                };
                let ankle = rendered.global.translation();
                let support_observation =
                    observed_support_weight(nominal_support, Some(ankle), terrain, *enabled);
                if left {
                    footwork.left_solve_target = Some(ankle);
                    if let Some(observed) = support_observation {
                        footwork.left_support_weight = observed;
                    }
                    memory.left_foot_world_target = Some(ankle);
                    memory.left_support_weight = support_observation;
                } else {
                    footwork.right_solve_target = Some(ankle);
                    if let Some(observed) = support_observation {
                        footwork.right_support_weight = observed;
                    }
                    memory.right_foot_world_target = Some(ankle);
                    memory.right_support_weight = support_observation;
                }
            }
            if let Err(error) = footwork.validate_targets() {
                warn!(%error, "Raised footwork rejected before publication");
                continue;
            }
            if let Err(error) = memory.publish(owner, &mut ik_states, &mut commands) {
                warn!(%error, "IK targets rejected before publication");
                continue;
            }
            if let Ok(mut state) = raised_states.get_mut(owner) {
                *state = footwork;
            } else {
                commands.entity(owner).insert(footwork);
            }
            continue;
        }

        if !enabled.is_enabled()
            && terrain_blend.fraction() <= 0.001
            && memory.settle.is_none()
            && !memory.left_release_active
            && !memory.right_release_active
        {
            memory.finish_terrain_release();
            if let Err(error) = memory.publish(owner, &mut ik_states, &mut commands) {
                warn!(%error, "IK targets rejected before publication");
                continue;
            }
            continue;
        }
        let Some(terrain) = terrain else {
            continue;
        };
        prepare_slope_rotation_cache(&mut memory, SlopeAlignmentMode::Ordinary);
        let mut desired_hip_shift = 0.0_f32;
        let mut settle_contact_reached = false;
        for (upper_role, lower_role, foot_role, weight, foot_side) in legs {
            let (Some(&upper), Some(&lower), Some(&foot)) = (
                rig.get(&upper_role),
                rig.get(&lower_role),
                rig.get(&foot_role),
            ) else {
                continue;
            };
            let Some((upper_snapshot, lower_snapshot, foot_snapshot)) =
                snapshot_chain(upper, lower, foot, &parents, &transforms.p0())
            else {
                continue;
            };
            let position = foot_snapshot.global.translation();
            if let Some(height) = ScenePlanPoint::from_metres(position.xz()).and_then(|point| terrain.height_at(point)) {
                let desired_ankle = height.metres() + measured_ankle_sole_offset_metres().metres();
                desired_hip_shift = desired_hip_shift
                    .min(((desired_ankle - position.y) * weight.fraction()).clamp(-0.18, 0.0));
            }
            let plant = match foot_side {
                LeadFoot::Left => memory.left_foot_plant.target(),
                LeadFoot::Right => memory.right_foot_plant.target(),
            };
            let planned = match foot_side {
                LeadFoot::Left => memory.left_planned_contact,
                LeadFoot::Right => memory.right_planned_contact,
            };
            let acquired = match foot_side {
                LeadFoot::Left => memory.left_foot_plant.is_acquired(),
                LeadFoot::Right => memory.right_foot_plant.is_acquired(),
            };
            let planned_phase_start = match foot_side {
                LeadFoot::Left => memory.left_planned_contact_phase_start,
                LeadFoot::Right => memory.right_planned_contact_phase_start,
            };
            let run = profile.gait == LocomotionGait::Run;
            let planned_weight = if run && !acquired && (plant.is_some() || planned.is_some()) {
                let phase_to_contact = skeleton.gait_phase.until_contact(foot_side);
                match run_contact_approach_progress(
                    phase_to_contact,
                    planned_phase_start.unwrap_or(CycleDelta::from_unit_cycles(
                        ik_tuning().run_contact_approach_phase,
                    )),
                    contact_ready_phase,
                ) {
                    Ok(progress) => progress,
                    Err(error) => {
                        warn!(%error, "contact approach rejected");
                        continue;
                    }
                }
            } else {
                UnitInterval::ZERO
            };
            let Some(horizontal_target) = plant.or(planned) else {
                continue;
            };
            let reach_weight = if settle_is_terminal(&memory) && plant.is_some() {
                // A completed stop owns both final contacts. Idle has no raw
                // gait support weight, but the shared rig root must continue
                // descending until both analytic chains can actually reach
                // those contacts; otherwise one leg can remain frozen above
                // the ground forever with settle progress pinned at one.
                UnitInterval::ONE
            } else if plant.is_some() {
                weight.max(planned_weight)
            } else {
                planned_weight
            };
            // A remembered plant is world-space. Do not reproject it through
            // the rotating/moving anatomical corridor every frame: that made
            // a visibly planted foot skate during turns. New contacts are
            // constrained when acquired, and reach limiting below remains the
            // only reason an established plant may yield.
            let Some(horizontal_point) = ScenePlanPoint::from_metres(horizontal_target.xz()) else {
                continue;
            };
            let Some(height) = terrain.height_at(horizontal_point) else {
                continue;
            };
            let target_y = height.metres() + measured_ankle_sole_offset_metres().metres();
            let LimbSegments {
                upper: upper_length,
                lower: lower_length,
            } = match LimbSegments::measure(
                upper_snapshot.global.translation(),
                lower_snapshot.global.translation(),
                foot_snapshot.global.translation(),
            ) {
                Ok(segments) => segments,
                Err(error) => {
                    warn!(%error, "IK segment measurement rejected");
                    continue;
                }
            };
            let reach = terrain_maximum_reach(upper_length, lower_length);
            let reach_shift = match reach.terrain_pelvis_shift(
                upper_snapshot.global.translation(),
                horizontal_target.with_y(target_y),
                reach_weight,
            ) {
                Ok(shift) => shift,
                Err(error) => {
                    warn!(%error, "terrain pelvis reach rejected");
                    continue;
                }
            };
            desired_hip_shift = desired_hip_shift.min(reach_shift.metres());
        }
        desired_hip_shift *= terrain_blend.fraction();
        if profile.gait == LocomotionGait::Run {
            // Anticipate only the reach needed by the frozen run contact. The
            // bounded contact-phase drop reinforces the two existing minima;
            // it cannot add the earlier free-running terrain wave or move the
            // authoritative controller.
            desired_hip_shift = desired_hip_shift.clamp(
                -ik_tuning()
                    .run_maximum_planned_reach_pelvis_drop_metres
                    .metres(),
                0.0,
            );
        }
        let terminal_root_correction =
            settle_is_terminal(&memory) && memory.terminal_contacts.is_prepared();
        let hip_shift = match SignedLength::from_metres(desired_hip_shift)
            .map_err(|_| AnimationValueError::Overflow {
                quantity: AnimationQuantity::HeightOffset,
            })
            .and_then(|desired| {
                memory.advance_terrain_pelvis(
                    desired,
                    terminal_root_correction,
                    memory_was_missing,
                    profile.gait,
                    state_delta_seconds,
                )
            }) {
            Ok(shift) => shift,
            Err(error) => {
                warn!(%error, "terrain pelvis correction rejected");
                continue;
            }
        };
        if hip_shift.metres() < -0.001 {
            // MHR's anatomical `root` owns the spine and both thigh chains.
            // Correct its `body_world` parent so every cached knee pole and
            // local chain sees one coherent transform. Translating the pelvis
            // and two thigh locals independently inverted the knee hemisphere.
            if let Some(&bone) = rig.get(&BoneRole::Root) {
                let local_delta = parents
                    .get(bone)
                    .ok()
                    .and_then(|parent| {
                        transforms
                            .p0()
                            .compute_global_transform(parent.parent())
                            .ok()
                    })
                    .map(|parent| {
                        parent
                            .affine()
                            .inverse()
                            .transform_vector3(Vec3::Y * hip_shift.metres())
                    })
                    .unwrap_or(Vec3::Y * hip_shift.metres());
                if local_delta.is_finite()
                    && let Ok(mut transform) = transforms.p1().get_mut(bone)
                {
                    if terminal_root_correction {
                        if let Some(reach) = memory.terminal_contacts.prepared_mut() {
                            let base = *reach.base_translation.get_or_insert(transform.translation);
                            transform.translation = base + local_delta;
                        }
                    } else {
                        transform.translation += local_delta;
                    }
                }
            }
        }
        let mut airborne_orientation_owned = [false; 2];
        let mut airborne_just_released = [false; 2];
        for (leg_index, (upper_role, lower_role, foot_role, weight, foot_side)) in
            legs.into_iter().enumerate()
        {
            let left = foot_side == LeadFoot::Left;
            let mut weight = weight;
            let settle_support_owned = memory
                .settle
                .is_some_and(|settle| settle.support_foot == foot_side);
            if settle_support_owned {
                // The chosen settle support owns a retained footprint even if
                // stop began in flight. Keep that logical solve path stable
                // while its follower approaches contact; allowing the raw
                // gait lobe to drop to zero routes it through ordinary swing,
                // discards the toe floor, and prevents acquisition.
                weight = UnitInterval::ONE;
            }
            let terminal_contact_owned =
                settle_is_terminal(&memory) && memory.terminal_contacts.is_prepared();
            let raw_nominal_weight = raw_run_support
                .map(|(left_raw, right_raw)| if left { left_raw } else { right_raw })
                .unwrap_or(weight);
            let (Some(&upper), Some(&lower), Some(&foot)) = (
                rig.get(&upper_role),
                rig.get(&lower_role),
                rig.get(&foot_role),
            ) else {
                continue;
            };
            let Some((upper_snapshot, lower_snapshot, foot_snapshot)) =
                snapshot_chain(upper, lower, foot, &parents, &transforms.p0())
            else {
                continue;
            };
            let foot_position = foot_snapshot.global.translation();
            let toe_position = rig
                .get(if left {
                    &BoneRole::ToeLeft
                } else {
                    &BoneRole::ToeRight
                })
                .and_then(|toe| snapshot(*toe, &parents, &transforms.p0()))
                .map(|snapshot| snapshot.global.translation());
            let rendered_ankle_and_toe = if left {
                memory
                    .left_last_rendered_world
                    .zip(memory.left_last_rendered_toe_world)
            } else {
                memory
                    .right_last_rendered_world
                    .zip(memory.right_last_rendered_toe_world)
            }
            .or_else(|| toe_position.map(|toe| (foot_position, toe)));
            if left {
                memory.left_authored_world_target = Some(foot_position);
            } else {
                memory.right_authored_world_target = Some(foot_position);
            }
            let mut plant = if left {
                memory.left_foot_plant.target()
            } else {
                memory.right_foot_plant.target()
            };
            let settle_support_plant = settle_support_owned.then_some(plant).flatten();
            let side = anatomical_side(
                rig_rotation,
                rig_origin,
                upper_snapshot.global.translation(),
                foot_side,
            );
            let LimbSegments {
                upper: upper_length,
                lower: lower_length,
            } = match LimbSegments::measure(
                upper_snapshot.global.translation(),
                lower_snapshot.global.translation(),
                foot_position,
            ) {
                Ok(segments) => segments,
                Err(error) => {
                    warn!(%error, "IK segment measurement rejected");
                    continue;
                }
            };
            if terminal_contact_owned {
                // Terminal dual-contact ownership is a dedicated state. It
                // must bypass every ordinary phase, discontinuity, reach-
                // release, track-constrain, and replan mutation below: those
                // transitions can rewrite a frozen idle contact one sample
                // before completion and snap the rendered chain.
                let (logical_weight, terminal_plant) =
                    terminal_contact_solve_ownership(true, weight, plant);
                let Some(frozen_plant) = terminal_plant else {
                    continue;
                };
                let Some(height) = ScenePlanPoint::from_metres(frozen_plant.xz()).and_then(|point| terrain.height_at(point)) else {
                    continue;
                };
                let target =
                    frozen_plant.with_y(height.metres() + measured_ankle_sole_offset_metres().metres());
                let canonical_world = pole_to_world(rig_rotation, canonical_knee_pole(side));
                let (remembered_pole, previous_end_direction) = if left {
                    (
                        memory.left_terrain_pole_world,
                        memory.left_terrain_end_direction,
                    )
                } else {
                    (
                        memory.right_terrain_pole_world,
                        memory.right_terrain_end_direction,
                    )
                };
                let next_end_direction =
                    (target - upper_snapshot.global.translation()).normalize_or_zero();
                let pole = transported_terrain_pole(
                    remembered_pole,
                    previous_end_direction,
                    next_end_direction,
                    canonical_world,
                )
                .unwrap_or(canonical_world);
                let pole = constrain_rendered_leg_pole(
                    rig,
                    foot_side,
                    upper_snapshot.global.translation(),
                    foot_position,
                    target,
                    pole,
                    &parents,
                    &transforms.p0(),
                );
                if let Some(solution) = solve_two_bone_with_reach(
                    TwoBoneChain::new(
                        upper_snapshot.global.translation(),
                        lower_snapshot.global.translation(),
                        foot_position,
                        upper_length,
                        lower_length,
                        pole,
                    ),
                    target,
                    terrain_maximum_reach(upper_length, lower_length),
                ) {
                    apply_two_bone_solution(
                        upper,
                        lower,
                        foot,
                        solution,
                        &parents,
                        &mut transforms,
                    );
                    if !state_delta_seconds.is_zero() {
                        let bend = (solution.knee - upper_snapshot.global.translation())
                            .reject_from_normalized(solution.end_direction)
                            .try_normalize();
                        if left {
                            if let Some(bend) = bend {
                                memory.left_terrain_pole_world = Some(bend);
                            }
                            memory.left_terrain_end_direction = Some(solution.end_direction);
                        } else {
                            if let Some(bend) = bend {
                                memory.right_terrain_pole_world = Some(bend);
                            }
                            memory.right_terrain_end_direction = Some(solution.end_direction);
                        }
                    }
                }
                if let Some(normal) = ScenePlanPoint::from_metres(target.xz()).and_then(|point| terrain.normal_at(point))
                    && let Some(sole_axis) = rig.sole_axis(foot_side)
                {
                    align_foot_to_slope(foot, sole_axis, normal, &parents, &mut transforms);
                }
                let owner_target = rig_rotation.inverse() * (target - rig_origin);
                if left {
                    memory.left_foot_plant = FootPlant::Acquiring(frozen_plant);
                    memory.left_foot_target = Some(owner_target);
                    memory.left_foot_world_target = Some(target);
                    memory.left_support_weight = Some(UnitInterval::ZERO);
                    memory.left_transition_support_weight = Some(logical_weight);
                    memory.left_release_active = false;
                    memory.left_release_target = None;
                } else {
                    memory.right_foot_plant = FootPlant::Acquiring(frozen_plant);
                    memory.right_foot_target = Some(owner_target);
                    memory.right_foot_world_target = Some(target);
                    memory.right_support_weight = Some(UnitInterval::ZERO);
                    memory.right_transition_support_weight = Some(logical_weight);
                    memory.right_release_active = false;
                    memory.right_release_target = None;
                }
                airborne_orientation_owned[leg_index] = true;
                continue;
            }
            let plant_acquired = if left {
                memory.left_foot_plant.is_acquired()
            } else {
                memory.right_foot_plant.is_acquired()
            };
            let exhausted = if left {
                memory.left_support_exhausted_until_flight
            } else {
                memory.right_support_exhausted_until_flight
            };
            let retained_plan = if left {
                memory.left_planned_contact
            } else {
                memory.right_planned_contact
            };
            // A run can begin inside a raw support lobe whose foot was never
            // rendered at contact (notably the hard-start fixture). Without a
            // preceding swing plan there is no truthful footprint to acquire;
            // jumping to a freshly predicted plant moves the whole chain. Skip
            // the remainder of that lobe and begin normally after true flight.
            // True raw flight clears the preceding toe-off latch before any
            // effective-support suppression. Plan state must not be able to
            // keep a latch alive across a complete same-foot cycle.
            let exhausted = exhausted_latch_after_raw_cadence(exhausted, raw_nominal_weight);
            let exhausted = exhausted
                || unplanned_run_support_requires_flight(
                    profile.gait,
                    speed,
                    weight,
                    plant_acquired,
                    retained_plan,
                );
            let (mut next_exhausted, mut effective_weight) =
                support_after_exhausted_lobe(exhausted, weight);
            if run_plan_is_on_rising_support(
                profile.gait,
                skeleton.gait_phase,
                foot_side,
                profile.support_phase_radius,
                raw_nominal_weight,
                retained_plan,
                plant_acquired,
            ) {
                next_exhausted = false;
                effective_weight = raw_nominal_weight;
            }
            if left {
                memory.left_support_exhausted_until_flight = next_exhausted;
            } else {
                memory.right_support_exhausted_until_flight = next_exhausted;
            }
            let release_now = run_is_at_support_exit(
                skeleton.gait_phase,
                foot_side,
                profile.support_phase_radius,
            );
            let (toe_off_started, toe_off_weight) = run_toe_off_support_weight(
                profile.gait,
                run_retained_support_through_lobe_edge(
                    profile.gait,
                    effective_weight,
                    plant_acquired && plant.is_some(),
                    release_now,
                ),
                plant_acquired && plant.is_some(),
                release_now,
            );
            weight = toe_off_weight;
            // Commit the new transition after the prior latch's flight-clear
            // result. Otherwise an abrupt 1 -> 0 profile clears a latch in the
            // same evaluation that created it, allowing next-tick reentry.
            if toe_off_started {
                if left {
                    memory.left_support_exhausted_until_flight = true;
                } else {
                    memory.right_support_exhausted_until_flight = true;
                }
            }
            if exhausted && next_exhausted {
                plant = None;
            }
            if plant_acquired
                && let Some(retained_plant) = plant
                && let Some(height) = ScenePlanPoint::from_metres(retained_plant.xz()).and_then(|point| terrain.height_at(point))
            {
                let retained_target = Vec3::new(
                    retained_plant.x,
                    height.metres() + measured_ankle_sole_offset_metres().metres(),
                    retained_plant.z,
                );
                let Some(reachable_target) = constrain_target_to_reach(
                    retained_target,
                    upper_snapshot.global.translation(),
                    terrain_maximum_reach(upper_length, lower_length),
                ) else {
                    continue;
                };
                if retained_plant_requires_release(retained_target, reachable_target) {
                    // A support footprint is either stationary or released.
                    // Do not preserve nominal support by skating the plant as
                    // the hip outruns its reachable region.
                    weight = UnitInterval::ZERO;
                    plant = None;
                    if left {
                        memory.left_support_exhausted_until_flight = true;
                    } else {
                        memory.right_support_exhausted_until_flight = true;
                    }
                }
            }
            if settle_support_owned {
                // Run cadence/exhaustion is evaluated above for ordinary
                // locomotion and may suppress or clear an unacquired plant.
                // Stop capture has its own completion-driven ownership: put
                // the selected footprint back and keep solving it until the
                // propagated contact becomes truthful.
                weight = UnitInterval::ONE;
                plant = settle_support_plant;
                if left {
                    memory.left_support_exhausted_until_flight = false;
                } else {
                    memory.right_support_exhausted_until_flight = false;
                }
            }
            let opposite_acquired = if left {
                memory.right_foot_plant.is_acquired()
                    && memory.right_foot_plant.target().is_some()
                    && memory
                        .right_support_weight
                        .is_some_and(terrain_leg_has_support)
            } else {
                memory.left_foot_plant.is_acquired()
                    && memory.left_foot_plant.target().is_some()
                    && memory
                        .left_support_weight
                        .is_some_and(terrain_leg_has_support)
            };
            weight = coordinated_support_weight(
                profile.gait,
                weight,
                plant_acquired && plant.is_some(),
                opposite_acquired,
            );
            if ordinary_plant_requires_clear(weight, plant_acquired, plant, foot_position) {
                plant = None;
            }
            if !terrain_leg_has_support(weight) {
                airborne_orientation_owned[leg_index] = true;
                // An airborne foot is never retained at its old plant. During
                // ordinary locomotion it follows authored FK immediately;
                // during a stop it follows an explicit clearance arc toward
                // the balance-restoring contact.
                let settle_swing = memory
                    .settle
                    .filter(|settle| settle.support_foot != foot_side);
                if settle_swing.is_none() {
                    let gait = profile.gait;
                    let run_airborne_budget = uses_run_airborne_motion_budget(
                        gait,
                        memory.effective_planar_speed(planar_speed),
                    );
                    let airborne_budget_gait = if run_airborne_budget {
                        LocomotionGait::Run
                    } else {
                        gait
                    };
                    let phase_to_contact = skeleton.gait_phase.until_contact(foot_side);
                    let mut retained_contact = if left {
                        memory.left_planned_contact
                    } else {
                        memory.right_planned_contact
                    };
                    let mut retained_start = if left {
                        memory.left_planned_contact_start
                    } else {
                        memory.right_planned_contact_start
                    };
                    let mut retained_phase_start = if left {
                        memory.left_planned_contact_phase_start
                    } else {
                        memory.right_planned_contact_phase_start
                    };
                    let previous_transition_support = if left {
                        memory.left_transition_support_weight
                    } else {
                        memory.right_transition_support_weight
                    };
                    let failed_acquisition_lobe_exited = acquisition_lobe_exited_without_contact(
                        retained_contact,
                        plant_acquired,
                        previous_transition_support,
                        weight,
                    );
                    if failed_acquisition_lobe_exited {
                        clear_planned_contact_metadata(
                            &mut retained_contact,
                            &mut retained_start,
                            &mut retained_phase_start,
                        );
                        if left {
                            clear_planned_contact_metadata(
                                &mut memory.left_planned_contact,
                                &mut memory.left_planned_contact_start,
                                &mut memory.left_planned_contact_phase_start,
                            );
                        } else {
                            clear_planned_contact_metadata(
                                &mut memory.right_planned_contact,
                                &mut memory.right_planned_contact_start,
                                &mut memory.right_planned_contact_phase_start,
                            );
                        }
                    }
                    let propagated_visible_target = if left {
                        memory
                            .left_last_rendered_world
                            .or(memory.left_foot_world_target)
                    } else {
                        memory
                            .right_last_rendered_world
                            .or(memory.right_foot_world_target)
                    };
                    let (was_releasing, previous_owner_target) = if left {
                        (
                            memory.left_release_active,
                            run_previous_owner_target(
                                airborne_budget_gait,
                                memory.left_last_rendered_owner,
                                memory.left_foot_target,
                            ),
                        )
                    } else {
                        (
                            memory.right_release_active,
                            run_previous_owner_target(
                                airborne_budget_gait,
                                memory.right_last_rendered_owner,
                                memory.right_foot_target,
                            ),
                        )
                    };
                    let prior_visible_target = run_plan_visible_start(
                        airborne_budget_gait,
                        retained_contact.is_none(),
                        was_releasing,
                        previous_owner_target,
                        rig_origin,
                        rig_rotation,
                        propagated_visible_target,
                    );
                    let approach_window = if profile.gait == LocomotionGait::Run {
                        CycleDelta::from_unit_cycles(ik_tuning().run_contact_approach_phase)
                    } else {
                        // Existing authored walk-acquisition width, in cycles.
                        match CycleDelta::from_cycles(0.12) {
                            Ok(window) => window,
                            Err(error) => {
                                warn!(%error, "walk contact width rejected");
                                continue;
                            }
                        }
                    };
                    let support_lobe_exhausted = if left {
                        memory.left_support_exhausted_until_flight
                    } else {
                        memory.right_support_exhausted_until_flight
                    };
                    let planned_contact = run_planned_contact_allowed(
                        support_lobe_exhausted,
                        phase_to_contact,
                        approach_window,
                    )
                    .then(|| {
                        if let Some(contact) = retained_contact {
                            return Some(contact);
                        }
                        let candidate = ordinary_contact_target(
                            rig_origin,
                            rig_rotation,
                            projected_body_center(rig, &transforms.p0()).unwrap_or(rig_origin),
                            planar_velocity,
                            profile.step_distance,
                            phase_to_contact,
                            side,
                        );
                        if profile.gait == LocomotionGait::Run {
                            reachable_run_contact_target(
                                candidate,
                                upper_snapshot.global.translation(),
                                planar_velocity,
                                profile.step_distance,
                                phase_to_contact,
                                contact_ready_phase,
                                terrain_maximum_reach(upper_length, lower_length),
                                |xz| terrain.height_at(xz),
                            )
                        } else {
                            Some(candidate)
                        }
                    })
                    .flatten()
                    .filter(|_| ordinary_lowered);
                    let planned_start = planned_contact.map(|_| {
                        planned_contact_start(retained_start, prior_visible_target, foot_position)
                    });
                    let planned_contact = planned_contact.map(|contact| {
                        if profile.gait == LocomotionGait::Run
                            && late_run_plan_requires_bound(retained_contact, phase_to_contact)
                        {
                            bound_late_run_contact(
                                planned_start.unwrap_or(foot_position),
                                contact,
                                speed,
                                profile.step_distance,
                                phase_to_contact,
                                contact_ready_phase,
                            )
                        } else {
                            contact
                        }
                    });
                    let planned_phase_start =
                        planned_contact.map(|_| retained_phase_start.unwrap_or(phase_to_contact));
                    if left {
                        memory.left_planned_contact = planned_contact;
                        memory.left_planned_contact_start = planned_start;
                        memory.left_planned_contact_phase_start = planned_phase_start;
                    } else {
                        memory.right_planned_contact = planned_contact;
                        memory.right_planned_contact_start = planned_start;
                        memory.right_planned_contact_phase_start = planned_phase_start;
                    }
                    let planned_progress = if profile.gait == LocomotionGait::Run {
                        match run_contact_approach_progress(
                            phase_to_contact,
                            planned_phase_start.unwrap_or(approach_window),
                            contact_ready_phase,
                        ) {
                            Ok(progress) => progress,
                            Err(error) => {
                                warn!(%error, "contact approach rejected");
                                continue;
                            }
                        }
                    } else {
                        match UnitInterval::from_fraction(smoothstep(
                            approach_window.cycles(),
                            0.0,
                            phase_to_contact.cycles(),
                        )) {
                            Ok(progress) => progress,
                            Err(error) => {
                                warn!(%error, "walk contact approach rejected");
                                continue;
                            }
                        }
                    };
                    let mut desired_target =
                        planned_contact.map_or(foot_position, |mut contact| {
                            if let Some(height) = ScenePlanPoint::from_metres(contact.xz()).and_then(|point| terrain.height_at(point)) {
                                contact.y = height.metres() + measured_ankle_sole_offset_metres().metres();
                            }
                            planned_start
                                .unwrap_or(foot_position)
                                .lerp(contact, planned_progress.fraction())
                        });
                    if profile.gait == LocomotionGait::Run {
                        desired_target = match conform_run_swing_target(
                            terrain,
                            desired_target,
                            phase_to_contact,
                            planned_contact.map(|_| planned_progress),
                        ) {
                            Ok(target) => target,
                            Err(error) => {
                                warn!(%error, "IK geometry rejected");
                                continue;
                            }
                        };
                    }
                    let desired_owner_target =
                        rig_rotation.inverse() * (desired_target - rig_origin);
                    let (
                        previous_owner_target,
                        previous_world_target,
                        previous_support,
                        was_releasing,
                        previous_goal,
                    ) = if left {
                        (
                            run_previous_owner_target(
                                airborne_budget_gait,
                                memory.left_last_rendered_owner,
                                memory.left_foot_target,
                            ),
                            memory.left_foot_world_target,
                            memory.left_transition_support_weight,
                            memory.left_release_active,
                            memory.left_release_target,
                        )
                    } else {
                        (
                            run_previous_owner_target(
                                airborne_budget_gait,
                                memory.right_last_rendered_owner,
                                memory.right_foot_target,
                            ),
                            memory.right_foot_world_target,
                            memory.right_transition_support_weight,
                            memory.right_release_active,
                            memory.right_release_target,
                        )
                    };
                    // Support loss releases in owner space at a bounded speed.
                    // This remains a purely airborne solve: there is no plant,
                    // terrain projection, or clearance floor. Once converged,
                    // authored FK owns the swing again until final acquisition.
                    let just_released = previous_support.is_some_and(terrain_leg_has_support);
                    let run_release_edge = run_release_edge(just_released, toe_off_started);
                    airborne_just_released[leg_index] = run_release_edge;
                    let (mut owner_target, next_release_goal) = if run_release_edge {
                        // The authored FK foot may be nearly a metre from the
                        // world plant on the first release sample. Begin from
                        // the preceding visible target and defer movement to
                        // the bounded release follower instead of rebuilding
                        // the whole chain against that authored endpoint.
                        let owner_target = release_start_owner_target(
                            airborne_budget_gait,
                            previous_owner_target,
                            previous_world_target,
                            rig_origin,
                            rig_rotation,
                            desired_owner_target,
                        );
                        (owner_target, Some(desired_owner_target))
                    } else if planned_contact.is_some() {
                        // A predicted touchdown is stationary in world space.
                        // Run's frozen-start Hermite trajectory already owns
                        // continuity and reaches support entry exactly. Feeding
                        // it through the old retained-target follower starts a
                        // new swing from the previous cycle's stale endpoint.
                        let world_target = if profile.gait == LocomotionGait::Run {
                            desired_target
                        } else {
                            advance_foot_target_at_speed(
                                previous_world_target,
                                desired_target,
                                state_delta_seconds,
                                target_speeds.release,
                            )
                        };
                        let owner_target = rig_rotation.inverse() * (world_target - rig_origin);
                        let next = (world_target.distance_squared(desired_target) > 0.000001)
                            .then_some(desired_owner_target);
                        (owner_target, next)
                    } else {
                        let needs_release = was_releasing
                            || previous_support.is_some_and(|support| support.fraction() > 0.5)
                            || previous_owner_target.is_some_and(|previous| {
                                previous.distance(desired_owner_target)
                                    > target_speeds.release.metres_per_second()
                                        * state_delta_seconds.seconds()
                                        + 0.001
                            });
                        let release_goal = if was_releasing {
                            previous_goal.unwrap_or(desired_owner_target)
                        } else {
                            desired_owner_target
                        };
                        let owner_target = if needs_release {
                            advance_foot_target_at_speed(
                                previous_owner_target,
                                release_goal,
                                state_delta_seconds,
                                target_speeds.release,
                            )
                        } else {
                            desired_owner_target
                        };
                        let reached_goal = owner_target.distance_squared(release_goal) <= 0.000001;
                        let next = if reached_goal
                            && owner_target.distance_squared(desired_owner_target) > 0.000001
                        {
                            Some(desired_owner_target)
                        } else if reached_goal {
                            None
                        } else {
                            Some(release_goal)
                        };
                        (owner_target, next)
                    };
                    let mut target = rig_origin + rig_rotation * owner_target;
                    if run_airborne_budget && run_release_edge {
                        // On flat ground the owner-transported point is already
                        // feasible and remains unchanged. Uphill, transporting
                        // the full root delta can raise terrain plus the 5 cm
                        // flight floor beyond the 9 cm 3D budget. Aim back
                        // toward the prior visible world plant so the joint
                        // terrain/budget projection below can retain only the
                        // feasible fraction of root transport.
                        target = previous_world_target.unwrap_or(target);
                    }
                    if run_airborne_budget && let Some(height) = ScenePlanPoint::from_metres(target.xz()).and_then(|point| terrain.height_at(point)) {
                        let contact_reachable = run_contact_within_follower_step(
                            previous_owner_target,
                            target,
                            rig_origin,
                            rig_rotation,
                            state_delta_seconds,
                        );
                        let descent = run_support_descent(
                            airborne_budget_gait,
                            skeleton.gait_phase,
                            foot_side,
                            profile.support_phase_radius,
                            raw_nominal_weight,
                            contact_reachable
                                && run_contact_within_leg_reach(
                                    target,
                                    upper_snapshot.global.translation(),
                                    terrain_maximum_reach(upper_length, lower_length),
                                ),
                        );
                        let clearance = match run_airborne_clearance_for_sample(
                            run_release_edge,
                            phase_to_contact,
                            planned_contact.map(|_| planned_progress),
                            descent,
                        ) {
                            Ok(value) => value,
                            Err(error) => {
                                warn!(%error, "IK geometry rejected");
                                continue;
                            }
                        };
                        target = match (FootClearanceFloor::Terrain {
                            surface: height,
                            clearance,
                        })
                        .apply(target, descent)
                        {
                            Ok(value) => value,
                            Err(error) => {
                                warn!(%error, "IK geometry rejected");
                                continue;
                            }
                        };
                        owner_target = rig_rotation.inverse() * (target - rig_origin);
                    }
                    if run_airborne_budget {
                        // Limit the complete owner-local 3D swing after terrain
                        // height and clearance are applied. World plants never
                        // enter this airborne branch and remain exact.
                        let contact_reachable = run_contact_within_follower_step(
                            previous_owner_target,
                            target,
                            rig_origin,
                            rig_rotation,
                            state_delta_seconds,
                        );
                        let descent = run_support_descent(
                            airborne_budget_gait,
                            skeleton.gait_phase,
                            foot_side,
                            profile.support_phase_radius,
                            raw_nominal_weight,
                            contact_reachable
                                && run_contact_within_leg_reach(
                                    target,
                                    upper_snapshot.global.translation(),
                                    terrain_maximum_reach(upper_length, lower_length),
                                ),
                        );
                        let clearance = match run_airborne_clearance_for_sample(
                            run_release_edge,
                            phase_to_contact,
                            planned_contact.map(|_| planned_progress),
                            descent,
                        ) {
                            Ok(value) => value,
                            Err(error) => {
                                warn!(%error, "IK geometry rejected");
                                continue;
                            }
                        };
                        target = match advance_run_airborne_world_target(
                            previous_owner_target,
                            target,
                            rig_origin,
                            rig_rotation,
                            state_delta_seconds,
                            target_speeds
                                .run_sample(run_release_edge, settle_cancelled_for_restart),
                            |xz| terrain_clearance_floor(terrain, clearance, xz),
                        ) {
                            Ok(value) => value,
                            Err(error) => {
                                warn!(%error, "IK geometry rejected");
                                continue;
                            }
                        };
                        owner_target = rig_rotation.inverse() * (target - rig_origin);
                        if toe_off_started
                            && retained_contact.is_none()
                            && planned_contact.is_some()
                        {
                            // Toe-off and next-plan creation can occur in the
                            // same evaluation. Freeze the terrain-feasible
                            // release result as the new swing start so the next
                            // tick cannot reconstruct the plan from the
                            // pre-projection world ankle and repeat the seam.
                            if left {
                                memory.left_planned_contact_start = Some(target);
                                memory.left_planned_contact_phase_start = Some(phase_to_contact);
                            } else {
                                memory.right_planned_contact_start = Some(target);
                                memory.right_planned_contact_phase_start = Some(phase_to_contact);
                            }
                        }
                    }
                    let release_active = next_release_goal.is_some()
                        || owner_target.distance_squared(desired_owner_target) > 0.000001
                        || unplanned_terrain_solve_requires_release(
                            planned_contact,
                            target,
                            foot_position,
                        );
                    let canonical_world = pole_to_world(rig_rotation, canonical_knee_pole(side));
                    let (remembered_pole, previous_end_direction) = if left {
                        (
                            memory.left_terrain_pole_world,
                            memory.left_terrain_end_direction,
                        )
                    } else {
                        (
                            memory.right_terrain_pole_world,
                            memory.right_terrain_end_direction,
                        )
                    };
                    let next_end_direction =
                        (target - upper_snapshot.global.translation()).normalize_or_zero();
                    let pole = transported_terrain_pole(
                        remembered_pole,
                        previous_end_direction,
                        next_end_direction,
                        canonical_world,
                    )
                    .unwrap_or(canonical_world);
                    let pole = constrain_rendered_leg_pole(
                        rig,
                        foot_side,
                        upper_snapshot.global.translation(),
                        foot_position,
                        target,
                        pole,
                        &parents,
                        &transforms.p0(),
                    );
                    let mut resolved_end = None;
                    if let Some(solution) = solve_two_bone_with_reach(
                        TwoBoneChain::new(
                            upper_snapshot.global.translation(),
                            lower_snapshot.global.translation(),
                            foot_position,
                            upper_length,
                            lower_length,
                            pole,
                        ),
                        target,
                        maximum_reach(upper_length, lower_length),
                    ) {
                        resolved_end = Some(solution.end);
                        apply_two_bone_solution(
                            upper,
                            lower,
                            foot,
                            solution,
                            &parents,
                            &mut transforms,
                        );
                        if !state_delta_seconds.is_zero() {
                            let bend = (solution.knee - upper_snapshot.global.translation())
                                .reject_from_normalized(solution.end_direction)
                                .try_normalize();
                            if left {
                                if let Some(bend) = bend {
                                    memory.left_terrain_pole_world = Some(bend);
                                }
                                memory.left_terrain_end_direction = Some(solution.end_direction);
                            } else {
                                if let Some(bend) = bend {
                                    memory.right_terrain_pole_world = Some(bend);
                                }
                                memory.right_terrain_end_direction = Some(solution.end_direction);
                            }
                        }
                    }
                    if profile.gait == LocomotionGait::Run
                        && let Some(normal) = ScenePlanPoint::from_metres(target.xz()).and_then(|point| terrain.normal_at(point))
                        && let Some(sole_axis) = rig.sole_axis(foot_side)
                    {
                        // Run swing orientation is terrain-aware before the
                        // nominal support edge. Arriving tangent prevents the
                        // toe joint from sweeping through the ground while the
                        // nine-degree contact blend catches up.
                        align_foot_to_slope(foot, sole_axis, normal, &parents, &mut transforms);
                    }
                    if left {
                        memory.left_foot_plant = FootPlant::Absent;
                        memory.left_foot_target = Some(owner_target);
                        memory.left_foot_world_target = Some(target);
                        memory.left_support_weight = Some(UnitInterval::ZERO);
                        memory.left_transition_support_weight = Some(UnitInterval::ZERO);
                        memory.left_release_active = release_active;
                        memory.left_release_target = next_release_goal;
                    } else {
                        memory.right_foot_plant = FootPlant::Absent;
                        memory.right_foot_target = Some(owner_target);
                        memory.right_foot_world_target = Some(target);
                        memory.right_support_weight = Some(UnitInterval::ZERO);
                        memory.right_transition_support_weight = Some(UnitInterval::ZERO);
                        memory.right_release_active = release_active;
                        memory.right_release_target = next_release_goal;
                    }
                    if let Some(resolved_end) = resolved_end {
                        // A high-speed unplanned release can request a
                        // terrain waypoint beyond the current analytic reach.
                        // Continue the next sample from the ankle the player
                        // actually sees instead of repeatedly owning the
                        // rejected request. Planned swings keep their frozen
                        // endpoint metadata unchanged.
                        commit_resolved_unplanned_airborne_release(
                            &mut memory,
                            foot_side,
                            run_airborne_budget,
                            planned_contact,
                            release_active,
                            resolved_end,
                            rig_origin,
                            rig_rotation,
                        );
                    }
                    continue;
                }
                let Some(settle) = settle_swing else {
                    continue;
                };
                let mut desired_target = match settle_swing_target(
                    settle.swing_start,
                    settle.landing_target,
                    settle.progress,
                ) {
                    Ok(target) => target,
                    Err(error) => {
                        warn!(%error, "settle swing target rejected");
                        continue;
                    }
                };
                desired_target = match conform_settle_swing_target(
                    terrain,
                    desired_target,
                    foot_position,
                    settle.progress,
                    terrain_blend,
                    rendered_ankle_and_toe,
                    state_delta_seconds,
                ) {
                    Ok(target) => target,
                    Err(error) => {
                        warn!(%error, "IK geometry rejected");
                        continue;
                    }
                };
                let desired_owner_target = rig_rotation.inverse() * (desired_target - rig_origin);
                let previous_owner_target = if left {
                    memory.left_foot_target
                } else {
                    memory.right_foot_target
                };
                // Resolve the follower and toe/sole floor together. Applying
                // clearance only to the distant settle goal leaves the
                // rate-limited intermediate waypoint below terrain for several
                // frames; projecting Y after the cap can instead exceed the
                // continuity budget. This joint search finds the closest
                // terrain-valid waypoint inside the same owner-local sphere.
                let target = match advance_run_airborne_world_target(
                    previous_owner_target,
                    desired_target,
                    rig_origin,
                    rig_rotation,
                    state_delta_seconds,
                    target_speeds.settle(settle),
                    |xz| {
                        transition_clearance_floor(
                            terrain,
                            rendered_ankle_and_toe,
                            state_delta_seconds,
                            xz,
                        )
                    },
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        warn!(%error, "IK geometry rejected");
                        continue;
                    }
                };
                let owner_target = rig_rotation.inverse() * (target - rig_origin);
                let release_active = owner_target.distance_squared(desired_owner_target) > 0.000001;
                let canonical_pole = canonical_knee_pole(side);
                let canonical_world = pole_to_world(rig_rotation, canonical_pole);
                let (remembered_pole, previous_end_direction) = if left {
                    (
                        memory.left_terrain_pole_world,
                        memory.left_terrain_end_direction,
                    )
                } else {
                    (
                        memory.right_terrain_pole_world,
                        memory.right_terrain_end_direction,
                    )
                };
                let next_end_direction =
                    (target - upper_snapshot.global.translation()).normalize_or_zero();
                let remembered = transported_terrain_pole(
                    remembered_pole,
                    previous_end_direction,
                    next_end_direction,
                    canonical_world,
                );
                let pole = remembered.unwrap_or(canonical_world);
                let pole = constrain_rendered_leg_pole(
                    rig,
                    foot_side,
                    upper_snapshot.global.translation(),
                    foot_position,
                    target,
                    pole,
                    &parents,
                    &transforms.p0(),
                );
                if let Some(solution) = solve_two_bone_with_reach(
                    TwoBoneChain::new(
                        upper_snapshot.global.translation(),
                        lower_snapshot.global.translation(),
                        foot_position,
                        upper_length,
                        lower_length,
                        pole,
                    ),
                    target,
                    maximum_reach(upper_length, lower_length),
                ) {
                    settle_contact_reached = settle.progress.is_complete()
                        && solution.end.xz().distance(settle.landing_target.xz()) <= 0.02
                        && ScenePlanPoint::from_metres(solution.end.xz())
                            .and_then(|point| terrain.height_at(point))
                            .is_some_and(|height| sole_is_at_contact(solution.end.y, height));
                    apply_two_bone_solution(
                        upper,
                        lower,
                        foot,
                        solution,
                        &parents,
                        &mut transforms,
                    );
                    if !state_delta_seconds.is_zero() {
                        let bend = (solution.knee - upper_snapshot.global.translation())
                            .reject_from_normalized(solution.end_direction)
                            .try_normalize();
                        if left {
                            if let Some(bend) = bend {
                                memory.left_terrain_pole_world = Some(bend);
                            }
                            memory.left_terrain_end_direction = Some(solution.end_direction);
                        } else {
                            if let Some(bend) = bend {
                                memory.right_terrain_pole_world = Some(bend);
                            }
                            memory.right_terrain_end_direction = Some(solution.end_direction);
                        }
                    }
                }
                if let Some(normal) = ScenePlanPoint::from_metres(target.xz()).and_then(|point| terrain.normal_at(point))
                    && let Some(sole_axis) = rig.sole_axis(foot_side)
                {
                    // A settling foot approaches its contact tangent throughout
                    // the capture arc. Deferring alignment until terminal idle
                    // can drive the rear toe through rising terrain even while
                    // the ankle remains clear. The final rotation pass retains
                    // the existing bounded world-angle step.
                    align_foot_to_slope(foot, sole_axis, normal, &parents, &mut transforms);
                }
                if left {
                    memory.left_foot_plant = FootPlant::Absent;
                    memory.left_foot_target = Some(owner_target);
                    memory.left_foot_world_target = Some(target);
                    memory.left_support_weight = Some(UnitInterval::ZERO);
                    memory.left_transition_support_weight = Some(UnitInterval::ZERO);
                    memory.left_release_active = release_active;
                    memory.left_release_target = release_active.then_some(desired_owner_target);
                } else {
                    memory.right_foot_plant = FootPlant::Absent;
                    memory.right_foot_target = Some(owner_target);
                    memory.right_foot_world_target = Some(target);
                    memory.right_support_weight = Some(UnitInterval::ZERO);
                    memory.right_transition_support_weight = Some(UnitInterval::ZERO);
                    memory.right_release_active = release_active;
                    memory.right_release_target = release_active.then_some(desired_owner_target);
                }
                continue;
            }
            // Do not memorize a footprint while the swing foot is merely
            // approaching the ground. Capturing that stale position early
            // makes the pelvis outrun it, forcing the reach limiter to drag a
            // fully weighted foot and drive the knee toward extension.
            let retained_planned_contact = if left {
                memory.left_planned_contact
            } else {
                memory.right_planned_contact
            };
            // The first nominal-support sample is only an acquisition request.
            // Keep the frozen plan until the propagated sole actually reaches
            // it; clearing here made the next sample rebuild from authored FK
            // just as the ankle entered the final centimetre of contact.
            if acquired_plan_can_clear(plant_acquired) {
                if left {
                    clear_planned_contact_metadata(
                        &mut memory.left_planned_contact,
                        &mut memory.left_planned_contact_start,
                        &mut memory.left_planned_contact_phase_start,
                    );
                } else {
                    clear_planned_contact_metadata(
                        &mut memory.right_planned_contact,
                        &mut memory.right_planned_contact_start,
                        &mut memory.right_planned_contact_phase_start,
                    );
                }
            }
            let ordinary_planned_contact = (ordinary_lowered
                && speed > STATIONARY_ANIMATION_SPEED_LIMIT
                && planar_velocity.length_squared() > 0.0025)
                .then(|| {
                    retained_planned_contact.unwrap_or_else(|| {
                        let projected_com =
                            projected_body_center(rig, &transforms.p0()).unwrap_or(rig_origin);
                        ordinary_contact_target(
                            rig_origin,
                            rig_rotation,
                            projected_com,
                            planar_velocity,
                            profile.step_distance,
                            skeleton.gait_phase.until_contact(foot_side),
                            side,
                        )
                    })
                });
            if plant.is_none()
                && let Some(planned_contact) = ordinary_planned_contact
            {
                // Freeze the next contact as soon as the contact ramp begins.
                // Recomputing it from the advancing COM every tick would make
                // a nominally supported foot chase the body instead of land.
                plant = Some(if profile.gait == LocomotionGait::Run {
                    // Run planning already froze a stance-reachable world
                    // footprint. Reapplying the body's small local foot-track
                    // corridor here would replace it with a point under the
                    // advancing hip and recreate the support-entry snap.
                    planned_contact
                } else {
                    constrain_foot_to_track(planned_contact, rig_origin, rig_rotation, side)
                });
            } else if weight.fraction() >= 0.95 && plant.is_none() && !raised_guard_follower {
                let visible_contact = ordinary_planned_contact.unwrap_or_else(|| {
                    if left {
                        memory.left_foot_world_target
                    } else {
                        memory.right_foot_world_target
                    }
                    .unwrap_or(foot_position)
                });
                plant = Some(constrain_foot_to_track(
                    visible_contact,
                    rig_origin,
                    rig_rotation,
                    side,
                ));
            }
            let mut horizontal_target = plant.unwrap_or_else(|| {
                ordinary_planned_contact.unwrap_or_else(|| {
                    constrain_foot_to_track(foot_position, rig_origin, rig_rotation, side)
                })
            });
            let plant_local = rig_rotation.inverse() * (horizontal_target - rig_origin);
            if plant_local.x * side.coefficient() < foot_track_inner_metres().metres() {
                // A retained world plant can rotate through the body's center
                // during an exact reversal. Move only the offending lateral
                // component back to its anatomical corridor; target velocity
                // limiting below keeps that correction continuous.
                horizontal_target =
                    constrain_foot_to_track(horizontal_target, rig_origin, rig_rotation, side);
                plant = plant.map(|_| horizontal_target);
            }
            let Some(horizontal_point) = ScenePlanPoint::from_metres(horizontal_target.xz()) else {
                continue;
            };
            let Some(height) = terrain.height_at(horizontal_point) else {
                continue;
            };
            let sole_offset = measured_ankle_sole_offset_metres();
            let mut planted_target = Vec3::new(
                horizontal_target.x,
                height.metres() + sole_offset.metres(),
                horizontal_target.z,
            );
            // A planned terrain contact can be vertically unreachable early
            // in its acquisition. Solve toward the nearest reachable point
            // without overwriting the frozen plan; once the body arrives, the
            // same plan becomes the stationary stance plant.
            let Some(reachable_target) = acquisition_planted_target(
                planted_target,
                upper_snapshot.global.translation(),
                terrain_maximum_reach(upper_length, lower_length),
                profile.gait,
                plant_acquired,
            ) else {
                continue;
            };
            planted_target = reachable_target;
            horizontal_target.x = planted_target.x;
            horizontal_target.z = planted_target.z;
            // Reach limiting may have moved the target into another triangle.
            // Resample that actual point instead of retaining a height from the
            // old XZ and a normal from the new one.
            let Some(horizontal_point) = ScenePlanPoint::from_metres(horizontal_target.xz()) else {
                continue;
            };
            let Some(height) = terrain.height_at(horizontal_point) else {
                continue;
            };
            planted_target.y = height.metres() + sole_offset.metres();
            if let Some((rendered_ankle, rendered_toe)) = rendered_ankle_and_toe
                && let Some(toe_floor) = match toe_aware_clearance_floor(
                    rendered_ankle,
                    rendered_toe,
                    horizontal_point,
                    ik_tuning().terrain_contact_toe_clearance_metres,
                    |xz| terrain.height_at(xz),
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        warn!(%error, "IK geometry rejected");
                        continue;
                    }
                }
            {
                planted_target = match toe_floor.apply(planted_target, FootDescent::RaiseOnly) {
                    Ok(target) => target,
                    Err(error) => {
                        warn!(%error, "IK geometry rejected");
                        continue;
                    }
                };
            }
            if left {
                memory.left_foot_plant.replace_target(plant);
            } else {
                memory.right_foot_plant.replace_target(plant);
            }
            // Acquisition advances in world space. Rate-limiting the equivalent
            // owner-space point made a stationary plant move backward by the
            // controller's 8.6 cm/tick run displacement before applying the
            // 5.3 cm target cap, so the ankle could never catch its contact.
            // Once contact is reported, solve directly to the frozen world
            // plant; a stance foot is stationary or released, never skated.
            let terrain_alignment_active =
                smoothstep(0.05, 0.9, weight.fraction()) * terrain_blend.fraction() > 0.001;
            let release_target_speed = memory
                .settle
                .map(|settle| target_speeds.settle(settle))
                .unwrap_or(target_speeds.release);
            let support_run_airborne_budget = uses_run_airborne_motion_budget(
                profile.gait,
                memory.effective_planar_speed(planar_speed),
            );
            let support_budget_gait = if support_run_airborne_budget {
                LocomotionGait::Run
            } else {
                profile.gait
            };
            let (
                previous_owner_target,
                previous_world_target,
                previous_support,
                previous_reported_support,
                was_releasing,
            ) = if left {
                (
                    run_previous_owner_target(
                        support_budget_gait,
                        memory.left_last_rendered_owner,
                        memory.left_foot_target,
                    ),
                    memory.left_foot_world_target,
                    memory.left_transition_support_weight,
                    memory.left_support_weight,
                    memory.left_release_active,
                )
            } else {
                (
                    run_previous_owner_target(
                        support_budget_gait,
                        memory.right_last_rendered_owner,
                        memory.right_foot_target,
                    ),
                    memory.right_foot_world_target,
                    memory.right_transition_support_weight,
                    memory.right_support_weight,
                    memory.right_release_active,
                )
            };
            let mut target = if plant_acquired {
                planted_target
            } else if profile.gait == LocomotionGait::Run {
                // Entering nominal support does not bypass the airborne
                // follower. The frozen plant becomes direct only after the
                // propagated sole has truthfully acquired it.
                let fixed_contact_reachable = run_contact_within_follower_motion_step(
                    previous_owner_target,
                    planted_target,
                    rig_origin,
                    rig_rotation,
                    state_delta_seconds,
                );
                // Match the exact reach enforced by the upright analytic
                // solve below. Planning's looser 12-degree terrain reach left
                // the captured phase-.867 target about 1 cm beyond Run's
                // 20-degree solve reach, so the isolated retarget passed while
                // the rendered sole still stopped above contact.
                let final_solve_reach = maximum_reach(upper_length, lower_length);
                let fixed_contact_within_leg_reach = run_contact_within_leg_reach(
                    planted_target,
                    upper_snapshot.global.translation(),
                    final_solve_reach,
                );
                let descent = run_support_descent(
                    profile.gait,
                    skeleton.gait_phase,
                    foot_side,
                    profile.support_phase_radius,
                    raw_nominal_weight,
                    true,
                );
                if descent.permits_descent()
                    && (!fixed_contact_reachable || !fixed_contact_within_leg_reach)
                    && let Some(transported_contact) = retarget_unacquired_run_contact_for_descent(
                        previous_owner_target,
                        planted_target,
                        rig_origin,
                        rig_rotation,
                        side,
                        upper_snapshot.global.translation(),
                        final_solve_reach,
                        state_delta_seconds,
                        |xz| terrain.height_at(xz),
                    )
                {
                    // The final pre-contact footprint follows the current
                    // owner displacement once, then becomes the new frozen
                    // world plant atomically. After truthful acquisition the
                    // ordinary direct-plant path keeps this point stationary.
                    planted_target = transported_contact;
                    plant = Some(transported_contact);
                    if left {
                        memory.left_foot_plant.replace_target(plant);
                        memory.left_planned_contact = Some(transported_contact);
                    } else {
                        memory.right_foot_plant.replace_target(plant);
                        memory.right_planned_contact = Some(transported_contact);
                    }
                }
                let contact_reachable = run_contact_within_follower_step(
                    previous_owner_target,
                    planted_target,
                    rig_origin,
                    rig_rotation,
                    state_delta_seconds,
                );
                let acquisition_clearance = if contact_reachable {
                    SignedLength::ZERO
                } else {
                    ik_tuning().run_swing_minimum_sole_clearance_metres
                };

                match advance_run_airborne_world_target(
                    previous_owner_target,
                    planted_target,
                    rig_origin,
                    rig_rotation,
                    state_delta_seconds,
                    target_speeds.run_airborne,
                    |xz| terrain_clearance_floor(terrain, acquisition_clearance, xz),
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        warn!(%error, "IK geometry rejected");
                        continue;
                    }
                }
            } else if previous_support.is_some_and(terrain_leg_has_support) {
                planted_target
            } else {
                advance_foot_target_at_speed(
                    previous_world_target,
                    planted_target,
                    state_delta_seconds,
                    release_target_speed,
                )
            };
            let unplanned_support_release_owned = unplanned_support_release_is_owned(
                was_releasing,
                previous_support,
                previous_reported_support,
                retained_planned_contact,
                target,
                planted_target,
                foot_position,
            );
            let bounded_unplanned_support_release = support_run_airborne_budget
                && retained_planned_contact.is_none()
                && !plant_acquired
                && unplanned_support_release_owned;
            target = match bound_unacquired_run_support_release_target(
                bounded_unplanned_support_release,
                false,
                false,
                true,
                previous_owner_target,
                target,
                rig_origin,
                rig_rotation,
                state_delta_seconds,
                target_speeds.run_airborne,
                |xz| terrain_clearance_floor(terrain, SignedLength::ZERO, xz),
            ) {
                Ok(value) => value,
                Err(error) => {
                    warn!(%error, "IK geometry rejected");
                    continue;
                }
            };
            if memory.settle.is_some() && !plant_acquired {
                // The selected settle support can begin airborne. Until its
                // rendered sole truthfully acquires contact it needs the same
                // toe/sole flight floor as the opposite capture foot. Once
                // the toe-aware contact itself fits in this tick's follower
                // and analytic-reach budgets, land atomically; permanently
                // reapplying the flight floor would make truthful acquisition
                // impossible and leave settle stuck at progress 1 forever.
                let contact_candidate = match advance_run_airborne_world_target(
                    previous_owner_target,
                    planted_target,
                    rig_origin,
                    rig_rotation,
                    state_delta_seconds,
                    release_target_speed,
                    |xz| terrain_clearance_floor(terrain, SignedLength::ZERO, xz),
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        warn!(%error, "IK geometry rejected");
                        continue;
                    }
                };
                let contact_reachable = contact_candidate.distance_squared(planted_target)
                    <= 0.000001
                    && run_contact_within_leg_reach(
                        planted_target,
                        upper_snapshot.global.translation(),
                        terrain_maximum_reach(upper_length, lower_length),
                    );
                target = if contact_reachable {
                    planted_target
                } else {
                    match advance_run_airborne_world_target(
                        previous_owner_target,
                        planted_target,
                        rig_origin,
                        rig_rotation,
                        state_delta_seconds,
                        release_target_speed,
                        |xz| {
                            transition_clearance_floor(
                                terrain,
                                rendered_ankle_and_toe,
                                state_delta_seconds,
                                xz,
                            )
                        },
                    ) {
                        Ok(value) => value,
                        Err(error) => {
                            warn!(%error, "IK geometry rejected");
                            continue;
                        }
                    }
                };
            }
            let release_active = target.distance_squared(planted_target) > 0.000001
                || (!plant_acquired
                    && unplanned_terrain_solve_requires_release(
                        retained_planned_contact,
                        target,
                        foot_position,
                    ));
            memory.retain_solve_request(
                foot_side,
                FootSolveRequest {
                    frame,
                    target,
                    planted_target,
                    weight,
                    release_active,
                    bounded_unplanned_support_release,
                },
                state_delta_seconds,
            );
            let canonical_pole = canonical_knee_pole(side);
            let canonical_world = pole_to_world(rig_rotation, canonical_pole);
            let (remembered_pole, previous_end_direction) = if left {
                (
                    memory.left_terrain_pole_world,
                    memory.left_terrain_end_direction,
                )
            } else {
                (
                    memory.right_terrain_pole_world,
                    memory.right_terrain_end_direction,
                )
            };
            let next_end_direction =
                (target - upper_snapshot.global.translation()).normalize_or_zero();
            let remembered = transported_terrain_pole(
                remembered_pole,
                previous_end_direction,
                next_end_direction,
                canonical_world,
            );
            let pole = remembered
                .or_else(|| {
                    authored_knee_pole_world(
                        upper_snapshot.global.translation(),
                        lower_snapshot.global.translation(),
                        target,
                        canonical_world,
                    )
                })
                .unwrap_or(canonical_world);
            let pole = constrain_rendered_leg_pole(
                rig,
                foot_side,
                upper_snapshot.global.translation(),
                foot_position,
                target,
                pole,
                &parents,
                &transforms.p0(),
            );
            let solve_reach = if speed <= STATIONARY_ANIMATION_SPEED_LIMIT {
                terrain_maximum_reach(upper_length, lower_length)
            } else {
                maximum_reach(upper_length, lower_length)
            };
            // The transported pole already provides temporal continuity.
            // Authored-bend preservation happens inside the generic solver
            // after pole selection and can rotate the resulting knee outside
            // the anatomical foot-facing cone, so leg IK must use the final
            // constrained pole without another authored blend.
            let solution = solve_two_bone_with_reach(
                TwoBoneChain::new(
                    upper_snapshot.global.translation(),
                    lower_snapshot.global.translation(),
                    foot_position,
                    upper_length,
                    lower_length,
                    pole,
                ),
                target,
                solve_reach,
            );
            let mut reported_support_weight = UnitInterval::ZERO;
            if let Some(solution) = solution {
                let sole_at_contact = ScenePlanPoint::from_metres(solution.end.xz()).and_then(|point| terrain.height_at(point)).is_some_and(|height| {
                    sole_is_at_contact(solution.end.y, height)
                        && plant.is_some_and(|plant| solution.end.xz().distance(plant.xz()) <= 0.02)
                });
                if sole_at_contact {
                    reported_support_weight = weight;
                    if left {
                        memory.left_foot_plant.observe_contact(true);
                        memory.left_planned_contact_phase_start = None;
                    } else {
                        memory.right_foot_plant.observe_contact(true);
                        memory.right_planned_contact_phase_start = None;
                    }
                }
                apply_two_bone_solution(upper, lower, foot, solution, &parents, &mut transforms);
                // The terrain-feasible waypoint can still lie beyond the
                // current analytic chain. Persist the end the player can
                // actually see and reach, not the rejected pre-solve
                // request, so the next sample continues from the visible
                // ankle and diagnostics report truthful release ownership.
                commit_resolved_unacquired_support_release(
                    &mut memory,
                    foot_side,
                    bounded_unplanned_support_release,
                    solution.end,
                    rig_origin,
                    rig_rotation,
                );
                let bend = (solution.knee - upper_snapshot.global.translation())
                    .reject_from_normalized(solution.end_direction);
                if !state_delta_seconds.is_zero()
                    && let Some(valid) = bend.try_normalize()
                {
                    if left {
                        memory.left_terrain_pole_world = Some(valid);
                    } else {
                        memory.right_terrain_pole_world = Some(valid);
                    }
                }
                if !state_delta_seconds.is_zero() {
                    if left {
                        memory.left_terrain_end_direction = Some(solution.end_direction);
                    } else {
                        memory.right_terrain_end_direction = Some(solution.end_direction);
                    }
                }
            }
            if left {
                memory.left_support_weight = Some(reported_support_weight);
            } else {
                memory.right_support_weight = Some(reported_support_weight);
            }
            // The final rendered-contact result owns orientation authority.
            // Planned acquisition is still airborne until its sole actually
            // reaches the intended contact, so bound that transition after
            // every solve/alignment path rather than inferring it from gait.
            airborne_orientation_owned[leg_index] =
                !terrain_leg_has_support(reported_support_weight);
            if evaluation_advances {
                if left {
                    memory.left_contact_orientation_blend_active = update_contact_orientation_blend(
                        memory.left_contact_orientation_blend_active,
                        previous_support,
                        reported_support_weight,
                    );
                } else {
                    memory.right_contact_orientation_blend_active =
                        update_contact_orientation_blend(
                            memory.right_contact_orientation_blend_active,
                            previous_support,
                            reported_support_weight,
                        );
                }
            }
            if terrain_alignment_active
                && let Some(normal) = terrain.normal_at(horizontal_point)
                && let Some(sole_axis) = rig.sole_axis(foot_side)
            {
                let cached_chain = if left {
                    memory.left_rotation_chain
                } else {
                    memory.right_rotation_chain
                };
                if evaluation_advances || cached_chain.is_none() {
                    align_foot_to_slope(foot, sole_axis, normal, &parents, &mut transforms);
                }
            }
        }
        if let Err(error) = finalize_leg_rotation_chains(
            rig,
            skeleton,
            speed,
            rig_rotation,
            &mut memory,
            evaluation_advances,
            state_delta_seconds,
            airborne_orientation_owned,
            airborne_just_released,
            &parents,
            &mut transforms,
        ) {
            warn!(%error, ?owner, "Rejected final leg rotation");
            continue;
        }
        let safe_settle_fallback = memory.settle.is_some_and(|settle| {
            settle.elapsed_seconds >= ik_tuning().safe_settle_fallback_seconds
                && settle_stance_is_safe(
                    projected_body_center(rig, &transforms.p0()).unwrap_or(rig_origin),
                    memory.left_foot_world_target,
                    memory.right_foot_world_target,
                    terrain,
                )
        });
        let settle_requests_completion = (settle_ready_for_contact && settle_contact_reached)
            || safe_settle_fallback
            || settle_is_terminal(&memory);
        if settle_requests_completion {
            // Completion leaves two exact terrain plants for idle. A progress
            // 1 settle with no active followers is terminal even if its last
            // analytic solve stopped above contact; otherwise it can freeze a
            // mid-stride pose forever with neither foot reporting support.
            prepare_terminal_settle_contacts(&mut memory, rig_origin, rig_rotation, |xz| {
                terrain.height_at(xz)
            });
            if terminal_settle_contacts_are_rendered(&memory, |xz| terrain.height_at(xz)) {
                finish_settle_for_idle(&mut memory);
            }
        }
        if let Err(error) = memory.publish(owner, &mut ik_states, &mut commands) {
            warn!(%error, "IK targets rejected before publication");
            continue;
        }
    }
}

fn ik_tuning() -> InverseKinematicsConfig {
    runtime_animation_config().inverse_kinematics
}

#[cfg(test)]
mod tests;
