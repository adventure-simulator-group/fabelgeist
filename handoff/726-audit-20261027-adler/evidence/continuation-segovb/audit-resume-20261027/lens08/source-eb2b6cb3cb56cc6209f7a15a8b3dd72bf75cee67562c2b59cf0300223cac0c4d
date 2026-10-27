//! ECS fixture driving, sampling, and three-view capture.

use super::*;
pub(super) use admission::reject_capture;
use admission::{CaptureAdmission, RejectedCaptureAdmission, fail_capture};
use adventuresim_tactical_core::animation::STATIONARY_ANIMATION_SPEED_LIMIT;
use fabelgeist_geometry::SignedLength;
pub(super) use capture_schedule::CapturePlugin;
use frame_clock::{capture_frame_elapsed, next_capture_simulation_tick};
use repeated_evaluation::RepeatedEvaluationSnapshot;

mod admission;
mod capture_schedule;
pub(super) mod frame_clock;
mod global_bones;
mod overlay;
mod readiness;
mod repeated_evaluation;
mod root_motion;
mod sample;
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
    admission: CaptureAdmission,
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
    scenario_distance: SignedLength,
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
    pub(super) fn new(
        output: PathBuf,
        settle_frames: u32,
        scenario: Option<ScenarioMetadata<'_>>,
    ) -> AnimationValueResult<Self> {
        let selected = scenario.map(ScenarioMetadata::into_owned);
        let plan = match selected {
            Some(scenario) if scenario.identity() == Some(ScenarioId::FlatGridWalk20) => {
                steady_scenario(scenario, authored_drive_speed(2.0), 3.0)?
            }
            Some(scenario) if scenario.identity() == Some(ScenarioId::FlatGridRun55) => {
                steady_scenario(scenario, authored_drive_speed(5.5), 3.0)?
            }
            Some(scenario) if scenario.identity() == Some(ScenarioId::FlatGridWalkNoIk) => {
                steady_scenario(scenario, authored_drive_speed(2.0), 3.0)?
            }
            Some(scenario) if scenario.identity() == Some(ScenarioId::FlatGridSprintNoIk) => {
                steady_scenario(scenario, canonical_john_sprint_speed(), 3.0)?
            }
            Some(scenario) if scenario.identity() == Some(ScenarioId::FlatGridWalkStop) => {
                flat_grid_walk_stop_scenario()
            }
            Some(scenario) if scenario.identity() == Some(ScenarioId::FullRagdoll) => {
                full_ragdoll_scenario()
            }
            selected => capture_plan()?
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
        Ok(Self {
            output,
            settle_frames: settle_frames.max(1),
            plan,
            index: 0,
            view_index: 0,
            applied: false,
            admission: CaptureAdmission::Open,
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
            scenario_distance: SignedLength::ZERO,
        })
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

/// Screenshot completion advances capture views asynchronously. Reassert the
/// planned look on every render of a logical sample so all three views enter
/// procedural PostUpdate with identical presentation input.
pub(super) fn freeze_capture_look(
    sequence: Res<CaptureSequence>,
    mut subjects: Query<&mut CharacterLook, With<CaptureSubject>>,
) {
    if !sequence.admission.is_open() {
        return;
    }
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
    if !sequence.can_drive_frame() {
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
            sequence.scenario_distance = SignedLength::ZERO;
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
            let Some(initial) = sequence.initial_root(&terrain, *transform) else {
                return;
            };
            *transform = initial;
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
                    .is_some_and(|motion| animation_runtime.motion_is_processed(motion));
            skeleton.set_downed_motion(
                if !frame.scenario.is(ScenarioId::DownedProneLookAt)
                    && (preload_locomotion
                        || (frame.scenario_frame >= 4
                            && frame.speed <= SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)))
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
        let attack_start_frame = frame.scenario.has(ScenarioFeature::AttackLive).then_some(8);
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
            let step = if frame.scenario_frame == 0 {
                Ok(DownedFacingStep::ZERO)
            } else {
                DownedFacingStep::from_half_turns(1.0 / 84.0)
            };
            if let Err(error) = step.and_then(|step| {
                skeleton.advance_downed_facing(target, DownedFacingAim::Held, step)
            }) {
                sequence.admission =
                    CaptureAdmission::Rejected(RejectedCaptureAdmission::Locomotion(error));
                return;
            }
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
            frame.local_direction.x * frame.speed.metres_per_second(),
            vertical_velocity,
            frame.local_direction.y * frame.speed.metres_per_second(),
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
        let elapsed = capture_frame_elapsed(frame.scenario_frame);
        procedural_clock.set_fixed_step(sequence.simulation_tick, elapsed);
        let motion = subject_motion::CaptureMotion {
            orientation,
            world_velocity,
            contact: GroundContact::from(grounded),
            elapsed,
        };
        let Some(position) = sequence.fixture_root_position(
            &terrain,
            transform.translation,
            &frame,
            &motion,
            *terrain_ik,
        ) else {
            return;
        };
        transform.translation = position.metres();
        subject_motion::begin_capture_action(
            &mut skeleton,
            &frame,
            sequence.simulation_tick,
            attack_start_frame,
        );
        if let Err(error) = subject_motion::advance_capture_facing(
            &skeleton,
            &mut transform,
            orientation,
            world_velocity,
            frame.action,
            elapsed,
        ) {
            sequence.admission =
                CaptureAdmission::Rejected(RejectedCaptureAdmission::Locomotion(error));
            return;
        }
        if frame.scenario.is(ScenarioId::FullRagdoll) {
            let Some(projected) = sequence.full_ragdoll_root(&terrain, *transform, &frame) else {
                return;
            };
            *transform = projected;
        }
        if !sequence.advance_distance(frame.speed, elapsed) {
            return;
        }
        if let Err(error) = subject_motion::project_capture_motion(
            &mut skeleton,
            &mut transform,
            &frame,
            motion,
            &sequence,
        ) {
            sequence.admission =
                CaptureAdmission::Rejected(RejectedCaptureAdmission::Locomotion(error));
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
            frame.speed.metres_per_second(),
            gait_phase.cycles(),
            VIEWS[sequence.view_index].slug(),
            frame.scenario_frame,
        );
    }
    sequence.applied = true;
    sequence.settled = 0;
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
    if !sequence.admission.is_open() || sequence.index >= sequence.plan.len() {
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
    if !sequence.admission.is_open() || !sequence.applied || sequence.capture_in_flight {
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
    let Some(ground) = sequence.capture_ground(*terrain_ik, subject_global.translation()) else {
        return;
    };
    let evaluation_bones = match ground.collect_bones(subject, &bones, &terrain) {
        Ok(bones) => bones,
        Err(error) => {
            sequence.admission =
                CaptureAdmission::Rejected(RejectedCaptureAdmission::Geometry(error));
            return;
        }
    };
    let evaluation_leg_ik = leg_ik.map(LegIkState::diagnostics);
    sequence.record_repeated_evaluation(&frame, &evaluation_bones, skeleton, evaluation_leg_ik);
    if sequence.view_index == 0 {
        sequence.global_bone_frames.push(GlobalBoneFrame::capture(
            &frame,
            skeleton,
            subject_global,
            subject,
            &animation_bones,
        ));
        let Some(cadence_support) =
            sequence.admit_locomotion_support(skeleton, semantic_route.inputs.speed)
        else {
            return;
        };
        let leg_ik = evaluation_leg_ik;
        let Some(ik_support) =
            sequence.admit_ik_support(leg_ik, cadence_support, skeleton, *terrain_ik)
        else {
            return;
        };
        let (left_support_weight, right_support_weight) = raised_footwork
            .filter(|state| state.initialized())
            .map(|state| (state.left_support_weight, state.right_support_weight))
            .unwrap_or(ik_support);
        let sample = match FrameSample::capture(sample::CaptureSample {
            frame: &frame,
            skeleton,
            subject_global,
            bouncy_bones_telemetry: &bouncy_bones_telemetry,
            raised_footwork,
            body_response,
            height_state,
            leg_ik,
            semantic_route,
            root_distance_metres: sequence.scenario_distance,
            left_support_weight,
            right_support_weight,
            evaluation_bones,
        }) {
            Ok(sample) => sample,
            Err(error) => {
                sequence.admission =
                    CaptureAdmission::Rejected(error);
                return;
            }
        };
        sequence.samples.push(sample);
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
                    && previous_frame.speed_metres_per_second
                        > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
                    && frame.speed_metres_per_second
                        <= SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT);
                let quickstep_takeoff = is_quickstep_scenario(&frame.scenario)
                    && previous_frame.left_support_weight.fraction() >= FULL_PLANT_SUPPORT_WEIGHT
                    && previous_frame.right_support_weight.fraction() >= FULL_PLANT_SUPPORT_WEIGHT
                    && frame.left_support_weight.fraction() <= 0.01
                    && frame.right_support_weight.fraction() <= 0.01;
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
                time_seconds: frame.time_seconds,
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
                            *name,
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
    fn flat_grid_scenarios_are_opt_in_complete_cycles_with_explicit_ik_ownership() {
        for (scenario, speed) in [
            ("flat-grid-walk-2.0", authored_drive_speed(2.0)),
            ("flat-grid-run-5.5", authored_drive_speed(5.5)),
        ] {
            let sequence = CaptureSequence::new(
                PathBuf::new(),
                1,
                Some(ScenarioMetadata::from_label(scenario)),
            )
            .unwrap();
            assert!(sequence.uses_flat_grid());
            assert!(sequence.plan.len() > 64);
            assert!(sequence.plan.iter().all(|frame| {
                frame.scenario.label() == scenario
                    && frame.speed == speed
                    && terrain_ik_mode_for_frame(frame) == TerrainIkMode::Enabled
            }));
        }

        let sprint = CaptureSequence::new(
            PathBuf::new(),
            1,
            Some(ScenarioMetadata::from_label("flat-grid-sprint-no-ik")),
        )
        .unwrap();
        assert!(sprint.uses_flat_grid());
        assert!(sprint.plan.len() > 64);
        assert!(sprint.plan.iter().all(|frame| {
            frame.scenario.is(ScenarioId::FlatGridSprintNoIk)
                && frame.speed == canonical_john_sprint_speed()
                && terrain_ik_mode_for_frame(frame) == TerrainIkMode::Disabled
        }));
        assert!((canonical_john_sprint_speed().metres_per_second() - 8.957_856).abs() < 0.000_01);

        let walk = CaptureSequence::new(
            PathBuf::new(),
            1,
            Some(ScenarioMetadata::from_label("flat-grid-walk-no-ik")),
        )
        .unwrap();
        assert!(walk.uses_flat_grid());
        assert!(walk.plan.len() > 64);
        assert!(walk.plan.iter().all(|frame| {
            frame.scenario.is(ScenarioId::FlatGridWalkNoIk)
                && frame.speed == authored_drive_speed(2.0)
                && terrain_ik_mode_for_frame(frame) == TerrainIkMode::Disabled
        }));

        let ordinary = CaptureSequence::new(
            PathBuf::new(),
            1,
            Some(ScenarioMetadata::from_label("steady-walk-2.0")),
        )
        .unwrap();
        assert!(!ordinary.uses_flat_grid());

        let stop = CaptureSequence::new(
            PathBuf::new(),
            1,
            Some(ScenarioMetadata::from_label("flat-grid-walk-stop")),
        )
        .unwrap();
        assert!(stop.uses_flat_grid());
        assert!(
            stop.plan[..48]
                .iter()
                .all(|frame| frame.speed == authored_drive_speed(2.0))
        );
        assert!(
            stop.plan[56..]
                .iter()
                .all(|frame| frame.speed == SignedSpeed::ZERO)
        );
        assert!(
            stop.plan[48..=56]
                .windows(2)
                .all(|pair| pair[1].speed <= pair[0].speed)
        );
    }

    #[test]
    fn completed_capture_owns_the_finished_ecs_state() {
        let output = PathBuf::from("owned-capture");
        let mut sequence = CaptureSequence::new(
            output.clone(),
            1,
            Some(ScenarioMetadata::from_label("flat-grid-walk-2.0")),
        )
        .unwrap();
        let plan_len = sequence.plan.len();

        let completed = sequence.complete("pose_buffer", PoseBufferMetrics::default());

        assert_eq!(completed.output, output);
        assert_eq!(completed.plan.len(), plan_len);
        assert_eq!(completed.playback_backend, "pose_buffer");
        assert!(sequence.output.as_os_str().is_empty());
        assert!(sequence.plan.is_empty());
    }
}
