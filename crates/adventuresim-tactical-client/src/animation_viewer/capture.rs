//! ECS fixture driving, sampling, and three-view capture.

use super::*;
use admission::{RejectedCaptureAdmission, fail_capture};
use frame_clock::{capture_frame_elapsed, next_capture_simulation_tick};

mod admission;
mod frame_clock;
mod readiness;
mod subject_motion;

pub(super) const CAPTURE_ROOT_GROUND_OFFSET_METRES: f32 = 0.95;
pub(super) const FULL_PLANT_SUPPORT_WEIGHT: f32 = 0.99;

#[derive(Component)]
pub(super) struct CaptureSubject;

#[derive(Component)]
pub(super) struct CaptureLabel;

#[derive(Resource)]
pub(super) struct CaptureSequence {
    output: PathBuf,
    settle_frames: u32,
    plan: Vec<PlannedFrame>,
    index: usize,
    view_index: usize,
    applied: bool,
    rejection: Option<RejectedCaptureAdmission>,
    settled: u32,
    waiting: u32,
    capture_in_flight: bool,
    view_fingerprints: Vec<u64>,
    duplicate_view_frames: Vec<String>,
    samples: Vec<FrameSample>,
    global_bone_frames: Vec<GlobalBoneFrame>,
    presentation_events: Vec<PresentationEventSample>,
    repeated_evaluation_baseline: Option<RepeatedEvaluationSnapshot>,
    repeated_evaluation_valid: bool,
    active_scenario: Option<ScenarioMetadata<'static>>,
    warmup_frames: u32,
    motion_ready_frames: u32,
    simulation_tick: AnimationSampleTick,
    scenario_distance: f32,
}

pub(super) struct CompletedCapture {
    pub(super) output: PathBuf,
    pub(super) plan: Vec<PlannedFrame>,
    pub(super) frames: Vec<FrameSample>,
    pub(super) global_bone_frames: Vec<GlobalBoneFrame>,
    pub(super) presentation_events: Vec<PresentationEventSample>,
    pub(super) duplicate_view_frames: Vec<String>,
    pub(super) repeated_evaluation_valid: bool,
    pub(super) playback_backend: &'static str,
    pub(super) pose_buffer_metrics: PoseBufferMetrics,
}

impl CaptureSequence {
    pub(super) fn new(output: PathBuf, settle_frames: u32, scenario: Option<&str>) -> Self {
        let selected = scenario.map(|label| ScenarioMetadata::from_label(label).into_owned());
        let plan = match selected.as_ref().and_then(ScenarioMetadata::identity) {
            Some(ScenarioId::FlatGridWalk20) => steady_scenario(
                ScenarioMetadata::authored(ScenarioId::FlatGridWalk20),
                2.0,
                3.0,
            ),
            Some(ScenarioId::FlatGridRun55) => steady_scenario(
                ScenarioMetadata::authored(ScenarioId::FlatGridRun55),
                5.5,
                3.0,
            ),
            Some(ScenarioId::FlatGridWalkNoIk) => steady_scenario(
                ScenarioMetadata::authored(ScenarioId::FlatGridWalkNoIk),
                2.0,
                3.0,
            ),
            Some(ScenarioId::FlatGridSprintNoIk) => steady_scenario(
                ScenarioMetadata::authored(ScenarioId::FlatGridSprintNoIk),
                canonical_john_sprint_speed(),
                3.0,
            ),
            Some(ScenarioId::FlatGridWalkStop) => flat_grid_walk_stop_scenario(),
            Some(ScenarioId::FullRagdoll) => full_ragdoll_scenario(),
            _ => capture_plan()
                .into_iter()
                .filter(|frame| {
                    selected
                        .as_ref()
                        .is_none_or(|scenario| &frame.scenario == scenario)
                })
                .collect(),
        };
        assert!(
            !plan.is_empty(),
            "requested animation capture scenario is unknown"
        );
        Self {
            output,
            settle_frames: settle_frames.max(1),
            plan,
            index: 0,
            view_index: 0,
            applied: false,
            rejection: None,
            settled: 0,
            waiting: 0,
            capture_in_flight: false,
            view_fingerprints: Vec::with_capacity(VIEWS.len()),
            duplicate_view_frames: Vec::new(),
            samples: Vec::new(),
            global_bone_frames: Vec::new(),
            presentation_events: Vec::new(),
            repeated_evaluation_baseline: None,
            repeated_evaluation_valid: true,
            active_scenario: None,
            warmup_frames: 0,
            motion_ready_frames: 0,
            simulation_tick: AnimationSampleTick::ZERO,
            scenario_distance: 0.0,
        }
    }

    pub(super) fn view(&self) -> CaptureView {
        VIEWS[self.view_index.min(VIEWS.len() - 1)]
    }

    pub(super) fn applied_frame(&self) -> Option<&PlannedFrame> {
        self.applied.then(|| self.plan.get(self.index)).flatten()
    }

    pub(super) fn uses_flat_grid(&self) -> bool {
        self.plan
            .iter()
            .all(|frame| frame.scenario.has(ScenarioFeature::FlatGrid))
    }

    fn complete(
        &mut self,
        playback_backend: &'static str,
        pose_buffer_metrics: PoseBufferMetrics,
    ) -> CompletedCapture {
        CompletedCapture {
            output: std::mem::take(&mut self.output),
            plan: std::mem::take(&mut self.plan),
            frames: std::mem::take(&mut self.samples),
            global_bone_frames: std::mem::take(&mut self.global_bone_frames),
            presentation_events: std::mem::take(&mut self.presentation_events),
            duplicate_view_frames: std::mem::take(&mut self.duplicate_view_frames),
            repeated_evaluation_valid: self.repeated_evaluation_valid,
            playback_backend,
            pose_buffer_metrics,
        }
    }
}

pub(super) struct RepeatedEvaluationSnapshot {
    scenario: ScenarioMetadata<'static>,
    scenario_frame: usize,
    bones: CapturedBones,
    contact_sequence: ContactSequence,
    landing_sequence: adventuresim_tactical_core::animation::LandingSequence,
    event_count: usize,
    leg_ik: LegIkDiagnostics,
}

pub(super) fn repeated_bone_mismatch(
    expected: &CapturedBones,
    actual: &CapturedBones,
) -> Option<(String, f32, f32)> {
    if let Some(name) = expected
        .keys()
        .find(|name| !actual.contains_key(*name))
        .or_else(|| actual.keys().find(|name| !expected.contains_key(*name)))
    {
        return Some((
            report_bone_label(*name)
                .map(str::to_owned)
                .unwrap_or_else(|| format!("{name:?}")),
            f32::INFINITY,
            f32::INFINITY,
        ));
    }
    expected.iter().find_map(|(name, expected)| {
        let actual = actual
            .get(name)
            .expect("equal repeated-evaluation bone keys were checked above");
        let position_delta =
            Vec3::from_array(expected.position).distance(Vec3::from_array(actual.position));
        let expected_rotation = Quat::from_array(expected.rotation_xyzw);
        let actual_rotation = Quat::from_array(actual.rotation_xyzw);
        // `acos` quantizes identical f32 quaternions to roughly 0.056-0.079
        // degrees on some frames. Treat a direct dot match as identity, then
        // retain the angular report for genuine changes.
        let rotation_dot = expected_rotation.dot(actual_rotation).abs();
        let rotation_delta = if rotation_dot >= 1.0 - 0.000_001 {
            0.0
        } else {
            expected_rotation
                .angle_between(actual_rotation)
                .to_degrees()
        };
        // Re-evaluating one logical tick must be visually identical. Keep only
        // sub-half-millimetre/sub-twentieth-degree numeric noise.
        (position_delta > 0.0005 || rotation_delta > 0.05).then_some((
            name.clone(),
            position_delta,
            rotation_delta,
        ))
    })
}

pub(super) fn repeated_leg_ik_matches(
    expected: LegIkDiagnostics,
    actual: LegIkDiagnostics,
) -> bool {
    let option_vec3_matches =
        |expected: Option<Vec3>, actual: Option<Vec3>| match (expected, actual) {
            (Some(expected), Some(actual)) => expected.distance(actual) <= 0.0005,
            (None, None) => true,
            _ => false,
        };
    let option_scalar_matches =
        |expected: Option<f32>, actual: Option<f32>| match (expected, actual) {
            (Some(expected), Some(actual)) => (expected - actual).abs() <= 0.001,
            (None, None) => true,
            _ => false,
        };
    option_vec3_matches(expected.left_authored_target, actual.left_authored_target)
        && option_vec3_matches(expected.right_authored_target, actual.right_authored_target)
        && option_vec3_matches(expected.left_planned_contact, actual.left_planned_contact)
        && option_vec3_matches(expected.right_planned_contact, actual.right_planned_contact)
        && option_vec3_matches(expected.settle_capture_point, actual.settle_capture_point)
        && option_vec3_matches(expected.left_solve_target, actual.left_solve_target)
        && option_vec3_matches(expected.right_solve_target, actual.right_solve_target)
        && (expected.left_support_weight - actual.left_support_weight).abs() <= 0.001
        && (expected.right_support_weight - actual.right_support_weight).abs() <= 0.001
        && expected.left_release_active == actual.left_release_active
        && expected.right_release_active == actual.right_release_active
        && option_vec3_matches(expected.left_release_target, actual.left_release_target)
        && option_vec3_matches(expected.right_release_target, actual.right_release_target)
        && option_scalar_matches(expected.settle_progress, actual.settle_progress)
        && (expected.left_knee_foot_yaw_offset_degrees - actual.left_knee_foot_yaw_offset_degrees)
            .abs()
            <= 0.05
        && (expected.right_knee_foot_yaw_offset_degrees - actual.right_knee_foot_yaw_offset_degrees)
            .abs()
            <= 0.05
}

/// Screenshot completion advances capture views asynchronously. Reassert the
/// planned look on every render of a logical sample so all three views enter
/// procedural PostUpdate with identical presentation input.
pub(super) fn freeze_capture_look(
    sequence: Res<CaptureSequence>,
    mut subjects: Query<&mut CharacterLook, With<CaptureSubject>>,
) {
    let Some(frame) = sequence.plan.get(sequence.index) else {
        return;
    };
    for mut look in &mut subjects {
        look.yaw = frame.camera_yaw.radians();
        look.pitch = frame.camera_pitch.radians();
    }
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "the Bevy capture driver independently controls timing, IK modes, animation input, subject state, and labels"
)]
pub(super) fn drive_sequence(
    mut sequence: ResMut<CaptureSequence>,
    mut procedural_clock: ResMut<ProceduralAnimationClock>,
    mut terrain_ik: ResMut<TerrainIkMode>,
    mut guard_input: ResMut<WeaponGuardInputState>,
    animation_runtime: Res<AnimationRuntime>,
    terrain: Single<&SceneTerrain>,
    mut subjects: Query<
        (
            &mut SkeletonState,
            &mut Transform,
            &mut CharacterLook,
            Option<&AnimationPlayback>,
            Option<&mut LegIkState>,
            Option<&mut ArmIkState>,
            Option<&mut RaisedFootworkState>,
            Option<&mut LocomotionHeightState>,
            Option<&mut LocomotionBodyResponseState>,
        ),
        With<CaptureSubject>,
    >,
    mut labels: Query<&mut Text, With<CaptureLabel>>,
) {
    if sequence.applied || sequence.capture_in_flight || sequence.index >= sequence.plan.len() {
        return;
    }
    let frame = sequence.plan[sequence.index].clone();
    let metadata = frame.scenario.clone();
    let mut gait_phase = CyclePhase::ZERO;
    let mut observed_playback = None;
    for (
        mut skeleton,
        mut transform,
        mut look,
        playback,
        ik_state,
        arm_ik_state,
        raised_footwork,
        height_state,
        body_response,
    ) in &mut subjects
    {
        let Some(playback) = playback else {
            return;
        };
        observed_playback = Some(playback);
        if !frame.scenario.is(ScenarioId::FullRagdoll) && !playback.authored_pose_is_ready() {
            return;
        }

        if sequence.active_scenario.as_ref() != Some(&frame.scenario) {
            sequence.active_scenario = Some(frame.scenario.clone());
            sequence.scenario_distance = 0.0;
            sequence.motion_ready_frames = 0;
            *skeleton = SkeletonState::default();
            *guard_input = WeaponGuardInputState::default();
            if let Some(mut ik_state) = ik_state {
                *ik_state = LegIkState::default();
            }
            if let Some(mut arm_ik_state) = arm_ik_state {
                *arm_ik_state = ArmIkState::default();
            }
            if let Some(mut raised_footwork) = raised_footwork {
                *raised_footwork = RaisedFootworkState::default();
            }
            if let Some(mut height_state) = height_state {
                *height_state = LocomotionHeightState::default();
            }
            if let Some(mut body_response) = body_response {
                *body_response = LocomotionBodyResponseState::default();
            }
            let ground = terrain.height_at(Vec2::ZERO).unwrap_or_default();
            transform.translation = Vec3::new(0.0, ground + CAPTURE_ROOT_GROUND_OFFSET_METRES, 0.0);
            transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
            if frame.scenario.has(ScenarioFeature::RaisedTapStop) {
                // Prime the static combat stance and its terrain-conformed IK
                // before judging the later six-frame movement tap. A live
                // character has already held this stance; capturing component
                // allocation and pelvis acquisition would test viewer spawn,
                // not the authored-to-procedural stop transition.
                sequence.warmup_frames = 8;
            }
            if let Some((start_body, _)) = frame.scenario.transition() {
                skeleton.transition_body(start_body);
                // Prime the authored endpoint before beginning the transition.
                // Live characters have already evaluated their prone/supine or
                // upright base; a fresh viewer subject otherwise crossfades
                // from its default standing pose during the first samples.
                sequence.warmup_frames = 8;
            }
        }

        if let Some(body) = frame.scenario.body() {
            skeleton.transition_body(body);
            let preload_locomotion = frame.scenario_frame == 0
                && frame.scenario.required_motion().is_some_and(|motion| {
                    *motion == crate::animation::MotionId::PRONE_CRAWL
                        || *motion == crate::animation::MotionId::SUPINE_SCAMPER
                })
                && !frame
                    .scenario
                    .required_motion()
                    .is_some_and(|motion| animation_runtime.motion_is_processed(&motion));
            skeleton.set_downed_motion(
                if !frame.scenario.is(ScenarioId::DownedProneLookAt)
                    && (preload_locomotion || (frame.scenario_frame >= 4 && frame.speed <= 0.05))
                {
                    DownedMotion::Turning
                } else {
                    DownedMotion::Resting
                },
            );
        }

        let orientation = Quat::from_euler(
            EulerRot::YXZ,
            frame.camera_yaw.radians(),
            frame.camera_pitch.radians(),
            0.0,
        );
        look.yaw = frame.camera_yaw.radians();
        look.pitch = frame.camera_pitch.radians();
        let attack_start_frame = frame
            .scenario
            .label()
            .starts_with("attack-live-")
            .then_some(8);
        if frame.action != SkeletonAction::Attack
            || attack_start_frame == Some(frame.scenario_frame)
        {
            skeleton.lead_foot = frame.lead_foot;
        }
        *terrain_ik = terrain_ik_mode_for_frame(&frame);
        guard_input.desired = frame.weapon_guard;
        set_weapon_guard(&mut skeleton, guard_input.desired);
        if frame.scenario.is(ScenarioId::DownedProneLookAt) {
            let target = downed_camera_roll_target(transform.rotation, orientation);
            skeleton.advance_downed_facing(
                target,
                true,
                if frame.scenario_frame == 0 {
                    0.0
                } else {
                    1.0 / 84.0
                },
            );
        }
        let dive_impact = frame.scenario.has(ScenarioFeature::Impact);
        let quickstep = is_quickstep_scenario(&frame.scenario);
        let grounded = if quickstep {
            frame.scenario_frame < quickstep_release_frame()
                || frame.scenario_frame >= quickstep_landing_frame()
        } else if dive_impact {
            frame.scenario_frame == 0 || frame.scenario_frame >= 17
        } else {
            metadata.kind() != ScenarioKind::Landing || frame.scenario_frame >= 32
        };
        let vertical_velocity = if quickstep && frame.scenario_frame < quickstep_landing_frame() {
            quickstep_fixture_vertical_state(frame.scenario_frame).1
        } else if (metadata.kind() == ScenarioKind::Landing || dive_impact) && !grounded {
            -4.5
        } else {
            0.0
        };
        let requested_local_velocity = Vec3::new(
            frame.local_direction.x * frame.speed,
            vertical_velocity,
            frame.local_direction.y * frame.speed,
        );
        let local_velocity = requested_local_velocity;
        let world_velocity = controller_yaw(orientation) * local_velocity;
        sequence.simulation_tick = next_capture_simulation_tick(
            sequence.simulation_tick,
            sequence.index == 0 && sequence.warmup_frames == 0,
        );
        if sequence.warmup_frames == 0
            && frame.scenario_frame == 0
            && let Some((start_body, transition)) = frame.scenario.transition()
        {
            skeleton.transition_body(start_body);
            if frame.scenario.has(ScenarioFeature::AimedImpact) {
                // Match the authoritative launch seam: velocity and authored
                // direction capture one camera frame even if the previously
                // displayed root had not finished turning toward it.
                transform.rotation =
                    dive_launch_root_rotation(Quat::from_rotation_y(frame.camera_yaw.radians()));
            }
            // Matches the live server's terrain-contact dive recovery.
            let duration = if frame.scenario.has(ScenarioFeature::DiveBackward) {
                32
            } else if dive_impact {
                20
            } else {
                84
            };
            skeleton.begin_posture_transition(
                transition,
                sequence.simulation_tick,
                AnimationSampleSpan::from(duration as u64),
            );
        }
        let elapsed = match capture_frame_elapsed(frame.scenario_frame) {
            Ok(elapsed) => elapsed,
            Err(error) => {
                sequence.rejection = Some(RejectedCaptureAdmission::FrameInterval(error));
                return;
            }
        };
        let delta_seconds = elapsed.seconds();
        procedural_clock.set_fixed_step(sequence.simulation_tick, elapsed);
        let horizontal = transform.translation.xz() + world_velocity.xz() * delta_seconds;
        let vertical = if quickstep {
            let ground = terrain.height_at(horizontal).unwrap_or_default()
                + CAPTURE_ROOT_GROUND_OFFSET_METRES;
            ground
                + quickstep_fixture_vertical_state(frame.scenario_frame)
                    .0
                    .max(0.0)
        } else if terrain_ik.is_enabled() {
            terrain.height_at(horizontal).unwrap_or_default() + CAPTURE_ROOT_GROUND_OFFSET_METRES
        } else {
            transform.translation.y
        };
        transform.translation = Vec3::new(horizontal.x, vertical, horizontal.y);
        subject_motion::begin_capture_action(
            &mut skeleton,
            frame,
            sequence.simulation_tick,
            attack_start_frame,
        );
        if !skeleton.is_posture_transitioning() {
            transform.rotation = advance_body_facing(
                transform.rotation,
                orientation,
                world_velocity,
                frame.action,
                skeleton.weapon_guard(),
                delta_seconds,
            );
        }
        if frame.scenario.is(ScenarioId::FullRagdoll) {
            let fall = ((frame.scenario_frame + 4) as f32 / 8.0).clamp(0.0, 1.0);
            transform.rotation = Quat::from_rotation_y(std::f32::consts::PI)
                * Quat::from_rotation_x(1.25 * smoothstep01(fall));
            // The production root becomes a coarse dynamic body while the
            // client ragdoll is active. Reproduce its descent in this focused
            // viewer so the first sixty-frame settle window actually exercises
            // whole-body terrain contact instead of suspending the pelvis at
            // the standing controller height.
            let ground = terrain.height_at(Vec2::ZERO).unwrap_or_default();
            transform.translation.y =
                ground + CAPTURE_ROOT_GROUND_OFFSET_METRES - 0.62 * smoothstep01(fall);
        }
        sequence.scenario_distance += frame.speed * delta_seconds;
        if let Err(error) = subject_motion::project_capture_motion(
            &mut skeleton,
            &mut transform,
            &frame,
            subject_motion::CaptureMotion {
                orientation,
                world_velocity,
                contact: GroundContact::from(grounded),
                elapsed,
            },
            &sequence,
        ) {
            sequence.rejection = Some(RejectedCaptureAdmission::Locomotion(error));
            return;
        }
        gait_phase = skeleton.gait_phase;
    }
    if !sequence.ready_for_frame(&frame, &animation_runtime, observed_playback) {
        return;
    }
    for mut label in &mut labels {
        **label = format!(
            "{} | {:>4.2} m/s | phase {:>5.3} | {} view | 64 Hz frame {}",
            frame.scenario.label(),
            frame.speed,
            gait_phase.cycles(),
            VIEWS[sequence.view_index].slug(),
            frame.scenario_frame,
        );
    }
    sequence.applied = true;
    sequence.settled = 0;
}

pub(super) fn draw_skeleton_overlay(
    sequence: Res<CaptureSequence>,
    mut gizmos: Gizmos,
    subjects: Query<(Entity, &PresentedSkeleton), With<CaptureSubject>>,
    bones: Query<(&HumanoidBone, &GlobalTransform)>,
) {
    if matches!(
        VIEWS[sequence.view_index.min(VIEWS.len() - 1)],
        CaptureView::Gameplay
    ) {
        return;
    }
    let Ok((subject, skeleton)) = subjects.single() else {
        return;
    };
    let positions = bones
        .iter()
        .filter(|(bone, _)| bone.owner == subject)
        .map(|(bone, transform)| (bone.role, transform.translation()))
        .collect::<BTreeMap<_, _>>();
    let connections = [
        (BoneRole::Pelvis, BoneRole::Chest),
        (BoneRole::Chest, BoneRole::Head),
        (BoneRole::Chest, BoneRole::UpperArmLeft),
        (BoneRole::UpperArmLeft, BoneRole::ForearmLeft),
        (BoneRole::ForearmLeft, BoneRole::HandLeft),
        (BoneRole::Chest, BoneRole::UpperArmRight),
        (BoneRole::UpperArmRight, BoneRole::ForearmRight),
        (BoneRole::ForearmRight, BoneRole::HandRight),
        (BoneRole::Pelvis, BoneRole::ThighLeft),
        (BoneRole::ThighLeft, BoneRole::ShinLeft),
        (BoneRole::ShinLeft, BoneRole::FootLeft),
        (BoneRole::Pelvis, BoneRole::ThighRight),
        (BoneRole::ThighRight, BoneRole::ShinRight),
        (BoneRole::ShinRight, BoneRole::FootRight),
    ];
    for (start, end) in connections {
        if let (Some(&start), Some(&end)) = (positions.get(&start), positions.get(&end)) {
            gizmos.line(start, end, Color::srgba(0.1, 0.9, 1.0, 0.8));
        }
    }
    let (left_support, right_support) = locomotion_support_weights(skeleton);
    for (role, support) in [
        (BoneRole::FootLeft, left_support),
        (BoneRole::FootRight, right_support),
    ] {
        let Some(&position) = positions.get(&role) else {
            continue;
        };
        let color = if support >= 0.55 {
            Color::srgb(1.0, 0.8, 0.05)
        } else {
            Color::srgb(1.0, 0.1, 0.7)
        };
        gizmos.line(position - Vec3::X * 0.09, position + Vec3::X * 0.09, color);
        gizmos.line(position - Vec3::Z * 0.09, position + Vec3::Z * 0.09, color);
        gizmos.line(
            position,
            position + Vec3::Y * (0.05 + support * 0.15),
            color,
        );
    }
}

pub(super) fn draw_flat_ground_grid(sequence: Res<CaptureSequence>, mut gizmos: Gizmos) {
    let Some(frame) = sequence.plan.get(sequence.index) else {
        return;
    };
    if !frame.scenario.has(ScenarioFeature::FlatGrid) {
        return;
    }

    const HALF_EXTENT_METRES: i32 = 20;
    const SUBDIVISIONS_PER_METRE: i32 = 4;
    let half_steps = HALF_EXTENT_METRES * SUBDIVISIONS_PER_METRE;
    let height = 0.012;
    for step in -half_steps..=half_steps {
        let coordinate = step as f32 / SUBDIVISIONS_PER_METRE as f32;
        let whole_metre = step % SUBDIVISIONS_PER_METRE == 0;
        let color = if step == 0 {
            Color::srgba(1.0, 0.45, 0.12, 0.95)
        } else if whole_metre {
            Color::srgba(0.82, 0.86, 0.92, 0.80)
        } else {
            Color::srgba(0.42, 0.47, 0.55, 0.48)
        };
        gizmos.line(
            Vec3::new(coordinate, height, -HALF_EXTENT_METRES as f32),
            Vec3::new(coordinate, height, HALF_EXTENT_METRES as f32),
            color,
        );
        gizmos.line(
            Vec3::new(-HALF_EXTENT_METRES as f32, height, coordinate),
            Vec3::new(HALF_EXTENT_METRES as f32, height, coordinate),
            color,
        );
    }
}

pub(super) fn collect_locomotion_presentation_events(
    mut events: MessageReader<LocomotionPresentationEvent>,
    mut sequence: ResMut<CaptureSequence>,
) {
    if sequence.index >= sequence.plan.len() {
        return;
    }
    let scenario = sequence.plan[sequence.index].scenario.clone();
    let scenario_frame = sequence.plan[sequence.index].scenario_frame;
    sequence
        .presentation_events
        .extend(events.read().map(move |event| PresentationEventSample {
            scenario: scenario.clone(),
            scenario_frame,
            owner: capture_entity_id(event.owner),
            sample_tick: event.sample_tick,
            payload: event.payload,
        }));
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "the Bevy capture system records each independent animation state and diagnostic source"
)]
pub(super) fn capture_frame(
    mut commands: Commands,
    mut sequence: ResMut<CaptureSequence>,
    pose_buffer_metrics: Res<PoseBufferMetrics>,
    bouncy_bones_telemetry: Res<BouncyBonesTelemetry>,
    terrain_ik: Res<TerrainIkMode>,
    subjects: Query<
        (
            Entity,
            &PresentedSkeleton,
            &GlobalTransform,
            Option<&AnimationPlayback>,
            Option<&RaisedFootworkState>,
            Option<&LocomotionBodyResponseState>,
            Option<&LocomotionHeightState>,
            Option<&LegIkState>,
            Option<&SemanticRouteTrace>,
        ),
        With<CaptureSubject>,
    >,
    bones: Query<(&HumanoidBone, &GlobalTransform)>,
    animation_bones: Query<(
        &AnimationTargetId,
        &AuthoredBindTransform,
        Option<&Name>,
        &GlobalTransform,
    )>,
    terrain: Single<&SceneTerrain>,
    mut exit: MessageWriter<AppExit>,
) {
    let playback_backend = "pose_buffer";
    let pose_buffer_metrics = *pose_buffer_metrics;
    if let Some(rejection) = sequence.rejection.take() {
        fail_capture(&sequence, &rejection.message(), &mut exit);
        return;
    }
    if !sequence.applied || sequence.capture_in_flight {
        return;
    }
    let Ok((
        subject,
        skeleton,
        subject_global,
        playback,
        raised_footwork,
        body_response,
        height_state,
        leg_ik,
        semantic_route,
    )) = subjects.single()
    else {
        wait_or_fail(&mut sequence, "capture subject is missing", &mut exit);
        return;
    };
    let Some(playback) = playback else {
        wait_or_fail(
            &mut sequence,
            "capture subject has no AnimationPlayback",
            &mut exit,
        );
        return;
    };
    let Some(semantic_route) = semantic_route else {
        wait_or_fail(
            &mut sequence,
            "capture subject has no semantic route trace",
            &mut exit,
        );
        return;
    };
    if !sequence
        .active_scenario
        .as_ref()
        .is_some_and(|scenario| scenario.is(ScenarioId::FullRagdoll))
        && !playback.authored_pose_is_ready()
    {
        wait_or_fail(
            &mut sequence,
            "authored locomotion clip has not resolved",
            &mut exit,
        );
        return;
    }
    sequence.waiting = 0;
    sequence.settled += 1;
    let required = if sequence.index == 0 {
        sequence.settle_frames.max(60)
    } else {
        // A view change needs one complete render before requesting the next
        // asynchronous screenshot; otherwise two paths may receive the same
        // previously rendered camera image.
        sequence.settle_frames.max(2)
    };
    if sequence.settled < required {
        return;
    }

    let frame = sequence.plan[sequence.index].clone();
    let view = VIEWS[sequence.view_index];
    let file_name = view.screenshot_name(&frame);
    let path = sequence.output.join(&file_name);
    let evaluation_bones = collect_bones(
        subject,
        &bones,
        &terrain,
        (!terrain_ik.is_enabled())
            .then_some(subject_global.translation().y - CAPTURE_ROOT_GROUND_OFFSET_METRES),
    );
    let evaluation_leg_ik = leg_ik.map(LegIkState::diagnostics).unwrap_or_default();
    if sequence.view_index == 0 {
        sequence.repeated_evaluation_baseline = Some(RepeatedEvaluationSnapshot {
            scenario: frame.scenario.clone(),
            scenario_frame: frame.scenario_frame,
            bones: evaluation_bones.clone(),
            contact_sequence: skeleton.contact_sequence,
            landing_sequence: skeleton.landing_sequence,
            event_count: sequence.presentation_events.len(),
            leg_ik: evaluation_leg_ik,
        });
    } else if let Some(baseline) = &sequence.repeated_evaluation_baseline {
        let bone_mismatch = repeated_bone_mismatch(&baseline.bones, &evaluation_bones);
        let bones_match = bone_mismatch.is_none();
        let repeated_evaluation_matches = baseline.scenario == frame.scenario
            && baseline.scenario_frame == frame.scenario_frame
            && bones_match
            && baseline.contact_sequence == skeleton.contact_sequence
            && baseline.landing_sequence == skeleton.landing_sequence
            && baseline.event_count == sequence.presentation_events.len()
            && repeated_leg_ik_matches(baseline.leg_ik, evaluation_leg_ik);
        if !repeated_evaluation_matches
            && let Some((bone, position_delta, rotation_delta)) = &bone_mismatch
        {
            warn!(
                scenario = frame.scenario.label(),
                scenario_frame = frame.scenario_frame,
                bone,
                position_delta,
                rotation_delta,
                "repeated animation evaluation changed a captured bone"
            );
        }
        if !repeated_evaluation_matches && bone_mismatch.is_none() {
            warn!(
                scenario = frame.scenario.label(),
                scenario_frame = frame.scenario_frame,
                baseline_contact = baseline.contact_sequence.encoded(),
                contact = skeleton.contact_sequence.encoded(),
                baseline_landing = baseline.landing_sequence.encoded(),
                landing = skeleton.landing_sequence.encoded(),
                baseline_events = baseline.event_count,
                events = sequence.presentation_events.len(),
                "repeated animation evaluation changed non-bone state"
            );
        }
        sequence.repeated_evaluation_valid &= repeated_evaluation_matches;
    }
    if sequence.view_index == 0 {
        sequence.global_bone_frames.push(GlobalBoneFrame {
            scenario: frame.scenario.label().to_owned(),
            scenario_frame: frame.scenario_frame,
            time_seconds: frame.time_seconds.seconds(),
            action: skeleton.action_kind(),
            action_phase: skeleton.action_phase(),
            subject_translation: subject_global.translation().to_array(),
            subject_rotation_xyzw: subject_global.rotation().to_array(),
            bones: collect_global_bone_transforms(subject, &animation_bones),
        });
        let cadence_support = locomotion_support_weights(skeleton);
        let root_distance_metres = sequence.scenario_distance;
        let (desired_left_foot_target, desired_right_foot_target) = raised_footwork
            .map(|state| (state.left_solve_target, state.right_solve_target))
            .unwrap_or_default();
        let leg_ik = evaluation_leg_ik;
        let ik_support =
            if leg_ik.left_solve_target.is_some() || leg_ik.right_solve_target.is_some() {
                (leg_ik.left_support_weight, leg_ik.right_support_weight)
            } else {
                cadence_support
            };
        let (left_support_weight, right_support_weight) = raised_footwork
            .filter(|state| state.initialized())
            .map(|state| (state.left_support_weight, state.right_support_weight))
            .unwrap_or(ik_support);
        sequence.samples.push(FrameSample {
            scenario: frame.scenario.clone(),
            scenario_frame: frame.scenario_frame,
            time_seconds: frame.time_seconds,
            speed_metres_per_second: frame.speed,
            gait_phase: skeleton.gait_phase,
            locomotion_sample_tick: skeleton.locomotion_sample_tick,
            body_acceleration: (subject_global.rotation().inverse() * skeleton.world_acceleration)
                .to_array(),
            world_acceleration: skeleton.world_acceleration.to_array(),
            bouncy_active_springs: bouncy_bones_telemetry.active_springs,
            bouncy_maximum_deflection_radians: bouncy_bones_telemetry.maximum_deflection_radians,
            // Raised guard owns its visual contacts locally. Segment its
            // diagnostics by the sequence that actually changed the rendered
            // support foot, not by the replicated locomotion cadence.
            // RaisedFootworkState retains the presentation-owned contact
            // counter even while its actual step is deliberately
            // uninitialized across an authored/no-IK handoff. Falling back to
            // the replicated idle counter here invents a 1 -> 0 reset that
            // neither the rendered feet nor the local stepper performed.
            contact_sequence: raised_footwork.map_or(
                skeleton.contact_sequence,
                RaisedFootworkState::step_sequence,
            ),
            contact_foot: raised_footwork
                .and_then(RaisedFootworkState::contact_foot)
                .unwrap_or(skeleton.contact_foot),
            landing_sequence: skeleton.landing_sequence,
            landing_impact_speed: skeleton.landing_impact_speed,
            body_lean_pitch_degrees: body_response
                .map_or(BodyAngle::ZERO, |state| state.pitch_radians),
            body_lean_roll_degrees: body_response
                .map_or(BodyAngle::ZERO, |state| state.roll_radians),
            landing_compression_metres: height_state
                .map_or(HeightDistance::ZERO, |state| state.landing_compression),
            root_distance_metres,
            root_position_metres: subject_global.translation().to_array(),
            world_travel_direction: (controller_yaw(Quat::from_rotation_y(
                frame.camera_yaw.radians(),
            )) * Vec3::new(
                frame.local_direction.x,
                0.0,
                frame.local_direction.y,
            ))
            .normalize_or_zero()
            .to_array(),
            desired_body_forward_direction: if frame.weapon_guard == WeaponGuardState::Raised
                || matches!(frame.action, SkeletonAction::Attack | SkeletonAction::Block)
            {
                (controller_yaw(Quat::from_rotation_y(frame.camera_yaw.radians())) * Vec3::NEG_Z)
                    .to_array()
            } else {
                (controller_yaw(Quat::from_rotation_y(frame.camera_yaw.radians()))
                    * Vec3::new(frame.local_direction.x, 0.0, frame.local_direction.y))
                .normalize_or_zero()
                .to_array()
            },
            body_forward_direction: (subject_global.rotation() * Vec3::Z).to_array(),
            body_rotation_xyzw: subject_global.rotation().to_array(),
            weapon_guard: frame.weapon_guard,
            lead_foot: skeleton.lead_foot,
            action: skeleton.action_kind(),
            action_phase: skeleton.action_phase(),
            attack_animation: skeleton.attack_animation(),
            strike_family: skeleton.strike_family(),
            guard_action: frame.weapon_guard == WeaponGuardState::Raised
                || matches!(
                    frame.action,
                    SkeletonAction::Dodge | SkeletonAction::Attack | SkeletonAction::Block
                ),
            left_support_weight,
            right_support_weight,
            desired_left_foot_target: desired_left_foot_target.map(|value| value.to_array()),
            desired_right_foot_target: desired_right_foot_target.map(|value| value.to_array()),
            ik_left_authored_target: leg_ik.left_authored_target.map(|value| value.to_array()),
            ik_right_authored_target: leg_ik.right_authored_target.map(|value| value.to_array()),
            ik_left_planned_contact: leg_ik.left_planned_contact.map(|value| value.to_array()),
            ik_right_planned_contact: leg_ik.right_planned_contact.map(|value| value.to_array()),
            ik_settle_capture_point: leg_ik.settle_capture_point.map(|value| value.to_array()),
            ik_left_solve_target: leg_ik.left_solve_target.map(|value| value.to_array()),
            ik_right_solve_target: leg_ik.right_solve_target.map(|value| value.to_array()),
            ik_left_support_weight: leg_ik.left_support_weight,
            ik_right_support_weight: leg_ik.right_support_weight,
            ik_left_release_active: leg_ik.left_release_active,
            ik_right_release_active: leg_ik.right_release_active,
            ik_left_release_target: leg_ik.left_release_target.map(|value| value.to_array()),
            ik_right_release_target: leg_ik.right_release_target.map(|value| value.to_array()),
            ik_settle_progress: leg_ik.settle_progress,
            ik_left_knee_foot_yaw_offset_degrees: leg_ik.left_knee_foot_yaw_offset_degrees,
            ik_right_knee_foot_yaw_offset_degrees: leg_ik.right_knee_foot_yaw_offset_degrees,
            semantic_route_requested_path: semantic_route.requested_path,
            semantic_route_selected_path: semantic_route.path,
            semantic_route_runtime_evaluated: true,
            screenshots: VIEWS
                .into_iter()
                .map(|view| (view.slug().to_owned(), view.screenshot_name(&frame)))
                .collect(),
            bones: evaluation_bones,
        });
    }
    sequence.capture_in_flight = true;
    let frame_key = format!("{}:{}", frame.scenario.label(), frame.scenario_frame);
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>,
              mut sequence: ResMut<CaptureSequence>,
              mut exit: MessageWriter<AppExit>| {
            sequence
                .view_fingerprints
                .push(visual_fingerprint(&captured.image));
            save_to_disk(&path)(captured);
            sequence.capture_in_flight = false;
            sequence.view_index += 1;
            sequence.settled = 0;
            if sequence.view_index < VIEWS.len() {
                return;
            }
            if sequence
                .view_fingerprints
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                != VIEWS.len()
            {
                sequence.duplicate_view_frames.push(frame_key.clone());
            }
            sequence.view_fingerprints.clear();
            sequence.view_index = 0;
            sequence.index += 1;
            sequence.applied = false;
            if sequence.index == sequence.plan.len() {
                let completed = sequence.complete(playback_backend, pose_buffer_metrics);
                exit.write(write_completed_capture(completed));
            }
        },
    );
}

pub(super) fn visual_fingerprint(image: &Image) -> u64 {
    let Some(data) = image.data.as_deref() else {
        return 0;
    };
    let width = image.texture_descriptor.size.width as usize;
    let height = image.texture_descriptor.size.height as usize;
    let stride = width.saturating_mul(4);
    if stride == 0 || data.len() < stride.saturating_mul(height) {
        return 0;
    }
    // Ignore the top UI label and hash a regular sample of the rendered 3D
    // view. This catches accidentally identical camera outputs cheaply.
    let mut hash = 0xcbf29ce484222325_u64;
    for y in (96.min(height)..height).step_by(8) {
        for x in (0..width).step_by(8) {
            for channel in 0..3 {
                hash ^= data[y * stride + x * 4 + channel] as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
    }
    hash
}

pub(super) fn collect_bones(
    subject: Entity,
    bones: &Query<(&HumanoidBone, &GlobalTransform)>,
    terrain: &SceneTerrain,
    flat_ground_height: Option<f32>,
) -> CapturedBones {
    bones
        .iter()
        .filter(|(bone, _)| bone.owner == subject && report_bone_label(bone.role).is_some())
        .map(|(bone, transform)| {
            let rotation = transform.rotation();
            (
                bone.role,
                BoneSample {
                    position: transform.translation().to_array(),
                    rotation_xyzw: [rotation.x, rotation.y, rotation.z, rotation.w],
                    terrain_clearance_metres: flat_ground_height
                        .or_else(|| terrain.height_at(transform.translation().xz()))
                        .map(|height| transform.translation().y - height),
                },
            )
        })
        .collect()
}

pub(super) fn collect_global_bone_transforms(
    subject: Entity,
    bones: &Query<(
        &AnimationTargetId,
        &AuthoredBindTransform,
        Option<&Name>,
        &GlobalTransform,
    )>,
) -> Vec<GlobalBoneTransformSample> {
    let mut samples = bones
        .iter()
        .filter(|(_, bind, _, _)| bind.owner == subject)
        .map(|(target, _, name, transform)| {
            let (scale, rotation, translation) = transform.to_scale_rotation_translation();
            let sample = GlobalBoneTransformSample {
                name: name.map_or_else(|| "<unnamed>".to_owned(), |name| name.as_str().to_owned()),
                target_id: capture_animation_target_id(*target),
                translation: translation.to_array(),
                rotation_xyzw: rotation.to_array(),
                scale: scale.to_array(),
            };
            (*target, sample)
        })
        .collect::<Vec<_>>();
    samples.sort_by(|(left_target, left), (right_target, right)| {
        (&left.name, left_target).cmp(&(&right.name, right_target))
    });
    samples.into_iter().map(|(_, sample)| sample).collect()
}

pub(super) fn jitter_frames(frames: &[FrameSample]) -> Vec<JitterFrame<BoneRole>> {
    let mut previous: Option<&FrameSample> = None;
    let mut analysis_segment = 0_u64;
    frames
        .iter()
        // A ragdoll is intentionally non-smooth at the animation-to-physics
        // handoff and does not obey authored locomotion jerk thresholds.
        .filter(|frame| !frame.scenario.is(ScenarioId::FullRagdoll))
        .map(|frame| {
            if let Some(previous_frame) = previous {
                let action_transition = previous_frame.action != frame.action;
                let landing = previous_frame.landing_sequence != frame.landing_sequence;
                let foot_contact = previous_frame.contact_sequence != frame.contact_sequence;
                let guard_stop_handoff = is_guard_stop_transition(&frame.scenario)
                    && previous_frame.speed_metres_per_second > 0.05
                    && frame.speed_metres_per_second <= 0.05;
                let quickstep_takeoff = is_quickstep_scenario(&frame.scenario)
                    && previous_frame.left_support_weight >= FULL_PLANT_SUPPORT_WEIGHT
                    && previous_frame.right_support_weight >= FULL_PLANT_SUPPORT_WEIGHT
                    && frame.left_support_weight <= 0.01
                    && frame.right_support_weight <= 0.01;
                if previous_frame.scenario != frame.scenario {
                    analysis_segment = 0;
                } else if action_transition
                    || landing
                    || foot_contact
                    || guard_stop_handoff
                    || quickstep_takeoff
                {
                    analysis_segment = analysis_segment.wrapping_add(1);
                }
            }
            previous = Some(frame);
            JitterFrame {
                scenario: frame.scenario.clone(),
                analysis_segment,
                scenario_frame: frame.scenario_frame,
                time_seconds: frame.time_seconds.seconds(),
                bones: frame
                    .bones
                    .iter()
                    .map(|(name, bone)| {
                        let position = if *name == BoneRole::Pelvis {
                            // The capture root is authoritative locomotion, not
                            // a skeletal joint. Exclude its world translation
                            // from limb jitter while retaining pelvis rotation.
                            Vec3::ZERO
                        } else {
                            Vec3::from_array(bone.position)
                        };
                        let rotation = Quat::from_array(bone.rotation_xyzw);
                        let (position, rotation) = parent_bone(name)
                            .and_then(|parent| frame.bones.get(parent))
                            .map_or((position, rotation), |parent| {
                                let parent_position = Vec3::from_array(parent.position);
                                let parent_rotation = Quat::from_array(parent.rotation_xyzw);
                                (
                                    parent_rotation.inverse() * (position - parent_position),
                                    parent_rotation.inverse() * rotation,
                                )
                            });
                        (
                            name.clone(),
                            JitterBone {
                                position: position.to_array(),
                                rotation_xyzw: rotation.to_array(),
                            },
                        )
                    })
                    .collect(),
            }
        })
        .collect()
}

pub(super) fn wait_or_fail(
    sequence: &mut CaptureSequence,
    reason: &str,
    exit: &mut MessageWriter<AppExit>,
) {
    sequence.waiting += 1;
    if sequence.waiting < CAPTURE_LOAD_FRAME_LIMIT {
        return;
    }
    let message = format!(
        "animation viewer timed out after {} rendered frames: {reason}\n",
        sequence.waiting
    );
    fail_capture(sequence, &message, exit);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_evaluation_rejects_missing_or_extra_tracked_bones() {
        let sample = BoneSample {
            position: Vec3::ZERO.to_array(),
            rotation_xyzw: Quat::IDENTITY.to_array(),
            terrain_clearance_metres: Some(0.0),
        };
        let expected = CapturedBones::from_iter([(BoneRole::FootLeft, sample)]);
        let missing = CapturedBones::default();
        assert!(repeated_bone_mismatch(&expected, &missing).is_some());
        assert!(repeated_bone_mismatch(&missing, &expected).is_some());
        assert!(repeated_bone_mismatch(&expected, &expected).is_none());
    }

    #[test]
    fn repeated_evaluation_diagnostics_detect_hidden_ik_state_mutation() {
        let baseline = LegIkDiagnostics::default();
        let changed = LegIkDiagnostics {
            left_support_weight: 1.0,
            left_release_active: true,
            left_planned_contact: Some(Vec3::NEG_Z),
            settle_progress: Some(0.5),
            ..default()
        };
        assert_ne!(baseline, changed);
    }

    #[test]
    fn flat_grid_scenarios_are_opt_in_complete_cycles_with_explicit_ik_ownership() {
        for (scenario, speed) in [("flat-grid-walk-2.0", 2.0), ("flat-grid-run-5.5", 5.5)] {
            let sequence = CaptureSequence::new(PathBuf::new(), 1, Some(scenario));
            assert!(sequence.uses_flat_grid());
            assert!(sequence.plan.len() > 64);
            assert!(sequence.plan.iter().all(|frame| {
                frame.scenario.label() == scenario
                    && frame.speed == speed
                    && terrain_ik_mode_for_frame(frame) == TerrainIkMode::Enabled
            }));
        }

        let sprint = CaptureSequence::new(PathBuf::new(), 1, Some("flat-grid-sprint-no-ik"));
        assert!(sprint.uses_flat_grid());
        assert!(sprint.plan.len() > 64);
        assert!(sprint.plan.iter().all(|frame| {
            frame.scenario.is(ScenarioId::FlatGridSprintNoIk)
                && frame.speed == canonical_john_sprint_speed()
                && terrain_ik_mode_for_frame(frame) == TerrainIkMode::Disabled
        }));
        assert!((canonical_john_sprint_speed() - 8.957_856).abs() < 0.000_01);

        let walk = CaptureSequence::new(PathBuf::new(), 1, Some("flat-grid-walk-no-ik"));
        assert!(walk.uses_flat_grid());
        assert!(walk.plan.len() > 64);
        assert!(walk.plan.iter().all(|frame| {
            frame.scenario.is(ScenarioId::FlatGridWalkNoIk)
                && frame.speed == 2.0
                && terrain_ik_mode_for_frame(frame) == TerrainIkMode::Disabled
        }));

        let ordinary = CaptureSequence::new(PathBuf::new(), 1, Some("steady-walk-2.0"));
        assert!(!ordinary.uses_flat_grid());

        let stop = CaptureSequence::new(PathBuf::new(), 1, Some("flat-grid-walk-stop"));
        assert!(stop.uses_flat_grid());
        assert!(stop.plan[..48].iter().all(|frame| frame.speed == 2.0));
        assert!(stop.plan[56..].iter().all(|frame| frame.speed == 0.0));
        assert!(
            stop.plan[48..=56]
                .windows(2)
                .all(|pair| pair[1].speed <= pair[0].speed)
        );
    }

    #[test]
    fn completed_capture_owns_the_finished_ecs_state() {
        let output = PathBuf::from("owned-capture");
        let mut sequence = CaptureSequence::new(output.clone(), 1, Some("flat-grid-walk-2.0"));
        let plan_len = sequence.plan.len();

        let completed = sequence.complete("pose_buffer", PoseBufferMetrics::default());

        assert_eq!(completed.output, output);
        assert_eq!(completed.plan.len(), plan_len);
        assert_eq!(completed.playback_backend, "pose_buffer");
        assert!(sequence.output.as_os_str().is_empty());
        assert!(sequence.plan.is_empty());
    }
}
