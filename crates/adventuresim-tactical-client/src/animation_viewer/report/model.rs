//! Serialized capture, validation, metric, and trace contracts.

use super::*;
use crate::animation_viewer::scenarios::PlannedFrame;
use fabelgeist_geometry::SignedLength;
use adventuresim_tactical_core::animation::{
    AngularMagnitudeDegrees, AnimationQuantity, AnimationValueError, AnimationValueResult,
    NonnegativeLength, SignedAngleDegrees, UnitInterval,
};
use adventuresim_tactical_core::animation::{
    AnimationSampleTick, ContactSequence, LandingSequence,
};

pub(in crate::animation_viewer) const VIEWS: [CaptureView; 3] =
    [CaptureView::Gameplay, CaptureView::Side, CaptureView::Front];

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::animation_viewer) enum CaptureView {
    Gameplay,
    Side,
    Front,
}

impl CaptureView {
    pub(in crate::animation_viewer) fn slug(self) -> &'static str {
        match self {
            Self::Gameplay => "gameplay",
            Self::Side => "side",
            Self::Front => "front",
        }
    }

    pub(in crate::animation_viewer) fn screenshot_name(self, frame: &PlannedFrame) -> String {
        format!(
            "{}-{:04}-{}.png",
            frame.scenario.label(),
            frame.scenario_frame,
            self.slug()
        )
    }
}

#[derive(Debug, Serialize)]
pub(in crate::animation_viewer) struct AnimationCaptureManifest {
    pub(in crate::animation_viewer) sample_hz: adventuresim_tactical_core::animation::AnimationSampleRate,
    pub(in crate::animation_viewer) playback_backend: &'static str,
    pub(in crate::animation_viewer) global_bone_trace: &'static str,
    pub(in crate::animation_viewer) pose_buffer: PoseBufferMetrics,
    pub(in crate::animation_viewer) pipeline: &'static str,
    pub(in crate::animation_viewer) views: [CaptureView; 3],
    pub(in crate::animation_viewer) validation: AnimationCaptureValidation,
    pub(in crate::animation_viewer) quality_score: QualityScore,
    pub(in crate::animation_viewer) scenarios: Vec<ScenarioMetrics>,
    pub(in crate::animation_viewer) frames: Vec<FrameSample>,
    pub(in crate::animation_viewer) presentation_events: Vec<PresentationEventSample>,
    pub(in crate::animation_viewer) semantic_route_path_counts: BTreeMap<String, u64>,
}

#[derive(Debug, Serialize)]
pub(in crate::animation_viewer) struct QualityScore {
    pub(in crate::animation_viewer) weighted_defect_score: u8,
    pub(in crate::animation_viewer) maximum_weighted_defect_score: u8,
    pub(in crate::animation_viewer) quality_percent: Observation<QualityPercent>,
    pub(in crate::animation_viewer) acceptance_passed: bool,
    pub(in crate::animation_viewer) categories: QualityCategories,
}

#[derive(Debug, Serialize)]
pub(in crate::animation_viewer) struct QualityCategories {
    pub(in crate::animation_viewer) catastrophic_foot_displacement_failed: bool,
    pub(in crate::animation_viewer) guard_step_liveness_failed: bool,
    pub(in crate::animation_viewer) anatomical_invalid_joints_failed: bool,
    pub(in crate::animation_viewer) contact_foot_airborne_failed: bool,
    pub(in crate::animation_viewer) both_feet_behind_hips_failed: bool,
    pub(in crate::animation_viewer) foot_dragging_failed: bool,
    pub(in crate::animation_viewer) jitter_and_jerk_failed: bool,
}

#[derive(Debug)]
pub(in crate::animation_viewer) struct PresentationEventSample {
    pub(in crate::animation_viewer) scenario: ScenarioMetadata<'static>,
    pub(in crate::animation_viewer) scenario_frame: usize,
    pub(in crate::animation_viewer) owner: String,
    pub(in crate::animation_viewer) sample_tick: AnimationSampleTick,
    pub(in crate::animation_viewer) payload: LocomotionPresentationPayload,
}

#[derive(Debug, Serialize)]
pub(in crate::animation_viewer) struct AnimationCaptureValidation {
    pub(in crate::animation_viewer) finite_transforms: bool,
    pub(in crate::animation_viewer) all_scenarios_complete: bool,
    pub(in crate::animation_viewer) all_artifacts_written: bool,
    pub(in crate::animation_viewer) continuity_within_review_bounds: bool,
    pub(in crate::animation_viewer) biomechanics_within_review_bounds: bool,
    pub(in crate::animation_viewer) no_ground_penetration: bool,
    pub(in crate::animation_viewer) raised_guard_fixed_support: bool,
    pub(in crate::animation_viewer) raised_guard_step_liveness_valid: bool,
    pub(in crate::animation_viewer) flat_controller_height_stable: bool,
    pub(in crate::animation_viewer) phase_owned_height_valid: bool,
    pub(in crate::animation_viewer) run_flight_valid: bool,
    pub(in crate::animation_viewer) body_response_valid: bool,
    pub(in crate::animation_viewer) bouncy_bones_valid: bool,
    pub(in crate::animation_viewer) straight_run_torso_sway_valid: bool,
    pub(in crate::animation_viewer) speed_ramp_phase_continuity_valid: bool,
    pub(in crate::animation_viewer) contact_sequences_valid: bool,
    pub(in crate::animation_viewer) cadence_contact_valid: bool,
    pub(in crate::animation_viewer) event_stream_valid: bool,
    pub(in crate::animation_viewer) landing_response_valid: bool,
    pub(in crate::animation_viewer) landing_foot_preservation_valid: bool,
    pub(in crate::animation_viewer) ordinary_swing_tracking_valid: bool,
    pub(in crate::animation_viewer) reported_support_contacts_valid: bool,
    pub(in crate::animation_viewer) run_contact_acquisition_valid: bool,
    pub(in crate::animation_viewer) stop_settle_capture_valid: bool,
    pub(in crate::animation_viewer) final_support_balance_valid: bool,
    pub(in crate::animation_viewer) hard_stop_maximum_pelvis_step_metres: Observation<HeightDistance>,
    pub(in crate::animation_viewer) hard_stop_height_continuity_valid: bool,
    pub(in crate::animation_viewer) repeated_evaluation_valid: bool,
    pub(in crate::animation_viewer) semantic_route_paths_exercised: bool,
    pub(in crate::animation_viewer) jitter_validation: JitterValidationSummary<BoneRole>,
    pub(in crate::animation_viewer) views_are_distinct: bool,
    pub(in crate::animation_viewer) duplicate_view_frames: Vec<String>,
    pub(in crate::animation_viewer) note: &'static str,
}

#[derive(Debug, Serialize)]
pub(in crate::animation_viewer) struct ScenarioMetrics {
    pub(in crate::animation_viewer) scenario: ScenarioMetadata<'static>,
    pub(in crate::animation_viewer) frame_count: usize,
    pub(in crate::animation_viewer) expected: ExpectedScenarioBounds,
    #[serde(flatten)]
    pub(in crate::animation_viewer) height: ScenarioHeightMetrics,
    pub(in crate::animation_viewer) continuity: Observation<ScenarioContinuityMetrics>,
    pub(in crate::animation_viewer) loop_seam: Observation<LoopSeamMetrics>,
    #[serde(flatten)]
    pub(in crate::animation_viewer) clearance: ScenarioClearanceMetrics,
    #[serde(flatten)]
    pub(in crate::animation_viewer) pose: ScenarioPoseMetrics,
    #[serde(flatten)]
    pub(in crate::animation_viewer) facing: ScenarioFacingMetrics,
    pub(in crate::animation_viewer) maximum_planted_foot_drift_metres: Observation<NonnegativeLength>,
    pub(in crate::animation_viewer) guard_step_liveness: Observation<GuardStepLivenessMetrics>,
}

#[derive(Debug, Serialize)]
pub(in crate::animation_viewer) struct ScenarioHeightMetrics {
    pub(in crate::animation_viewer) pelvis_vertical_range_metres: Observation<HeightDistance>,
    pub(in crate::animation_viewer) maximum_pelvis_vertical_step_metres: Observation<HeightDistance>,
    pub(in crate::animation_viewer) controller_vertical_range_metres: Observation<HeightDistance>,
    pub(in crate::animation_viewer) phase_height_range_metres: Observation<HeightDistance>,
    pub(in crate::animation_viewer) contact_to_passing_height_gain_metres: Observation<SignedLength>,
    pub(in crate::animation_viewer) visual_height_peaks: Observation<VisualHeightPeaks>,
    pub(in crate::animation_viewer) maximum_no_support_seconds: Observation<AnimationElapsed>,
    pub(in crate::animation_viewer) head_vertical_range_metres: Observation<HeightDistance>,
}

#[derive(Debug, Serialize)]
pub(in crate::animation_viewer) struct ScenarioClearanceMetrics {
    pub(in crate::animation_viewer) minimum_flight_sole_clearance_metres: Observation<SignedLength>,
    pub(in crate::animation_viewer) contact_sole_clearance_metres: Observation<SignedLengthRange>,
    pub(in crate::animation_viewer) minimum_flight_toe_clearance_metres: Observation<SignedLength>,
    pub(in crate::animation_viewer) minimum_contact_toe_clearance_metres: Observation<SignedLength>,
    pub(in crate::animation_viewer) foot_terrain_relief_metres: Observation<HeightDistance>,
    pub(in crate::animation_viewer) minimum_foot_clearance_metres: Observation<SignedLength>,
}

#[derive(Debug, Serialize)]
pub(in crate::animation_viewer) struct ScenarioPoseMetrics {
    pub(in crate::animation_viewer) minimum_knee_forward_bend_metres: Observation<SignedLength>,
    pub(in crate::animation_viewer) minimum_signed_foot_track_metres: Observation<SignedLength>,
    pub(in crate::animation_viewer) minimum_inter_foot_separation_metres: Observation<NonnegativeLength>,
    pub(in crate::animation_viewer) minimum_knee_flexion_degrees: Observation<AngularMagnitudeDegrees>,
    pub(in crate::animation_viewer) minimum_knee_hemisphere_dot: Observation<KneeHemisphereDot>,
    pub(in crate::animation_viewer) maximum_knee_foot_yaw_offset_degrees: Observation<AngularMagnitudeDegrees>,
}

#[derive(Debug, Serialize)]
pub(in crate::animation_viewer) struct ScenarioFacingMetrics {
    pub(in crate::animation_viewer) maximum_facing_motion_error_degrees: Observation<AngularMagnitudeDegrees>,
    pub(in crate::animation_viewer) maximum_facing_tracking_excess_degrees: Observation<AngularMagnitudeDegrees>,
    pub(in crate::animation_viewer) maximum_guard_facing_error_degrees: Observation<AngularMagnitudeDegrees>,
    pub(in crate::animation_viewer) final_facing_motion_error_degrees: Observation<AngularMagnitudeDegrees>,
    pub(in crate::animation_viewer) maximum_dive_axis_motion_error_degrees: Observation<AngularMagnitudeDegrees>,
}

#[derive(Debug, Serialize)]
pub(in crate::animation_viewer) struct ScenarioContinuityMetrics {
    pub(in crate::animation_viewer::report) maximum_root_relative_step_metres: NonnegativeLength,
    pub(in crate::animation_viewer::report) maximum_leg_root_relative_step_metres: NonnegativeLength,
    pub(in crate::animation_viewer::report) maximum_foot_root_relative_step_metres: NonnegativeLength,
    pub(in crate::animation_viewer::report) maximum_knee_root_relative_step_metres: NonnegativeLength,
    pub(in crate::animation_viewer::report) worst_displacement: Option<ContinuityLocation<NonnegativeLength>>,
    pub(in crate::animation_viewer::report) maximum_bone_rotation_step_degrees: AngularMagnitudeDegrees,
    pub(in crate::animation_viewer::report) maximum_foot_rotation_step_degrees: AngularMagnitudeDegrees,
    pub(in crate::animation_viewer::report) worst_rotation: Option<ContinuityLocation<AngularMagnitudeDegrees>>,
    pub(in crate::animation_viewer::report) maximum_supported_foot_slip_metres_per_frame: Observation<NonnegativeLength>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(in crate::animation_viewer) struct LoopSeamMetrics {
    pub(in crate::animation_viewer::report) position_metres: NonnegativeLength,
    pub(in crate::animation_viewer::report) rotation_degrees: AngularMagnitudeDegrees,
}

#[derive(Debug, Serialize)]
pub(in crate::animation_viewer) struct GuardStepLivenessMetrics {
    pub(in crate::animation_viewer::report) completed_half_steps: usize,
    pub(in crate::animation_viewer::report) visible_half_steps: usize,
    pub(in crate::animation_viewer::report) minimum_swing_travel_metres: NonnegativeLength,
    pub(in crate::animation_viewer::report) minimum_swing_clearance_gain_metres: SignedLength,
}

#[derive(Debug, Clone, Serialize)]
pub(in crate::animation_viewer) struct ContinuityLocation<T> {
    #[serde(serialize_with = "serialize_report_bone")]
    pub(in crate::animation_viewer) bone: BoneRole,
    pub(in crate::animation_viewer) from_frame: usize,
    pub(in crate::animation_viewer) to_frame: usize,
    pub(in crate::animation_viewer) value: T,
}

#[derive(Debug, Clone, Serialize)]
pub(in crate::animation_viewer) struct FrameSample {
    pub(in crate::animation_viewer) scenario: ScenarioMetadata<'static>,
    pub(in crate::animation_viewer) scenario_frame: usize,
    pub(in crate::animation_viewer) time_seconds: AnimationElapsed,
    pub(in crate::animation_viewer) speed_metres_per_second: SignedSpeed,
    pub(in crate::animation_viewer) gait_phase: CyclePhase,
    pub(in crate::animation_viewer) locomotion_sample_tick: AnimationSampleTick,
    pub(in crate::animation_viewer) body_acceleration: [f32; 3],
    pub(in crate::animation_viewer) world_acceleration: [f32; 3],
    pub(in crate::animation_viewer) bouncy_active_springs: u32,
    pub(in crate::animation_viewer) bouncy_maximum_deflection_radians: adventuresim_building_generator::spatial_geometry::Radians,
    pub(in crate::animation_viewer) contact_sequence: ContactSequence,
    pub(in crate::animation_viewer) contact_foot: LeadFoot,
    pub(in crate::animation_viewer) landing_sequence: LandingSequence,
    pub(in crate::animation_viewer) landing_impact_speed: LandingImpactSpeed,
    pub(in crate::animation_viewer) body_lean_pitch_degrees: Option<SignedAngleDegrees>,
    pub(in crate::animation_viewer) body_lean_roll_degrees: Option<SignedAngleDegrees>,
    pub(in crate::animation_viewer) landing_compression_metres: Option<HeightDistance>,
    pub(in crate::animation_viewer) root_distance_metres: SignedLength,
    pub(in crate::animation_viewer) root_position_metres: [f32; 3],
    pub(in crate::animation_viewer) world_travel_direction: [f32; 3],
    pub(in crate::animation_viewer) desired_body_forward_direction: [f32; 3],
    pub(in crate::animation_viewer) body_forward_direction: [f32; 3],
    pub(in crate::animation_viewer) body_rotation_xyzw: [f32; 4],
    pub(in crate::animation_viewer) weapon_guard: WeaponGuardState,
    pub(in crate::animation_viewer) lead_foot: LeadFoot,
    pub(in crate::animation_viewer) action: SkeletonAction,
    pub(in crate::animation_viewer) action_phase:
        adventuresim_tactical_core::animation::UnitInterval,
    pub(in crate::animation_viewer) attack_animation: Option<AttackAnimation>,
    pub(in crate::animation_viewer) strike_family: StrikeFamily,
    pub(in crate::animation_viewer) guard_action: bool,
    pub(in crate::animation_viewer) left_support_weight: UnitInterval,
    pub(in crate::animation_viewer) right_support_weight: UnitInterval,
    pub(in crate::animation_viewer) desired_left_foot_target: Option<[f32; 3]>,
    pub(in crate::animation_viewer) desired_right_foot_target: Option<[f32; 3]>,
    pub(in crate::animation_viewer) ik_left_authored_target: Option<[f32; 3]>,
    pub(in crate::animation_viewer) ik_right_authored_target: Option<[f32; 3]>,
    pub(in crate::animation_viewer) ik_left_planned_contact: Option<[f32; 3]>,
    pub(in crate::animation_viewer) ik_right_planned_contact: Option<[f32; 3]>,
    pub(in crate::animation_viewer) ik_settle_capture_point: Option<[f32; 3]>,
    pub(in crate::animation_viewer) ik_left_solve_target: Option<[f32; 3]>,
    pub(in crate::animation_viewer) ik_right_solve_target: Option<[f32; 3]>,
    pub(in crate::animation_viewer) ik_left_support_weight: Option<UnitInterval>,
    pub(in crate::animation_viewer) ik_right_support_weight: Option<UnitInterval>,
    pub(in crate::animation_viewer) ik_left_release_active: Option<bool>,
    pub(in crate::animation_viewer) ik_right_release_active: Option<bool>,
    pub(in crate::animation_viewer) ik_left_release_target: Option<[f32; 3]>,
    pub(in crate::animation_viewer) ik_right_release_target: Option<[f32; 3]>,
    #[serde(serialize_with = "serialize_settle_progress")]
    pub(in crate::animation_viewer) ik_settle_progress: Option<adventuresim_tactical_core::animation::SettleProgress>,
    pub(in crate::animation_viewer) ik_left_knee_foot_yaw_offset_degrees: Option<AngularMagnitudeDegrees>,
    pub(in crate::animation_viewer) ik_right_knee_foot_yaw_offset_degrees: Option<AngularMagnitudeDegrees>,
    pub(in crate::animation_viewer) semantic_route_requested_path: SemanticRoutePath,
    pub(in crate::animation_viewer) semantic_route_selected_path: SemanticRoutePath,
    pub(in crate::animation_viewer) semantic_route_runtime_evaluated: bool,
    pub(in crate::animation_viewer) screenshots: BTreeMap<String, String>,
    pub(in crate::animation_viewer) bones: CapturedBones,
}

impl FrameSample {
    /// Required procedural-support readers distinguish absence from measured zero.
    pub(super) fn required_ik_support(&self) -> AnimationValueResult<(UnitInterval, UnitInterval)> {
        self.ik_left_support_weight
            .zip(self.ik_right_support_weight)
            .ok_or(AnimationValueError::UnavailableObservation {
                quantity: AnimationQuantity::SupportObservation,
            })
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(in crate::animation_viewer) struct BoneSample {
    pub(in crate::animation_viewer) position: [f32; 3],
    pub(in crate::animation_viewer) rotation_xyzw: [f32; 4],
    pub(in crate::animation_viewer) terrain_clearance_metres: Option<SignedLength>,
}

#[derive(Debug, Serialize)]
pub(in crate::animation_viewer) struct GlobalBoneFrame {
    pub(in crate::animation_viewer) scenario: String,
    pub(in crate::animation_viewer) scenario_frame: usize,
    pub(in crate::animation_viewer) time_seconds: AnimationElapsed,
    pub(in crate::animation_viewer) action: SkeletonAction,
    pub(in crate::animation_viewer) action_phase:
        adventuresim_tactical_core::animation::UnitInterval,
    pub(in crate::animation_viewer) subject_translation: [f32; 3],
    pub(in crate::animation_viewer) subject_rotation_xyzw: [f32; 4],
    pub(in crate::animation_viewer) bones: Vec<GlobalBoneTransformSample>,
}

#[derive(Debug, Serialize)]
pub(in crate::animation_viewer) struct GlobalBoneTransformSample {
    pub(in crate::animation_viewer) name: String,
    pub(in crate::animation_viewer) target_id: String,
    pub(in crate::animation_viewer) translation: [f32; 3],
    pub(in crate::animation_viewer) rotation_xyzw: [f32; 4],
    pub(in crate::animation_viewer) scale: [f32; 3],
}

pub(super) struct CompletedReport {
    pub(super) output: PathBuf,
    pub(super) manifest: AnimationCaptureManifest,
    pub(super) global_bone_frames: Vec<GlobalBoneFrame>,
}

impl FrameSample {
    pub(super) fn has_finite_bone_transforms(&self) -> bool {
        self.bones.values().all(|bone| {
            bone.position.into_iter().all(f32::is_finite)
                && bone.rotation_xyzw.into_iter().all(f32::is_finite)
        })
    }
}

impl PresentationEventSample {
    /// Advances the sequence without moving the observed sample tick backwards.
    pub(super) fn advances_from(&self, previous: &Self) -> bool {
        self.payload.advances_from(previous.payload)
            && self
                .sample_tick
                .checked_delta(previous.sample_tick)
                .is_some()
    }
}

#[derive(Serialize)]
#[serde(rename = "PresentationEventSample")]
struct PresentationEventView<'a> {
    scenario: &'a str,
    scenario_frame: usize,
    owner: &'a str,
    sequence: u64,
    sample_tick: AnimationSampleTick,
    kind: &'static str,
}

impl Serialize for PresentationEventSample {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let kind = match self.payload {
            LocomotionPresentationPayload::Contact {
                foot: LeadFoot::Left,
                ..
            } => "contact_left",
            LocomotionPresentationPayload::Contact {
                foot: LeadFoot::Right,
                ..
            } => "contact_right",
            LocomotionPresentationPayload::Landing { .. } => "landing",
        };
        PresentationEventView {
            scenario: self.scenario.label(),
            scenario_frame: self.scenario_frame,
            owner: &self.owner,
            sequence: self.payload.encoded_sequence(),
            sample_tick: self.sample_tick,
            kind,
        }
        .serialize(serializer)
    }
}

fn serialize_settle_progress<S: serde::Serializer>(
    progress: &Option<adventuresim_tactical_core::animation::SettleProgress>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    progress.map(|progress| progress.fraction()).serialize(serializer)
}
