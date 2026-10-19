//! Procedural sample roles and units retained between presentation updates.

use super::body_values::{
    BodyAngle, BodyAngularVelocity, BodyLocalAcceleration, BodyLocalVelocity,
};
use super::height_values::{HeightDistance, HeightSpeed};
use adventuresim_building_generator::spatial_geometry::SignedLength;
use adventuresim_tactical_core::animation::{AnimationSampleTick, LandingSequence};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

// The upright lowered-guard humanoid_unarmed root/pelvis rotations lift its
// pelvis by about 33 mm at passing. Subtract that measured authored rise only
// from the additive run-flight treatment; authored walk/pelvis translation
// remains untouched.
#[derive(Component, Debug, Clone, Copy, Default)]
pub(crate) struct LocomotionHeightState {
    pub(super) history: Option<HeightHistory>,
    pub(super) amplitude: HeightDistance,
    pub(super) authored_rise_compensation: HeightDistance,
    pub(super) displayed_wave: SignedLength,
    pub(super) wave_transition_offset: SignedLength,
    pub(crate) landing_compression: HeightDistance,
    pub(super) landing_compression_target: HeightDistance,
    pub(super) landing_absorption_metres_per_second: HeightSpeed,
    pub(super) landing_recovery_metres_per_second: HeightSpeed,
    pub(super) landing_left_foot_target: Option<Vec3>,
    pub(super) landing_right_foot_target: Option<Vec3>,
    pub(super) landing_plant_owner_position: Option<Vec3>,
    pub(super) landing_plant_tick: Option<AnimationSampleTick>,
    pub(super) landing_plant_resync_tick: Option<AnimationSampleTick>,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub(crate) struct LocomotionBodyResponseState {
    pub(crate) pitch_radians: BodyAngle,
    pub(crate) roll_radians: BodyAngle,
    pub(super) angular_velocity_radians_per_second: BodyAngularVelocity,
    pub(super) target_pitch_radians: BodyAngle,
    pub(super) target_roll_radians: BodyAngle,
    pub(super) smoothed_body_acceleration: BodyLocalAcceleration,
    pub(super) last_body_velocity: BodyLocalVelocity,
    pub(super) last_tick: Option<AnimationSampleTick>,
    pub(super) last_posture: Option<Posture>,
    pub(super) last_action: Option<SkeletonAction>,
    pub(super) last_ground_contact: Option<GroundContact>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct HeightHistory {
    pub(super) guard: WeaponGuardState,
    pub(super) posture: Posture,
    pub(super) action: SkeletonAction,
    pub(super) ground_contact: GroundContact,
    pub(super) landing_sequence: LandingSequence,
    pub(super) evaluation_tick: AnimationSampleTick,
}

impl HeightHistory {
    pub(super) fn observe(skeleton: &SkeletonState) -> Self {
        Self {
            guard: skeleton.weapon_guard(),
            posture: skeleton.posture(),
            action: skeleton.action_kind(),
            ground_contact: GroundContact::from(skeleton),
            landing_sequence: skeleton.landing_sequence,
            evaluation_tick: skeleton.locomotion_sample_tick,
        }
    }
}
