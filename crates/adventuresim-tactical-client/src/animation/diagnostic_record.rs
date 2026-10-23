//! Native diagnostic JSON records, after application-state interpretation.

use super::*;
use adventuresim_tactical_core::{
    city_layout::grounding::SupportElevation, scene_coordinates::ScenePlanPoint,
};
use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
pub(super) struct ScheduleCompletion {
    pub(super) count: u64,
    pub(super) elapsed_micros: u64,
}

#[derive(Serialize)]
pub(super) struct DiagnosticTransform {
    pub(super) translation: [f32; 3],
    pub(super) rotation_xyzw: [f32; 4],
}

#[derive(Serialize)]
pub(super) struct DiagnosticBone<'a> {
    pub(super) name: &'a str,
    pub(super) target_id: String,
    pub(super) translation: [f32; 3],
    pub(super) rotation_xyzw: [f32; 4],
    pub(super) scale: [f32; 3],
    pub(super) terrain_clearance_metres: Option<f32>,
}

#[derive(Serialize)]
pub(super) struct DiagnosticClip {
    pub(super) weight: BlendWeight,
    pub(super) time_seconds: ClipTime,
    pub(super) mirrored_weight: BlendWeight,
}

#[derive(Serialize)]
pub(super) struct DiagnosticPlayback {
    pub(super) use_authored_bind_pose: bool,
    pub(super) whole_body_mirror: UnitInterval,
    pub(super) ordinary_locomotion_active: bool,
    pub(super) clips: Vec<DiagnosticClip>,
}

#[derive(Serialize)]
pub(super) struct DiagnosticRoute<'a> {
    pub(super) requested_path: semantic_route::SemanticRoutePath,
    pub(super) path: semantic_route::SemanticRoutePath,
    pub(super) inputs: &'a semantic_route::SemanticRouteInputs,
    pub(super) runtime_evaluated: bool,
    pub(super) evaluation_equivalent: bool,
}

#[derive(Serialize)]
pub(super) struct AnimationDiagnosticRecord<'a> {
    pub(super) trace_format: &'static str,
    pub(super) scenario: &'static str,
    pub(super) scenario_frame: u64,
    pub(super) elapsed_seconds: f64,
    pub(super) wall_clock_unix_micros: u64,
    pub(super) render_delta_seconds: f32,
    pub(super) render_schedule_completion: Option<ScheduleCompletion>,
    pub(super) input: Option<&'a diagnostics::DiagnosticInputStatus>,
    pub(super) controller_transform: DiagnosticTransform,
    pub(super) controller_global_transform: DiagnosticTransform,
    pub(super) controller_physics_rotation_xyzw: Option<[f32; 4]>,
    pub(super) subject_translation: [f32; 3],
    pub(super) subject_rotation_xyzw: [f32; 4],
    pub(super) action: SkeletonAction,
    pub(super) action_phase: UnitInterval,
    pub(super) authored_weapon_target: Option<DiagnosticTransform>,
    pub(super) bones: Vec<DiagnosticBone<'a>>,
    pub(super) terrain_height: Option<SupportElevation>,
    pub(super) controller_height_above_terrain: Option<f32>,
    pub(super) authoritative: &'a SkeletonState,
    pub(super) presented: &'a SkeletonState,
    pub(super) presentation_phase_error_remaining: CycleDelta,
    pub(super) presentation_phase_prediction_delta: CycleDelta,
    pub(super) presentation_phase_correction_delta: CycleDelta,
    pub(super) presentation_phase_measurement_error: Option<CycleDelta>,
    pub(super) presentation_phase_source_changed: bool,
    pub(super) evaluation: AnimationEvaluation,
    pub(super) semantic_route: Option<DiagnosticRoute<'a>>,
    pub(super) playback: DiagnosticPlayback,
}

impl From<&Transform> for DiagnosticTransform {
    fn from(transform: &Transform) -> Self {
        Self {
            translation: transform.translation.to_array(),
            rotation_xyzw: transform.rotation.to_array(),
        }
    }
}

impl<'a> DiagnosticBone<'a> {
    pub(super) fn from_global(
        target: AnimationTargetId,
        name: Option<&'a Name>,
        transform: &GlobalTransform,
        terrain: Option<&SceneTerrain>,
    ) -> Self {
        let (scale, rotation, translation) = transform.to_scale_rotation_translation();
        let terrain_clearance_metres = ScenePlanPoint::from_metres(translation.xz())
            .and_then(|point| terrain.and_then(|terrain| terrain.height_at(point)))
            .map(|height| translation.y - height.metres());
        Self {
            name: name.map_or("<unnamed>", Name::as_str),
            target_id: capture_animation_target_id(target),
            translation: translation.to_array(),
            rotation_xyzw: rotation.to_array(),
            scale: scale.to_array(),
            terrain_clearance_metres,
        }
    }
}
