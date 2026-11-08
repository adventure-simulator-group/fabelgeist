//! Admitted native capture observations become the retained report sample.

use super::*;
use adventuresim_tactical_core::animation::SignedAngleDegrees;

type CaptureSampleResult<T> = Result<T, RejectedCaptureAdmission>;

pub(super) struct CaptureSample<'a> {
    pub(super) frame: &'a PlannedFrame,
    pub(super) skeleton: &'a SkeletonState,
    pub(super) subject_global: &'a GlobalTransform,
    pub(super) bouncy_bones_telemetry: &'a BouncyBonesTelemetry,
    pub(super) raised_footwork: Option<&'a RaisedFootworkState>,
    pub(super) body_response: Option<&'a LocomotionBodyResponseState>,
    pub(super) height_state: Option<&'a LocomotionHeightState>,
    pub(super) leg_ik: Option<LegIkDiagnostics>,
    pub(super) semantic_route: &'a SemanticRouteTrace,
    pub(super) root_distance_metres: SignedLength,
    pub(super) left_support_weight: UnitInterval,
    pub(super) right_support_weight: UnitInterval,
    pub(super) evaluation_bones: CapturedBones,
}

struct CapturedIkPoints {
    left_authored_target: Option<[f32; 3]>,
    right_authored_target: Option<[f32; 3]>,
    left_planned_contact: Option<[f32; 3]>,
    right_planned_contact: Option<[f32; 3]>,
    settle_capture_point: Option<[f32; 3]>,
    left_solve_target: Option<[f32; 3]>,
    right_solve_target: Option<[f32; 3]>,
    left_release_target: Option<[f32; 3]>,
    right_release_target: Option<[f32; 3]>,
}

struct CapturedContact {
    sequence: ContactSequence,
    foot: LeadFoot,
}

struct CapturedLean {
    pitch: Option<SignedAngleDegrees>,
    roll: Option<SignedAngleDegrees>,
}

struct CapturedDirections {
    world_travel: [f32; 3],
    desired_forward: [f32; 3],
}

impl CapturedIkPoints {
    fn from_diagnostics(ik: Option<LegIkDiagnostics>) -> Self {
        Self {
            left_authored_target: ik
                .and_then(|diagnostics| diagnostics.left_authored_target)
                .map(|value| value.to_array()),
            right_authored_target: ik
                .and_then(|diagnostics| diagnostics.right_authored_target)
                .map(|value| value.to_array()),
            left_planned_contact: ik
                .and_then(|diagnostics| diagnostics.left_planned_contact)
                .map(|value| value.to_array()),
            right_planned_contact: ik
                .and_then(|diagnostics| diagnostics.right_planned_contact)
                .map(|value| value.to_array()),
            settle_capture_point: ik
                .and_then(|diagnostics| diagnostics.settle_capture_point)
                .map(|value| value.to_array()),
            left_solve_target: ik
                .and_then(|diagnostics| diagnostics.left_solve_target)
                .map(|value| value.to_array()),
            right_solve_target: ik
                .and_then(|diagnostics| diagnostics.right_solve_target)
                .map(|value| value.to_array()),
            left_release_target: ik
                .and_then(|diagnostics| diagnostics.left_release_target)
                .map(|value| value.to_array()),
            right_release_target: ik
                .and_then(|diagnostics| diagnostics.right_release_target)
                .map(|value| value.to_array()),
        }
    }
}

impl CapturedContact {
    fn from_states(skeleton: &SkeletonState, raised: Option<&RaisedFootworkState>) -> Self {
        // The raised stepper owns its counter even while its step is uninitialized.
        Self {
            sequence: raised.map_or(
                skeleton.contact_sequence,
                RaisedFootworkState::step_sequence,
            ),
            foot: raised
                .and_then(RaisedFootworkState::contact_foot)
                .unwrap_or(skeleton.contact_foot),
        }
    }
}

impl CapturedLean {
    fn from_response(response: Option<&LocomotionBodyResponseState>) -> CaptureSampleResult<Self> {
        let pitch = response
            .map(|state| {
                SignedAngleDegrees::from_degrees(state.pitch_radians.radians().to_degrees())
            })
            .transpose()
            .map_err(RejectedCaptureAdmission::Locomotion)?;
        let roll = response
            .map(|state| {
                SignedAngleDegrees::from_degrees(state.roll_radians.radians().to_degrees())
            })
            .transpose()
            .map_err(RejectedCaptureAdmission::Locomotion)?;
        Ok(Self { pitch, roll })
    }
}

impl CapturedDirections {
    fn from_frame(frame: &PlannedFrame) -> Self {
        let yaw = controller_yaw(Quat::from_rotation_y(frame.camera_yaw.radians()));
        let travel = yaw * Vec3::new(frame.local_direction.x, 0.0, frame.local_direction.y);
        let desired = if frame.weapon_guard == WeaponGuardState::Raised
            || matches!(frame.action, SkeletonAction::Attack | SkeletonAction::Block)
        {
            yaw * Vec3::NEG_Z
        } else {
            travel.normalize_or_zero()
        };
        Self {
            world_travel: travel.normalize_or_zero().to_array(),
            desired_forward: desired.to_array(),
        }
    }
}

impl FrameSample {
    pub(super) fn capture(sample: CaptureSample<'_>) -> CaptureSampleResult<Self> {
        let CaptureSample {
            frame,
            skeleton,
            subject_global,
            bouncy_bones_telemetry,
            raised_footwork,
            body_response,
            height_state,
            leg_ik,
            semantic_route,
            root_distance_metres,
            left_support_weight,
            right_support_weight,
            evaluation_bones,
        } = sample;
        let maximum_deflection = bouncy_bones_telemetry
            .maximum_deflection_radians
            .clone()
            .map_err(RejectedCaptureAdmission::Geometry)?;
        let contact = CapturedContact::from_states(skeleton, raised_footwork);
        let lean = CapturedLean::from_response(body_response)?;
        let directions = CapturedDirections::from_frame(frame);
        let ik_points = CapturedIkPoints::from_diagnostics(leg_ik);
        let desired_left_foot_target = raised_footwork.and_then(|state| state.left_solve_target);
        let desired_right_foot_target = raised_footwork.and_then(|state| state.right_solve_target);
        Ok(Self {
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
            bouncy_maximum_deflection_radians: maximum_deflection,
            contact_sequence: contact.sequence,
            contact_foot: contact.foot,
            landing_sequence: skeleton.landing_sequence,
            landing_impact_speed: skeleton.landing_impact_speed,
            body_lean_pitch_degrees: lean.pitch,
            body_lean_roll_degrees: lean.roll,
            landing_compression_metres: height_state.map(|state| state.landing_compression),
            root_distance_metres,
            root_position_metres: subject_global.translation().to_array(),
            world_travel_direction: directions.world_travel,
            desired_body_forward_direction: directions.desired_forward,
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
            ik_left_authored_target: ik_points.left_authored_target,
            ik_right_authored_target: ik_points.right_authored_target,
            ik_left_planned_contact: ik_points.left_planned_contact,
            ik_right_planned_contact: ik_points.right_planned_contact,
            ik_settle_capture_point: ik_points.settle_capture_point,
            ik_left_solve_target: ik_points.left_solve_target,
            ik_right_solve_target: ik_points.right_solve_target,
            ik_left_support_weight: leg_ik.and_then(|diagnostics| diagnostics.left_support_weight),
            ik_right_support_weight: leg_ik
                .and_then(|diagnostics| diagnostics.right_support_weight),
            ik_left_release_active: leg_ik.map(|diagnostics| diagnostics.left_release_active),
            ik_right_release_active: leg_ik.map(|diagnostics| diagnostics.right_release_active),
            ik_left_release_target: ik_points.left_release_target,
            ik_right_release_target: ik_points.right_release_target,
            ik_settle_progress: leg_ik.and_then(|diagnostics| diagnostics.settle_progress),
            ik_left_knee_foot_yaw_offset_degrees: leg_ik
                .map(|diagnostics| diagnostics.left_knee_foot_yaw_offset_degrees),
            ik_right_knee_foot_yaw_offset_degrees: leg_ik
                .map(|diagnostics| diagnostics.right_knee_foot_yaw_offset_degrees),
            semantic_route_requested_path: semantic_route.requested_path,
            semantic_route_selected_path: semantic_route.path,
            semantic_route_runtime_evaluated: true,
            screenshots: VIEWS
                .into_iter()
                .map(|view| (view.slug().to_owned(), view.screenshot_name(&frame)))
                .collect(),
            bones: evaluation_bones,
        })
    }
}
