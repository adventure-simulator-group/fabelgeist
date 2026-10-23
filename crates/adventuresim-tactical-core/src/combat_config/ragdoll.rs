//! Tuning for the client presentation ragdoll.

use crate::animation::{
    AngularSpeedRadians, AngularStiffness, AnimationElapsed, AnimationValueError,
    AnimationValueResult, MuscleTone, NonnegativeLength, PositiveAnimationDuration, ResponseRate,
    UnitInterval,
};
use adventuresim_building_generator::spatial_geometry::PositiveLength;
use adventuresim_core::linear_speed::NonnegativeSpeed;
use serde::{Deserialize, Serialize};

/// The client presentation ragdoll: articulated bodies hanging off the
/// authoritative pelvis, braced by PD muscles toward the pose they fell in.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RagdollConfig {
    pub capsules: RagdollCapsulesConfig,
    pub muscles: RagdollMusclesConfig,
    /// Relative angular velocity drained across each joint per second: what
    /// stops limbs pendulum-swinging against each other.
    pub joint_relative_damping_per_second: ResponseRate,
    /// Cap on the per-step relative brake, radians per second.
    pub joint_maximum_brake_radians_per_second: AngularSpeedRadians,
    /// Speculative contact margin so a fast-falling limb is caught before it
    /// buries into a collider between steps.
    pub speculative_contact_margin_metres: NonnegativeLength,
    /// Entry velocities sampled from the animation are clamped here so a
    /// clip loop seam cannot read as a launch.
    pub maximum_seed_speed_metres_per_second: NonnegativeSpeed,
    pub maximum_seed_spin_radians_per_second: AngularSpeedRadians,
    /// Horizontal velocity kept by a body resting on terrain each frame.
    pub terrain_horizontal_velocity_retention: UnitInterval,
}

/// PD muscles bracing each joint toward its entry pose. Strength follows a
/// closed-form envelope: overwhelmed at impact, a minimum-jerk rise back to
/// full tone, then letting go so the body settles.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RagdollMusclesConfig {
    /// Overall tone; zero is a fully passive corpse.
    pub tone: MuscleTone,
    /// Radians per second squared of pull per radian of pose error.
    pub proportional_gain: AngularStiffness,
    /// Drain on the relative spin the pull creates.
    pub derivative_gain: ResponseRate,
    /// Per-step angular-velocity kick cap, radians per second.
    pub maximum_kick_radians_per_second: AngularSpeedRadians,
    /// Window right after entry during which muscles run at `stun_floor`.
    pub stun_seconds: AnimationElapsed,
    /// Seconds of quintic rise from the stun floor back to full tone.
    pub rise_seconds: AnimationElapsed,
    /// Strength during the stun, as a fraction of full tone.
    pub stun_floor: UnitInterval,
    /// Optional age at which the muscles start letting go; null never lets go.
    pub limp_at_seconds: MuscleRelease,
    /// Seconds over which tone fades to zero once letting go.
    pub limp_seconds: AnimationElapsed,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RagdollCapsulesConfig {
    pub pelvis: RagdollCapsuleConfig,
    pub chest: RagdollCapsuleConfig,
    pub head: RagdollCapsuleConfig,
    pub thigh: RagdollCapsuleConfig,
    pub shin: RagdollCapsuleConfig,
    pub foot: RagdollCapsuleConfig,
    pub upper_arm: RagdollCapsuleConfig,
    pub forearm: RagdollCapsuleConfig,
    pub hand: RagdollCapsuleConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RagdollCapsuleConfig {
    pub radius_metres: PositiveLength,
    pub length_metres: PositiveLength,
}

/// Explicit absence of muscle release, separate from a real positive release age.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Option<f32>", into = "Option<f32>")]
pub enum MuscleRelease {
    Never,
    After(PositiveAnimationDuration),
}

impl TryFrom<Option<f32>> for MuscleRelease {
    type Error = AnimationValueError;
    fn try_from(seconds: Option<f32>) -> AnimationValueResult<Self> {
        match seconds {
            None => Ok(Self::Never),
            Some(seconds) => PositiveAnimationDuration::try_from(seconds).map(Self::After),
        }
    }
}
impl From<MuscleRelease> for Option<f32> {
    fn from(release: MuscleRelease) -> Self {
        match release {
            MuscleRelease::Never => None,
            MuscleRelease::After(age) => Some(age.seconds()),
        }
    }
}

impl Default for RagdollConfig {
    fn default() -> Self {
        super::defaults::canonical().animation.ragdoll
    }
}
