//! Procedural sample roles and units retained between presentation updates.

use adventuresim_tactical_core::animation::{AnimationSampleTick, LandingSequence};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

// The upright lowered-guard humanoid_unarmed root/pelvis rotations lift its
// pelvis by about 33 mm at passing. Subtract that measured authored rise only
// from the additive run-flight treatment; authored walk/pelvis translation
// remains untouched.
#[derive(Component, Debug, Clone, Copy, Default)]
pub(crate) struct LocomotionHeightState {
    pub(super) initialization: HeightInitialization,
    pub(super) amplitude: f32,
    pub(super) authored_rise_compensation: f32,
    pub(super) displayed_wave: f32,
    pub(super) wave_transition_offset: f32,
    pub(crate) landing_compression: f32,
    pub(super) landing_compression_target: f32,
    pub(super) landing_absorption_metres_per_second: f32,
    pub(super) landing_recovery_metres_per_second: f32,
    pub(super) landing_left_foot_target: Option<Vec3>,
    pub(super) landing_right_foot_target: Option<Vec3>,
    pub(super) landing_plant_owner_position: Option<Vec3>,
    pub(super) landing_plant_tick: Option<AnimationSampleTick>,
    pub(super) landing_plant_resync_tick: Option<AnimationSampleTick>,
    pub(super) last_landing_sequence: LandingSequence,
    pub(super) last_guard: Option<WeaponGuardState>,
    pub(super) last_posture: Option<Posture>,
    pub(super) last_action: Option<SkeletonAction>,
    pub(super) last_ground_contact: Option<GroundContact>,
    pub(super) evaluation_tick: Option<AnimationSampleTick>,
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

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct BodyAngle(f32);

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct BodyAngularVelocity(Vec2);

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct BodyLocalVelocity(Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct BodyLocalAcceleration(Vec3);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum HeightInitialization {
    #[default]
    Pending,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GroundContact {
    Grounded,
    Airborne,
}

impl From<&SkeletonState> for GroundContact {
    fn from(state: &SkeletonState) -> Self {
        if state.is_grounded() {
            Self::Grounded
        } else {
            Self::Airborne
        }
    }
}

impl BodyAngle {
    pub(super) const ZERO: Self = Self(0.0);
    pub(super) fn from_radians(value: f32) -> Self {
        Self(value)
    }
    pub(crate) fn radians(self) -> f32 {
        self.0
    }
}

impl BodyAngularVelocity {
    pub(super) const ZERO: Self = Self(Vec2::ZERO);
    pub(super) fn from_radians_per_second(value: Vec2) -> Self {
        Self(value)
    }
    pub(super) fn radians_per_second(self) -> Vec2 {
        self.0
    }
}

impl BodyLocalVelocity {
    pub(super) fn from_metres_per_second(value: Vec3) -> Self {
        Self(value)
    }
    pub(super) fn metres_per_second(self) -> Vec3 {
        self.0
    }
}

impl BodyLocalAcceleration {
    pub(super) const ZERO: Self = Self(Vec3::ZERO);
    pub(super) fn from_metres_per_second_squared(value: Vec3) -> Self {
        Self(value)
    }
    pub(super) fn metres_per_second_squared(self) -> Vec3 {
        self.0
    }
}
