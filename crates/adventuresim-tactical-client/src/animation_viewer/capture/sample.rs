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

impl FrameSample {
    pub(super) fn capture(sample: CaptureSample<'_>) -> CaptureSampleResult<Self> {
        let CaptureSample { frame, skeleton, subject_global, bouncy_bones_telemetry, raised_footwork, body_response, height_state, leg_ik, semantic_route, root_distance_metres, left_support_weight, right_support_weight, evaluation_bones } = sample;
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
            bouncy_maximum_deflection_radians: bouncy_bones_telemetry.maximum_deflection_radians.clone()
                .map_err(RejectedCaptureAdmission::Geometry)?,
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
                .map(|state| SignedAngleDegrees::from_degrees(state.pitch_radians.radians().to_degrees()))
                .transpose().map_err(RejectedCaptureAdmission::Locomotion)?,
            body_lean_roll_degrees: body_response
                .map(|state| SignedAngleDegrees::from_degrees(state.roll_radians.radians().to_degrees()))
                .transpose().map_err(RejectedCaptureAdmission::Locomotion)?,
            landing_compression_metres: height_state
                .map(|state| state.landing_compression),
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
            ik_left_authored_target: leg_ik.and_then(|diagnostics| diagnostics.left_authored_target).map(|value| value.to_array()),
            ik_right_authored_target: leg_ik.and_then(|diagnostics| diagnostics.right_authored_target).map(|value| value.to_array()),
            ik_left_planned_contact: leg_ik.and_then(|diagnostics| diagnostics.left_planned_contact).map(|value| value.to_array()),
            ik_right_planned_contact: leg_ik.and_then(|diagnostics| diagnostics.right_planned_contact).map(|value| value.to_array()),
            ik_settle_capture_point: leg_ik.and_then(|diagnostics| diagnostics.settle_capture_point).map(|value| value.to_array()),
            ik_left_solve_target: leg_ik.and_then(|diagnostics| diagnostics.left_solve_target).map(|value| value.to_array()),
            ik_right_solve_target: leg_ik.and_then(|diagnostics| diagnostics.right_solve_target).map(|value| value.to_array()),
            ik_left_support_weight: leg_ik.and_then(|diagnostics| diagnostics.left_support_weight),
            ik_right_support_weight: leg_ik.and_then(|diagnostics| diagnostics.right_support_weight),
            ik_left_release_active: leg_ik.map(|diagnostics| diagnostics.left_release_active),
            ik_right_release_active: leg_ik.map(|diagnostics| diagnostics.right_release_active),
            ik_left_release_target: leg_ik.and_then(|diagnostics| diagnostics.left_release_target).map(|value| value.to_array()),
            ik_right_release_target: leg_ik.and_then(|diagnostics| diagnostics.right_release_target).map(|value| value.to_array()),
            ik_settle_progress: leg_ik.and_then(|diagnostics| diagnostics.settle_progress),
            ik_left_knee_foot_yaw_offset_degrees: leg_ik.map(|diagnostics| diagnostics.left_knee_foot_yaw_offset_degrees),
            ik_right_knee_foot_yaw_offset_degrees: leg_ik.map(|diagnostics| diagnostics.right_knee_foot_yaw_offset_degrees),
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
