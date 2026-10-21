//! Animation review metric policy and calculations.

use super::*;
use adventuresim_tactical_core::animation::STATIONARY_ANIMATION_SPEED_LIMIT;
pub(super) use catastrophic_contacts::catastrophic_foot_displacement;
#[cfg(test)]
use clearance::{contact_sole_clearance_range, foot_terrain_relief};
pub(super) use continuity::*;
#[cfg(test)]
use facing::{maximum_facing_error, maximum_facing_tracking_excess, maximum_guard_facing_error};
pub(super) use height::*;
use height_peaks::prominent_height_peaks;
pub(super) use pose::*;
use support_slip::supported_foot_slip;

mod catastrophic_contacts;
mod clearance;
mod continuity;
mod facing;
mod guard_liveness;
mod height;
mod height_peaks;
mod pose;
mod support_slip;

pub(super) const RAISED_MINIMUM_INTER_FOOT_SEPARATION_METRES: f32 = 0.16;
pub(super) const RAISED_MAXIMUM_PELVIS_VERTICAL_STEP_METRES: f32 = 0.05;
// The parent guard-entry fixture already moves 33.13 mm in one sampled frame.
// Allow less than 4 mm of additional height-system overhead without weakening
// the broader raised-guard continuity bound.
pub(super) const LOCOMOTION_STATE_MAXIMUM_PELVIS_VERTICAL_STEP_METRES: f32 = 0.04;
pub(super) const ORDINARY_VERTICAL_RANGE_LIMIT_METRES: f32 = 0.20;
pub(super) const RAISED_GUARD_VERTICAL_RANGE_LIMIT_METRES: f32 = 0.30;

pub(super) const MINIMUM_GUARD_SWING_TRAVEL_METRES: f32 = 0.05;
pub(super) const MINIMUM_GUARD_SWING_CLEARANCE_GAIN_METRES: f32 = 0.03;

pub(super) fn both_feet_behind_hips(frames: &[FrameSample]) -> bool {
    let mut duration = 0.0_f32;
    for pair in frames.windows(2) {
        let [before, after] = pair else {
            continue;
        };
        if before.scenario != after.scenario
            || after.speed_metres_per_second <= SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        {
            duration = 0.0;
            continue;
        }
        let Some(direction) = Vec3::from_array(after.world_travel_direction).try_normalize() else {
            duration = 0.0;
            continue;
        };
        let behind = [
            (BoneRole::ThighLeft, BoneRole::FootLeft),
            (BoneRole::ThighRight, BoneRole::FootRight),
        ]
        .into_iter()
        .all(|(hip, foot)| {
            let Some(hip) = after.bones.get(hip) else {
                return false;
            };
            let Some(foot) = after.bones.get(foot) else {
                return false;
            };
            (Vec3::from_array(foot.position) - Vec3::from_array(hip.position)).dot(direction)
                < -0.02
        });
        if behind {
            duration += (after.time_seconds.seconds() - before.time_seconds.seconds()).max(0.0);
            if duration >= 0.3 {
                return true;
            }
        } else {
            duration = 0.0;
        }
    }
    false
}

pub(super) fn scenario_metrics(
    frames: &[FrameSample],
) -> ReportResult<Vec<ScenarioMetrics>> {
    let mut grouped = BTreeMap::<&ScenarioMetadata<'static>, Vec<&FrameSample>>::new();
    for frame in frames {
        grouped.entry(&frame.scenario).or_default().push(frame);
    }
    grouped
        .into_iter()
        .map(|(scenario, frames)| {
            let metadata = scenario;
            let metric_frames = metric_frames_after_warmup(metadata, &frames);
            let procedural_frames = metric_frames
                .iter()
                .copied()
                .filter(|frame| metadata.kind() == ScenarioKind::Terrain || frame.guard_action)
                .collect::<Vec<_>>();
            let guard_liveness = GuardStepLivenessMetrics::measure(&metric_frames)?;
            let clearance = ScenarioClearanceMetrics::measure(&metric_frames)?;
            let expected = ExpectedScenarioBounds::for_scenario(metadata, &clearance.foot_terrain_relief_metres)?;
            Ok(ScenarioMetrics {
                scenario: (*scenario).clone(),
                frame_count: frames.len(),
                expected,
                continuity: ScenarioContinuityMetrics::measure(metadata, &metric_frames)?,
                loop_seam: LoopSeamMetrics::measure(metadata, &frames)?,
                height: ScenarioHeightMetrics::measure(metadata, &metric_frames)?,
                clearance,
                pose: ScenarioPoseMetrics::measure(metadata, &metric_frames, &procedural_frames)?,
                facing: ScenarioFacingMetrics::measure(&metric_frames)?,
                maximum_planted_foot_drift_metres: maximum_planted_foot_drift(metadata, &metric_frames)?,
                guard_step_liveness: guard_liveness,
            })
        })
        .collect()
}

pub(super) fn quat(bone: &BoneSample) -> Quat {
    Quat::from_array(bone.rotation_xyzw).normalize()
}

pub(super) fn body_local(frame: &FrameSample, bone: BoneRole) -> Option<Vec3> {
    let world = Vec3::from_array(frame.bones.get(bone)?.position)
        - Vec3::from_array(frame.root_position_metres);
    Some(Quat::from_array(frame.body_rotation_xyzw).inverse() * world)
}

pub(super) fn is_leg_bone(name: &BoneRole) -> bool {
    matches!(
        name,
        BoneRole::ThighLeft
            | BoneRole::ThighRight
            | BoneRole::ShinLeft
            | BoneRole::ShinRight
            | BoneRole::FootLeft
            | BoneRole::FootRight
    )
}

pub(super) fn is_foot_bone(name: &BoneRole) -> bool {
    matches!(name, BoneRole::FootLeft | BoneRole::FootRight)
}

pub(super) fn is_knee_bone(name: &BoneRole) -> bool {
    matches!(name, BoneRole::ShinLeft | BoneRole::ShinRight)
}

pub(super) fn target_body_local(frame: &FrameSample, world: Vec3) -> Vec3 {
    Quat::from_array(frame.body_rotation_xyzw).inverse()
        * (world - Vec3::from_array(frame.root_position_metres))
}

pub(super) fn body_local_rotation(frame: &FrameSample, bone: &BoneSample) -> Quat {
    Quat::from_array(frame.body_rotation_xyzw).inverse() * quat(bone)
}

pub(super) fn ordinary_swing_frame_is_owned(frame: &FrameSample) -> bool {
    if frame.scenario.kind() != ScenarioKind::Terrain
        || frame.speed_metres_per_second <= SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        || frame.ik_settle_progress.is_some()
    {
        return true;
    }
    [
        (
            "left",
            frame.ik_left_support_weight,
            frame.ik_left_authored_target,
            frame.ik_left_solve_target,
            frame.ik_left_planned_contact,
            frame.ik_left_release_active,
        ),
        (
            "right",
            frame.ik_right_support_weight,
            frame.ik_right_authored_target,
            frame.ik_right_solve_target,
            frame.ik_right_planned_contact,
            frame.ik_right_release_active,
        ),
    ]
    .into_iter()
    .all(
        |(_side, support, authored, solved, planned, release_active)| {
            support.zip(release_active).is_some_and(|(support, release_active)| {
                support.fraction() > 0.05
                    || authored.zip(solved).is_some_and(|(authored, solved)| {
                        let authored = Vec3::from_array(authored);
                        let solved = Vec3::from_array(solved);
                        planned.is_some() || release_active || solved.distance(authored) <= 0.03
                    })
            })
        },
    )
}

pub(super) fn ordinary_unplanned_release_transition_is_valid(
    before: &FrameSample,
    after: &FrameSample,
    before_solve: Option<[f32; 3]>,
    after_solve: Option<[f32; 3]>,
    before_release_target: Option<[f32; 3]>,
    after_release_target: Option<[f32; 3]>,
) -> bool {
    before_solve
        .zip(after_solve)
        .is_some_and(|(before_solve, after_solve)| {
            let before_solve_world = Vec3::from_array(before_solve);
            let after_solve_world = Vec3::from_array(after_solve);
            let before_solve = target_body_local(before, before_solve_world);
            let after_solve = target_body_local(after, after_solve_world);
            let frozen_goal_converges = before_release_target.zip(after_release_target).is_none_or(
                |(before_goal, after_goal)| {
                    let before_goal = Vec3::from_array(before_goal);
                    let after_goal = Vec3::from_array(after_goal);
                    target_body_local(before, before_goal)
                        .distance(target_body_local(after, after_goal))
                        > 0.002
                        || after_solve_world.distance(after_goal)
                            <= before_solve_world.distance(before_goal) + 0.001
                },
            );
            let step_limit = if frame_uses_run_motion_budget(after) {
                // Run's owner-space target advances up to 8.75 cm per sample;
                // its terrain scenario independently rejects rendered foot
                // motion above 9.5 cm. Walk/settle retains the 5.5 cm budget.
                0.095
            } else {
                0.055
            };
            after_solve.distance(before_solve) <= step_limit && frozen_goal_converges
        })
}

pub(super) fn frame_uses_run_motion_budget(frame: &FrameSample) -> bool {
    let run_speed_threshold = (walk_locomotion_profile()
        .reference_speed
        .metres_per_second()
        + run_locomotion_profile().reference_speed.metres_per_second())
        * 0.5;
    frame.speed_metres_per_second.metres_per_second() >= run_speed_threshold
}

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(super) fn ordinary_planned_transition_is_valid(
    before: &FrameSample,
    after: &FrameSample,
    foot: BoneRole,
    before_plan: Option<[f32; 3]>,
    after_plan: Option<[f32; 3]>,
    before_solve: Option<[f32; 3]>,
    after_solve: Option<[f32; 3]>,
    after_support: UnitInterval,
    release_active: bool,
) -> bool {
    before_plan
        .zip(after_plan)
        .zip(before_solve.zip(after_solve))
        .is_none_or(|((before_plan, after_plan), (before_solve, after_solve))| {
            let before_plan = Vec3::from_array(before_plan);
            let after_plan = Vec3::from_array(after_plan);
            let before_solve_world = Vec3::from_array(before_solve);
            let after_solve_world = Vec3::from_array(after_solve);
            if frame_uses_run_motion_budget(after) {
                let plan_is_frozen = before_plan.distance(after_plan) <= 0.002;
                let atomic_acquisition_retarget = after_support.fraction() > 0.05
                    && !release_active
                    && after_solve_world.distance(after_plan) <= 0.02;
                let owned_airborne_replan = after_support.fraction() <= 0.5 && release_active;
                let rendered_step = before.bones.get(foot).zip(after.bones.get(foot)).map(
                    |(before_foot, after_foot)| {
                        target_body_local(after, Vec3::from_array(after_foot.position)).distance(
                            target_body_local(before, Vec3::from_array(before_foot.position)),
                        )
                    },
                );
                (plan_is_frozen || atomic_acquisition_retarget || owned_airborne_replan)
                    && rendered_step.is_some_and(|step| step <= 0.095)
            } else {
                before_plan.distance(after_plan) <= 0.002
                    && after_solve_world.distance(after_plan)
                        <= before_solve_world.distance(before_plan) + 0.005
            }
        })
}

pub(super) fn transition_flight_toe_sample_is_active(
    scenario: &ScenarioMetadata<'_>,
    speed_metres_per_second: SignedSpeed,
    settle_progress: Option<adventuresim_tactical_core::animation::SettleProgress>,
) -> bool {
    !matches!(
        scenario.identity(),
        Some(ScenarioId::TerrainRunFlightStop) | Some(ScenarioId::TerrainTapRestartCrossfade)
    ) || speed_metres_per_second > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        || settle_progress.is_some()
}

pub(super) fn strict_transition_flight_toe_clearance_is_valid(clearance_metres: fabelgeist_geometry::SignedLength) -> bool {
    clearance_metres.metres() >= 0.01
}

pub(super) fn reported_support_contacts_are_valid(frames: &[FrameSample]) -> bool {
    let tolerance = sole_contact_tolerance_metres();
    frames.iter().all(|frame| {
        if is_quickstep_scenario(&frame.scenario)
            || frame.scenario.kind() == ScenarioKind::Attack
            || frame.action != SkeletonAction::None
            || (!scenario_uses_terrain_ik(&frame.scenario)
                && frame.weapon_guard == WeaponGuardState::Lowered)
        {
            // In an FK-only comparison the semantic weights describe authored
            // loading, not a claim that the procedural solver owns contact.
            // Attack captures exercise the same raised-guard locomotion and
            // terrain IK ownership used outside attacks.
            return true;
        }
        [
            (BoneRole::FootLeft, frame.ik_left_support_weight),
            (BoneRole::FootRight, frame.ik_right_support_weight),
        ]
        .into_iter()
        .all(|(foot, support)| {
            let quickstep_toe_contact = if is_quickstep_scenario(&frame.scenario) {
                let toe = if foot == BoneRole::FootLeft {
                    BoneRole::ToeLeft
                } else {
                    BoneRole::ToeRight
                };
                frame
                    .bones
                    .get(toe)
                    .and_then(|bone| bone.terrain_clearance_metres)
                    .is_some_and(|clearance| clearance.metres().abs() <= tolerance.metres())
            } else {
                false
            };
            support.is_some_and(|support| {
                support.fraction() < 0.95
                    || quickstep_toe_contact
                    || frame
                        .bones
                        .get(foot)
                        .and_then(|bone| bone.terrain_clearance_metres)
                        .is_some_and(|ankle_clearance| {
                            (ankle_clearance.metres() - measured_ankle_sole_offset_metres().metres()).abs()
                                <= if frame.scenario.is(ScenarioId::RaisedGuardStationaryTurn) {
                                    // The combat stance is deliberately
                                    // non-flat-footed; its planted pole target
                                    // has a measured 1.11 cm ankle residual.
                                    0.012
                                } else {
                                    tolerance.metres()
                                }
                        })
            })
        })
    })
}

pub(super) fn terrain_run_contacts_are_valid(frames: &[FrameSample]) -> AnimationValueResult<bool> {
    let run = frames
        .iter()
        .filter(|frame| frame.scenario.is(ScenarioId::TerrainSteadyRun55))
        .collect::<Vec<_>>();
    if run.is_empty() {
        return Ok(true);
    }
    let warmed = phase_validation_frames(&run);
    let mut acquisitions = Vec::new();
    let mut last = None;
    for frame in &warmed {
        let (left, right) = frame.required_ik_support()?;
        let support = if left.fraction().max(right.fraction()) < 0.5 {
            None
        } else {
            Some(left.fraction() >= right.fraction())
        };
        if support != last {
            if let Some(left) = support {
                acquisitions.push(left);
            }
            last = support;
        }
    }
    Ok(acquisitions.len() >= 4
        && acquisitions.contains(&true)
        && acquisitions.contains(&false)
        && acquisitions.windows(2).all(|pair| pair[0] != pair[1])
        && maximum_no_support_seconds(&warmed)?.measured().is_some_and(|elapsed| {
                (0.08..=0.20).contains(&elapsed.seconds())
            }))
}

fn metric_frames_after_warmup<'a>(
    metadata: &ScenarioMetadata<'_>,
    frames: &[&'a FrameSample],
) -> Vec<&'a FrameSample> {
    // Terrain conformity calibrates retained pelvis/plant state during its first cycle.
    if metadata.kind() == ScenarioKind::Terrain {
        phase_validation_frames(frames)
    } else {
        frames.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn foot_metric_frame(
        scenario_frame: usize,
        left_x: f32,
        left_support_weight: f32,
        left_terrain_height: f32,
        right_terrain_height: f32,
    ) -> FrameSample {
        let foot = |x: f32, terrain_height: f32| BoneSample {
            position: Vec3::new(x, terrain_height, 0.0).to_array(),
            rotation_xyzw: Quat::IDENTITY.to_array(),
            terrain_clearance_metres: Some(fabelgeist_geometry::SignedLength::from_metres(0.0).unwrap()),
        };
        FrameSample {
            scenario: "metric-test".into(),
            scenario_frame,
            time_seconds: AnimationElapsed::from_seconds(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            )
            .unwrap(),
            speed_metres_per_second: SignedSpeed::from_metres_per_second(2.0).unwrap(),
            gait_phase: CyclePhase::from_cycles(0.0).unwrap(),
            locomotion_sample_tick: AnimationSampleTick::from(scenario_frame as u64),
            body_acceleration: Vec3::ZERO.to_array(),
            world_acceleration: Vec3::ZERO.to_array(),
            bouncy_active_springs: 0,
            bouncy_maximum_deflection_radians: adventuresim_building_generator::spatial_geometry::Radians::ZERO,
            contact_sequence: ContactSequence::ZERO,
            contact_foot: LeadFoot::Left,
            landing_sequence: adventuresim_tactical_core::animation::LandingSequence::ZERO,
            landing_impact_speed: LandingImpactSpeed::ZERO,
            body_lean_pitch_degrees: Some(adventuresim_tactical_core::animation::SignedAngleDegrees::ZERO),
            body_lean_roll_degrees: Some(adventuresim_tactical_core::animation::SignedAngleDegrees::ZERO),
            landing_compression_metres: Some(HeightDistance::ZERO),
            root_distance_metres: fabelgeist_geometry::SignedLength::ZERO,
            root_position_metres: Vec3::new(0.0, CAPTURE_ROOT_GROUND_OFFSET_METRES, 0.0).to_array(),
            world_travel_direction: Vec3::NEG_Z.to_array(),
            desired_body_forward_direction: Vec3::NEG_Z.to_array(),
            body_forward_direction: Vec3::NEG_Z.to_array(),
            body_rotation_xyzw: Quat::IDENTITY.to_array(),
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
            action: SkeletonAction::None,
            action_phase: adventuresim_tactical_core::animation::UnitInterval::ZERO,
            attack_animation: None,
            strike_family: StrikeFamily::Thrust,
            guard_action: false,
            left_support_weight: UnitInterval::from_fraction(left_support_weight).unwrap(),
            right_support_weight: UnitInterval::from_fraction(0.0).unwrap(),
            desired_left_foot_target: None,
            desired_right_foot_target: None,
            ik_left_authored_target: None,
            ik_right_authored_target: None,
            ik_left_planned_contact: None,
            ik_right_planned_contact: None,
            ik_settle_capture_point: None,
            ik_left_solve_target: None,
            ik_right_solve_target: None,
            ik_left_support_weight: Some(UnitInterval::from_fraction(0.0).unwrap()),
            ik_right_support_weight: Some(UnitInterval::from_fraction(0.0).unwrap()),
            ik_left_release_active: Some(false),
            ik_right_release_active: Some(false),
            ik_left_release_target: None,
            ik_right_release_target: None,
            ik_settle_progress: None,
            ik_left_knee_foot_yaw_offset_degrees: Some(adventuresim_tactical_core::animation::AngularMagnitudeDegrees::ZERO),
            ik_right_knee_foot_yaw_offset_degrees: Some(adventuresim_tactical_core::animation::AngularMagnitudeDegrees::ZERO),
            semantic_route_requested_path: SemanticRoutePath::GeneralPose,
            semantic_route_selected_path: SemanticRoutePath::GeneralPose,
            semantic_route_runtime_evaluated: false,
            screenshots: BTreeMap::new(),
            bones: CapturedBones::from_iter([
                (BoneRole::FootLeft, foot(left_x, left_terrain_height)),
                (BoneRole::FootRight, foot(0.2, right_terrain_height)),
            ]),
        }
    }

    #[test]
    fn complete_run_cycles_use_the_run_foot_continuity_budget_only() {
        assert!(
            0.149
                <= foot_continuity_limit(&ScenarioMetadata::from_label("terrain-run-flight-stop")).unwrap().metres()
        );
        assert!(
            0.151 > foot_continuity_limit(&ScenarioMetadata::from_label("terrain-run-flight-stop")).unwrap().metres()
        );
        assert!(
            0.149
                <= foot_continuity_limit(&ScenarioMetadata::from_label(
                    "terrain-tap-restart-crossfade"
                )).unwrap().metres()
        );
        assert!(
            0.151
                > foot_continuity_limit(&ScenarioMetadata::from_label(
                    "terrain-tap-restart-crossfade"
                )).unwrap().metres()
        );
        assert!(
            0.056
                > foot_continuity_limit(&ScenarioMetadata::from_label("terrain-tap-stop-forward")).unwrap().metres()
        );
        assert_eq!(
            foot_continuity_limit(&ScenarioMetadata::from_label("terrain-steady-run-5.5")).unwrap().metres(),
            0.15
        );
        assert_eq!(
            knee_continuity_limit(&ScenarioMetadata::from_label("terrain-steady-run-5.5")).unwrap().metres(),
            0.16
        );
        assert_eq!(
            knee_continuity_limit(&ScenarioMetadata::from_label("steady-run-5.5")).unwrap().metres(),
            0.13
        );
        assert_eq!(
            knee_continuity_limit(&ScenarioMetadata::from_label("steady-walk")).unwrap().metres(),
            0.10
        );
        assert_eq!(
            loop_seam_position_limit(&ScenarioMetadata::from_label("steady-run-5.5")).unwrap().metres(),
            0.03
        );
        assert_eq!(
            loop_seam_position_limit(&ScenarioMetadata::from_label("steady-walk")).unwrap().metres(),
            0.015
        );
    }

    #[test]
    fn terrain_run_stop_and_restart_require_the_strict_toe_clearance_contract() {
        assert!(scenario_requires_strict_terrain_toe_clearance(
            &ScenarioMetadata::authored(ScenarioId::TerrainSteadyRun55)
        ));
        assert!(scenario_requires_strict_terrain_toe_clearance(
            &ScenarioMetadata::authored(ScenarioId::TerrainRunFlightStop)
        ));
        assert!(scenario_requires_strict_terrain_toe_clearance(
            &ScenarioMetadata::authored(ScenarioId::TerrainTapRestartCrossfade)
        ));
        assert!(!scenario_requires_strict_terrain_toe_clearance(
            &ScenarioMetadata::authored(ScenarioId::TerrainTapStopForward)
        ));
    }

    #[test]
    fn transient_toe_gate_excludes_idle_preroll_but_keeps_motion_and_settle() {
        let moving_speed = SignedSpeed::from_metres_per_second(5.5).unwrap();
        for scenario in ["terrain-run-flight-stop", "terrain-tap-restart-crossfade"]
            .map(ScenarioMetadata::from_label)
        {
            assert!(!transition_flight_toe_sample_is_active(
                &scenario,
                SignedSpeed::ZERO,
                None
            ));
            assert!(transition_flight_toe_sample_is_active(
                &scenario,
                moving_speed,
                None
            ));
            assert!(transition_flight_toe_sample_is_active(
                &scenario,
                SignedSpeed::ZERO,
                Some(adventuresim_tactical_core::animation::SettleProgress::from_progress(0.25).unwrap())
            ));
        }
        assert!(transition_flight_toe_sample_is_active(
            &ScenarioMetadata::from_label("terrain-steady-run-5.5"),
            SignedSpeed::ZERO,
            None
        ));
        assert!(strict_transition_flight_toe_clearance_is_valid(fabelgeist_geometry::SignedLength::from_metres(0.01).unwrap()));
        assert!(!strict_transition_flight_toe_clearance_is_valid(fabelgeist_geometry::SignedLength::from_metres(-0.001).unwrap()));
    }

    #[test]
    fn manifest_metrics_detect_crossed_feet_and_facing_mismatch() {
        let bone = |position: Vec3| BoneSample {
            position: position.to_array(),
            rotation_xyzw: Quat::IDENTITY.to_array(),
            terrain_clearance_metres: Some(fabelgeist_geometry::SignedLength::from_metres(0.0).unwrap()),
        };
        let bones = CapturedBones::from_iter([
            (BoneRole::ThighLeft, bone(Vec3::new(-0.15, 1.0, 0.0))),
            (BoneRole::ShinLeft, bone(Vec3::new(-0.1, 0.5, 0.2))),
            (BoneRole::FootLeft, bone(Vec3::new(0.12, 0.0, 0.0))),
            (BoneRole::ThighRight, bone(Vec3::new(0.15, 1.0, 0.0))),
            (BoneRole::ShinRight, bone(Vec3::new(0.1, 0.5, 0.2))),
            (BoneRole::FootRight, bone(Vec3::new(-0.12, 0.0, 0.0))),
        ]);
        let mut frame = FrameSample {
            scenario: "metric-test".into(),
            scenario_frame: 0,
            time_seconds: AnimationElapsed::from_seconds(0.0).unwrap(),
            speed_metres_per_second: SignedSpeed::from_metres_per_second(2.0).unwrap(),
            gait_phase: CyclePhase::from_cycles(0.0).unwrap(),
            locomotion_sample_tick: AnimationSampleTick::from(0u64),
            body_acceleration: Vec3::ZERO.to_array(),
            world_acceleration: Vec3::ZERO.to_array(),
            bouncy_active_springs: 0,
            bouncy_maximum_deflection_radians: adventuresim_building_generator::spatial_geometry::Radians::ZERO,
            contact_sequence: ContactSequence::ZERO,
            contact_foot: LeadFoot::Left,
            landing_sequence: adventuresim_tactical_core::animation::LandingSequence::ZERO,
            landing_impact_speed: LandingImpactSpeed::ZERO,
            body_lean_pitch_degrees: Some(adventuresim_tactical_core::animation::SignedAngleDegrees::ZERO),
            body_lean_roll_degrees: Some(adventuresim_tactical_core::animation::SignedAngleDegrees::ZERO),
            landing_compression_metres: Some(HeightDistance::ZERO),
            root_distance_metres: fabelgeist_geometry::SignedLength::ZERO,
            root_position_metres: Vec3::ZERO.to_array(),
            world_travel_direction: Vec3::NEG_Z.to_array(),
            desired_body_forward_direction: Vec3::NEG_Z.to_array(),
            body_forward_direction: Vec3::Z.to_array(),
            body_rotation_xyzw: Quat::IDENTITY.to_array(),
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
            action: SkeletonAction::None,
            action_phase: adventuresim_tactical_core::animation::UnitInterval::ZERO,
            attack_animation: None,
            strike_family: StrikeFamily::Thrust,
            guard_action: false,
            left_support_weight: UnitInterval::from_fraction(1.0).unwrap(),
            right_support_weight: UnitInterval::from_fraction(1.0).unwrap(),
            desired_left_foot_target: None,
            desired_right_foot_target: None,
            ik_left_authored_target: None,
            ik_right_authored_target: None,
            ik_left_planned_contact: None,
            ik_right_planned_contact: None,
            ik_settle_capture_point: None,
            ik_left_solve_target: None,
            ik_right_solve_target: None,
            ik_left_support_weight: Some(UnitInterval::from_fraction(0.0).unwrap()),
            ik_right_support_weight: Some(UnitInterval::from_fraction(0.0).unwrap()),
            ik_left_release_active: Some(false),
            ik_right_release_active: Some(false),
            ik_left_release_target: None,
            ik_right_release_target: None,
            ik_settle_progress: None,
            ik_left_knee_foot_yaw_offset_degrees: Some(adventuresim_tactical_core::animation::AngularMagnitudeDegrees::ZERO),
            ik_right_knee_foot_yaw_offset_degrees: Some(adventuresim_tactical_core::animation::AngularMagnitudeDegrees::ZERO),
            semantic_route_requested_path: SemanticRoutePath::GeneralPose,
            semantic_route_selected_path: SemanticRoutePath::GeneralPose,
            semantic_route_runtime_evaluated: false,
            screenshots: BTreeMap::new(),
            bones,
        };
        let frames = [&frame];
        assert!(minimum_signed_foot_track(&frames).unwrap().measured().unwrap().metres() < 0.0);
        assert!(maximum_facing_error(&frames).unwrap().measured().unwrap().degrees() > 179.0);

        let mut previous = frame.clone();
        previous.desired_body_forward_direction = Vec3::Z.to_array();
        frame.scenario_frame = 1;
        frame.time_seconds = locomotion_sample_rate().interval().elapsed();
        assert!(maximum_facing_tracking_excess(&[&previous, &frame]).unwrap().measured().unwrap().degrees() > 8.0);

        frame.guard_action = true;
        assert!(maximum_guard_facing_error(&[&frame]).unwrap().measured().unwrap().degrees() > 179.0);

        frame.ik_left_knee_foot_yaw_offset_degrees = Some(adventuresim_tactical_core::animation::AngularMagnitudeDegrees::from_degrees(90.0).unwrap());
        assert!(maximum_knee_foot_yaw_offset(&[&frame]).unwrap().measured().unwrap().degrees() > 45.0);
    }

    #[test]
    fn hard_stop_height_gate_measures_only_the_transition_and_settle_window() {
        let frame = |scenario_frame: usize, speed: f32, pelvis_y: f32| {
            let mut frame = foot_metric_frame(scenario_frame, -0.1, 1.0, 0.0, 0.0);
            frame.scenario = "hard-stop".into();
            frame.speed_metres_per_second = SignedSpeed::from_metres_per_second(speed).unwrap();
            frame.bones.insert(
                BoneRole::Pelvis,
                BoneSample {
                    position: (Vec3::Y * pelvis_y).to_array(),
                    rotation_xyzw: Quat::IDENTITY.to_array(),
                    terrain_clearance_metres: None,
                },
            );
            frame
        };
        let frames = vec![
            frame(0, 5.5, 0.0),
            frame(1, 5.5, 0.03),
            frame(2, 5.5, 0.04),
            frame(3, 0.0, 0.055),
            frame(4, 0.0, 0.065),
        ];
        assert!(
            (root_relative_vertical_step(&frames.iter().collect::<Vec<_>>(), BoneRole::Pelvis)
                .unwrap().measured().unwrap().metres() - 0.03)
                .abs()
                < 0.0001
        );
        assert!((hard_stop_pelvis_vertical_step(&frames).unwrap().measured().unwrap().metres() - 0.015).abs() < 0.0001);
        assert_eq!(hard_stop_pelvis_vertical_step(&[]).unwrap(), Observation::NotApplicable);
    }

    #[test]
    fn cold_start_manifest_frame_requires_truthful_release_ownership() {
        let mut frame = foot_metric_frame(0, -0.1, 0.0, 0.0, 0.0);
        frame.scenario = "terrain-steady-run-5.5".into();
        frame.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        frame.ik_left_authored_target = Some(Vec3::ZERO.to_array());
        frame.ik_left_solve_target = Some((Vec3::Y * 0.095).to_array());
        frame.ik_right_authored_target = Some(Vec3::ZERO.to_array());
        frame.ik_right_solve_target = Some(Vec3::ZERO.to_array());
        assert!(!ordinary_swing_frame_is_owned(&frame));

        frame.ik_left_release_active = Some(true);
        assert!(ordinary_swing_frame_is_owned(&frame));
    }

    #[test]
    fn unplanned_release_transition_uses_run_specific_motion_budget() {
        let mut before = foot_metric_frame(0, -0.1, 0.0, 0.0, 0.0);
        let mut after = foot_metric_frame(1, -0.1, 0.0, 0.0, 0.0);
        before.scenario = "terrain-steady-run-5.5".into();
        after.scenario = before.scenario.clone();
        before.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        after.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        let origin = Some(Vec3::ZERO.to_array());
        let run_step = Some((Vec3::X * 0.0875).to_array());
        assert!(ordinary_unplanned_release_transition_is_valid(
            &before, &after, origin, run_step, None, None,
        ));

        let over_run_budget = Some((Vec3::X * 0.096).to_array());
        assert!(!ordinary_unplanned_release_transition_is_valid(
            &before,
            &after,
            origin,
            over_run_budget,
            None,
            None,
        ));

        before.speed_metres_per_second = SignedSpeed::from_metres_per_second(2.0).unwrap();
        after.speed_metres_per_second = SignedSpeed::from_metres_per_second(2.0).unwrap();
        assert!(!ordinary_unplanned_release_transition_is_valid(
            &before, &after, origin, run_step, None, None,
        ));
    }

    #[test]
    fn planned_run_transition_validates_metadata_and_body_local_motion() {
        let mut before = foot_metric_frame(0, -0.1, 0.0, 0.0, 0.0);
        let mut after = foot_metric_frame(1, -0.1, 0.0, 0.0, 0.0);
        before.scenario = "terrain-steady-run-5.5".into();
        after.scenario = before.scenario.clone();
        before.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        after.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        let plan = Some(Vec3::ZERO.to_array());
        let before_solve = Some((Vec3::X * 0.05).to_array());
        let bounded_run_solve = Some((Vec3::X * 0.1375).to_array());
        after.bones.get_mut(BoneRole::FootLeft).unwrap().position =
            Vec3::new(-0.0125, 0.0, 0.0).to_array();
        assert!(ordinary_planned_transition_is_valid(
            &before,
            &after,
            BoneRole::FootLeft,
            plan,
            plan,
            before_solve,
            bounded_run_solve,
            UnitInterval::from_fraction(0.0).unwrap(),
            false,
        ));

        let over_budget_solve = Some((Vec3::X * 0.146).to_array());
        after.bones.get_mut(BoneRole::FootLeft).unwrap().position =
            Vec3::new(-0.004, 0.0, 0.0).to_array();
        assert!(!ordinary_planned_transition_is_valid(
            &before,
            &after,
            BoneRole::FootLeft,
            plan,
            plan,
            before_solve,
            over_budget_solve,
            UnitInterval::from_fraction(0.0).unwrap(),
            false,
        ));

        before.speed_metres_per_second = SignedSpeed::from_metres_per_second(2.0).unwrap();
        after.speed_metres_per_second = SignedSpeed::from_metres_per_second(2.0).unwrap();
        assert!(!ordinary_planned_transition_is_valid(
            &before,
            &after,
            BoneRole::FootLeft,
            plan,
            plan,
            before_solve,
            bounded_run_solve,
            UnitInterval::from_fraction(0.0).unwrap(),
            false,
        ));

        before.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        after.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        after.bones.get_mut(BoneRole::FootLeft).unwrap().position =
            Vec3::new(-0.05, 0.0, 0.0).to_array();
        let retarget = Some((Vec3::X * 0.05).to_array());
        assert!(ordinary_planned_transition_is_valid(
            &before,
            &after,
            BoneRole::FootLeft,
            plan,
            retarget,
            plan,
            retarget,
            UnitInterval::from_fraction(0.205).unwrap(),
            false,
        ));

        let replacement = Some(Vec3::new(0.0, 0.0, -5.0).to_array());
        let hidden_solve_jump = Some((Vec3::X * 0.50).to_array());
        assert!(ordinary_planned_transition_is_valid(
            &before,
            &after,
            BoneRole::FootLeft,
            plan,
            replacement,
            before_solve,
            hidden_solve_jump,
            UnitInterval::from_fraction(0.0).unwrap(),
            true,
        ));
        after.bones.get_mut(BoneRole::FootLeft).unwrap().position =
            Vec3::new(-0.004, 0.0, 0.0).to_array();
        assert!(!ordinary_planned_transition_is_valid(
            &before,
            &after,
            BoneRole::FootLeft,
            plan,
            replacement,
            before_solve,
            hidden_solve_jump,
            UnitInterval::from_fraction(0.0).unwrap(),
            true,
        ));
    }

    #[test]
    fn planted_drift_excludes_the_deliberate_support_release_interval() {
        let frames = [
            foot_metric_frame(0, 0.0, 1.0, 0.0, 0.0),
            foot_metric_frame(1, 0.02, 1.0, 0.0, 0.0),
            foot_metric_frame(2, 0.20, 0.95, 0.0, 0.0),
        ];
        let references = frames.iter().collect::<Vec<_>>();

        assert!(
            (maximum_planted_foot_drift(
                &ScenarioMetadata::from_label("cross-slope-walk"),
                &references
            ).unwrap().measured().unwrap().metres() - 0.02)
                .abs()
                < 0.0001
        );
    }

    #[test]
    fn authored_guard_locomotion_does_not_require_procedural_step_liveness() {
        let mut frames = [
            foot_metric_frame(0, 0.0, 0.0, 0.0, 0.0),
            foot_metric_frame(1, 0.0, 0.0, 0.0, 0.0),
            foot_metric_frame(2, 0.0, 1.0, 0.0, 0.0),
        ];
        for frame in &mut frames {
            frame.scenario = "raised-guard-right".into();
            frame.weapon_guard = WeaponGuardState::Raised;
            frame.right_support_weight = if frame.scenario_frame < 2 {
                UnitInterval::ONE
            } else {
                UnitInterval::ZERO
            };
        }
        frames[2].contact_sequence = ContactSequence::from(1u64);
        frames[2].contact_foot = LeadFoot::Right;
        let references = frames.iter().collect::<Vec<_>>();

        let metrics = GuardStepLivenessMetrics::measure(&references).unwrap();

        assert!(matches!(metrics, Observation::NotApplicable));
    }

    #[test]
    fn contact_clearance_uses_effective_ik_support_instead_of_gait_phase() {
        let mut frame = foot_metric_frame(0, 0.0, 1.0, 0.0, 0.0);
        frame.ik_left_solve_target = Some(Vec3::ZERO.to_array());
        frame.ik_right_solve_target = Some(Vec3::ZERO.to_array());
        frame.ik_left_support_weight = Some(UnitInterval::from_fraction(0.0).unwrap());
        frame.ik_right_support_weight = Some(UnitInterval::from_fraction(1.0).unwrap());
        frame
            .bones
            .get_mut(BoneRole::FootLeft)
            .unwrap()
            .terrain_clearance_metres = Some(fabelgeist_geometry::SignedLength::from_metres(0.161).unwrap());
        frame
            .bones
            .get_mut(BoneRole::FootRight)
            .unwrap()
            .terrain_clearance_metres = Some(fabelgeist_geometry::SignedLength::from_metres(0.085).unwrap());

        // Gait phase zero identifies the left foot, but the solver reports it
        // airborne and the right foot supported. Only the supported sole is a
        // contact-clearance sample.
        let clearance = contact_sole_clearance_range(&[&frame]).unwrap();
        let range = clearance.measured().unwrap();
        assert_eq!((range.minimum().metres(), range.maximum().metres()), (0.0, 0.0));
    }

    #[test]
    fn contact_clearance_falls_back_to_phase_without_procedural_targets() {
        let mut frame = foot_metric_frame(0, 0.0, 0.0, 0.0, 0.0);
        frame
            .bones
            .get_mut(BoneRole::FootLeft)
            .unwrap()
            .terrain_clearance_metres = Some(fabelgeist_geometry::SignedLength::from_metres(0.095).unwrap());
        frame
            .bones
            .get_mut(BoneRole::FootRight)
            .unwrap()
            .terrain_clearance_metres = Some(fabelgeist_geometry::SignedLength::from_metres(0.145).unwrap());

        let clearance = contact_sole_clearance_range(&[&frame]).unwrap();
        let range = clearance.measured().unwrap();
        assert!((range.minimum().metres() - 0.01).abs() < 0.0001);
        assert!((range.maximum().metres() - 0.01).abs() < 0.0001);
    }

    #[test]
    fn viewer_rejects_reported_support_beyond_the_shared_contact_tolerance() {
        let mut frame = foot_metric_frame(0, 0.0, 0.0, 0.0, 0.0);
        frame.scenario = "terrain-steady-run-5.5".into();
        frame.ik_left_support_weight = Some(UnitInterval::from_fraction(1.0).unwrap());
        frame
            .bones
            .get_mut(BoneRole::FootLeft)
            .unwrap()
            .terrain_clearance_metres = Some(fabelgeist_geometry::SignedLength::from_metres(
            measured_ankle_sole_offset_metres().metres() + sole_contact_tolerance_metres().metres()
                - 0.00001,
        ).unwrap());
        assert!(reported_support_contacts_are_valid(&[frame.clone()]));

        frame
            .bones
            .get_mut(BoneRole::FootLeft)
            .unwrap()
            .terrain_clearance_metres = Some(fabelgeist_geometry::SignedLength::from_metres(
            measured_ankle_sole_offset_metres().metres()
                + sole_contact_tolerance_metres().metres()
                + 0.0001,
        ).unwrap());
        assert!(!reported_support_contacts_are_valid(&[frame]));
    }

    #[test]
    fn viewer_uses_action_support_instead_of_stale_ordinary_ik_support() {
        let mut frame = foot_metric_frame(0, 0.0, 0.0, 0.0, 0.0);
        frame.scenario = "attack-live-forward-left-support".into();
        frame.action = SkeletonAction::Attack;
        frame.ik_left_support_weight = Some(UnitInterval::from_fraction(1.0).unwrap());
        frame
            .bones
            .get_mut(BoneRole::FootLeft)
            .unwrap()
            .terrain_clearance_metres = Some(fabelgeist_geometry::SignedLength::from_metres(1.0).unwrap());
        assert!(reported_support_contacts_are_valid(&[frame]));
    }

    #[test]
    fn terrain_run_contact_gate_is_non_vacuous_and_requires_alternation() {
        let phase_step = gait_cycle_phase_delta(
            run_locomotion_profile(),
            run_locomotion_profile().reference_speed,
            locomotion_sample_rate().interval().elapsed(),
        )
        .unwrap()
        .cycles();
        let mut unsupported = (0..160)
            .map(|index| {
                let mut frame = foot_metric_frame(index, 0.0, 0.0, 0.0, 0.0);
                frame.scenario = "terrain-steady-run-5.5".into();
                frame.gait_phase =
                    CyclePhase::from_cycles((index as f32 * phase_step).rem_euclid(1.0)).unwrap();
                frame.time_seconds =
                    AnimationElapsed::from_seconds(index as f32 / locomotion_sample_rate().hertz())
                        .unwrap();
                frame
            })
            .collect::<Vec<_>>();
        assert!(!terrain_run_contacts_are_valid(&unsupported).unwrap());

        for frame in &mut unsupported {
            let phase = frame.gait_phase.cycles();
            let left_distance = phase.min(1.0 - phase);
            let right_distance = (phase - 0.5).abs();
            frame.ik_left_support_weight = Some(if left_distance <= 0.17 {
                UnitInterval::ONE
            } else {
                UnitInterval::ZERO
            });
            frame.ik_right_support_weight = Some(if right_distance <= 0.17 {
                UnitInterval::ONE
            } else {
                UnitInterval::ZERO
            });
            frame.left_support_weight = frame.ik_left_support_weight.unwrap();
            frame.right_support_weight = frame.ik_right_support_weight.unwrap();
        }
        assert!(terrain_run_contacts_are_valid(&unsupported).unwrap());
    }

    #[test]
    fn continuity_bone_classes_separate_feet_from_knees() {
        assert!(is_foot_bone(&BoneRole::FootLeft));
        assert!(is_foot_bone(&BoneRole::FootRight));
        assert!(!is_foot_bone(&BoneRole::ShinLeft));
        assert!(is_knee_bone(&BoneRole::ShinLeft));
        assert!(is_knee_bone(&BoneRole::ShinRight));
        assert!(!is_knee_bone(&BoneRole::ThighRight));
    }

    #[test]
    fn terrain_vertical_allowance_is_derived_from_sampled_foot_relief() {
        let frame = foot_metric_frame(0, 0.0, 1.0, -0.04, 0.04);

        assert!((foot_terrain_relief(&[&frame]).unwrap().measured().unwrap().metres() - 0.08).abs() < 0.0001);
        assert_eq!(
            vertical_range_limit(&ScenarioMetadata::from_label("steady-walk"), HeightDistance::from_metres(0.08).unwrap()).unwrap().metres(),
            ORDINARY_VERTICAL_RANGE_LIMIT_METRES
        );
        assert!(
            (vertical_range_limit(&ScenarioMetadata::from_label("cross-slope-walk"), HeightDistance::from_metres(0.08).unwrap()).unwrap().metres() - 0.28)
                .abs()
                < 0.0001
        );
    }

    #[test]
    fn pelvis_vertical_step_detects_a_one_frame_guard_height_snap() {
        let mut first = foot_metric_frame(0, 0.0, 1.0, 0.0, 0.0);
        let mut second = foot_metric_frame(1, 0.0, 1.0, 0.0, 0.0);
        let pelvis = |height: f32| BoneSample {
            position: Vec3::new(0.0, height, 0.0).to_array(),
            rotation_xyzw: Quat::IDENTITY.to_array(),
            terrain_clearance_metres: None,
        };
        first.bones.insert(BoneRole::Pelvis, pelvis(1.0));
        second.bones.insert(BoneRole::Pelvis, pelvis(0.86));

        assert!(
            (root_relative_vertical_step(&[&first, &second], BoneRole::Pelvis)
                .unwrap().measured().unwrap().metres() - 0.14).abs()
                < 0.0001
        );
    }

    #[test]
    fn prominent_height_gate_requires_only_the_two_passing_peaks() {
        let mut clean = (0..64)
            .map(|sample| {
                let phase = sample as f32 / 64.0;
                let sine = (phase * std::f32::consts::TAU).sin();
                HeightSample::new(
                    CyclePhase::from_cycles(phase).unwrap(),
                    fabelgeist_geometry::SignedLength::from_metres(0.04 * sine * sine).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(prominent_height_peaks(&clean).unwrap(), Observation::Measured(HeightPeaks { count: 2, in_passing_windows: true }));

        // A visible contact beat is a third gait-height peak even though the
        // intended passing peaks remain correctly positioned.
        clean[0] = HeightSample::new(clean[0].phase(), fabelgeist_geometry::SignedLength::from_metres(0.01).unwrap());
        let measurement = prominent_height_peaks(&clean).unwrap();
        let peaks = measurement.measured().unwrap();
        assert!(peaks.in_passing_windows);
        assert_ne!(peaks.count, 2);
    }

    #[test]
    fn release_transition_is_not_misclassified_as_a_repeatable_loop() {
        assert!(!expects_loop_seam(&ScenarioMetadata::from_label(
            "raised-guard-release-at-peak"
        )));
        assert!(expects_loop_seam(&ScenarioMetadata::from_label(
            "raised-guard-forward"
        )));
        assert!(expects_loop_seam(&ScenarioMetadata::from_label(
            "steady-walk"
        )));
    }
}

#[cfg(test)]
mod scenario_policy_tests {
    use super::*;

    #[test]
    fn raised_guard_uses_strict_plant_and_separation_gates() {
        assert_eq!(
            inter_foot_separation_limit(&ScenarioMetadata::from_label("quickstep-right")).unwrap().metres(),
            0.04
        );
        assert_eq!(
            planted_drift_limit(&ScenarioMetadata::from_label("raised-guard-right")).unwrap().metres(),
            0.01
        );
        assert_eq!(
            inter_foot_separation_limit(&ScenarioMetadata::from_label("raised-guard-right")).unwrap().metres(),
            0.08
        );
        assert_eq!(
            inter_foot_separation_limit(&ScenarioMetadata::from_label(
                "raised-guard-stationary-turn"
            )).unwrap().metres(),
            0.16
        );
        assert_eq!(
            planted_drift_limit(&ScenarioMetadata::from_label("steady-walk-2.0")).unwrap().metres(),
            0.035
        );
        assert_eq!(
            inter_foot_separation_limit(&ScenarioMetadata::from_label("steady-walk-2.0")).unwrap().metres(),
            0.08
        );
        assert!(!procedural_leg_solver_gates_apply(
            &ScenarioMetadata::from_label("steady-walk-2.0")
        ));
        assert!(!procedural_leg_solver_gates_apply(
            &ScenarioMetadata::from_label("start-stop-transition")
        ));
        assert!(procedural_leg_solver_gates_apply(
            &ScenarioMetadata::from_label("cross-slope-walk")
        ));
        assert!(!procedural_leg_solver_gates_apply(
            &ScenarioMetadata::from_label("raised-guard-forward")
        ));
        assert!(procedural_leg_solver_gates_apply(
            &ScenarioMetadata::authored(ScenarioId::RaisedGuardStationaryTurn)
        ));
        assert!(!procedural_leg_solver_gates_apply(
            &ScenarioMetadata::authored(ScenarioId::RaisedGuardReleaseAtPeak)
        ));
        assert!(!procedural_leg_solver_gates_apply(
            &ScenarioMetadata::authored(ScenarioId::RaisedGuardTapStopLeft)
        ));
        assert!(ScenarioMetadata::from_label("raised-guard-release-at-peak").procedural_solver());
        assert!(ScenarioMetadata::from_label("raised-guard-tap-stop-left").procedural_solver());
        assert!(!procedural_leg_solver_gates_apply(
            &ScenarioMetadata::authored(ScenarioId::RaisedGuardLeftRightReversal)
        ));
        for transition in [
            "raised-guard-release-at-peak",
            "raised-guard-right-support-release",
            "raised-guard-left-right-reversal",
            "raised-guard-right-support-reversal",
            "raised-guard-accelerate-from-rest",
            "terrain-tap-restart-crossfade",
            "terrain-speed-threshold-chatter",
        ]
        .map(ScenarioMetadata::from_label)
        {
            assert!(!expects_loop_seam(&transition));
        }
    }
}
