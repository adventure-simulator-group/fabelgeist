//! Procedural sample roles and units retained between presentation updates.

use super::body_values::{
    BodyAngle, BodyAngularVelocity, BodyLocalAcceleration, BodyLocalVelocity,
};
use super::landing_compression_for_action;
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

impl LocomotionHeightState {
    pub(super) fn begin_landing(&mut self, skeleton: &SkeletonState) {
        if skeleton.is_posture_transitioning() {
            // Authored dive/get-up recovery owns the complete skeleton.
            // Do not retain airborne feet or layer ordinary upright
            // landing compression over its terrain-timed contact blend.
            self.landing_compression = 0.0;
            self.landing_compression_target = 0.0;
            self.landing_absorption_metres_per_second = 0.0;
            self.landing_recovery_metres_per_second = 0.0;
        } else {
            self.landing_compression_target = landing_compression_for_action(
                locomotion_profile(skeleton),
                skeleton.landing_impact_speed,
                self.last_action,
            );
            // Continue the downward impact velocity into the pelvis. The
            // legs reach peak flexion over physical time instead of
            // changing their bend plane by the full compression in one
            // presentation sample.
            self.landing_absorption_metres_per_second =
                skeleton.landing_impact_speed.metres_per_second().max(0.001);
            self.landing_recovery_metres_per_second = self.landing_compression_target
                / locomotion_profile(skeleton)
                    .landing
                    .recovery_seconds
                    .seconds();
        }
        self.landing_left_foot_target = None;
        self.landing_right_foot_target = None;
        self.landing_plant_owner_position = None;
        self.landing_plant_tick = None;
        self.landing_plant_resync_tick = None;
    }
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
