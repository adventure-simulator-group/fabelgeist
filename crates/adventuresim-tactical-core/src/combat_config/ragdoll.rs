//! Tuning for the client presentation ragdoll.

use serde::{Deserialize, Serialize};

use super::TacticalCombatConfigError;

/// The client presentation ragdoll: articulated bodies hanging off the
/// authoritative pelvis, braced by PD muscles toward the pose they fell in.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RagdollConfig {
    pub capsules: RagdollCapsulesConfig,
    pub muscles: RagdollMusclesConfig,
    /// Relative angular velocity drained across each joint per second: what
    /// stops limbs pendulum-swinging against each other.
    pub joint_relative_damping_per_second: f32,
    /// Cap on the per-step relative brake, radians per second.
    pub joint_maximum_brake_radians_per_second: f32,
    /// Speculative contact margin so a fast-falling limb is caught before it
    /// buries into a collider between steps.
    pub speculative_contact_margin_metres: f32,
    /// Entry velocities sampled from the animation are clamped here so a
    /// clip loop seam cannot read as a launch.
    pub maximum_seed_speed_metres_per_second: f32,
    pub maximum_seed_spin_radians_per_second: f32,
    /// Horizontal velocity kept by a body resting on terrain each frame.
    pub terrain_horizontal_velocity_retention: f32,
}

/// PD muscles bracing each joint toward its entry pose. Strength follows a
/// closed-form envelope: overwhelmed at impact, a minimum-jerk rise back to
/// full tone, then letting go so the body settles.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RagdollMusclesConfig {
    /// Overall tone; zero is a fully passive corpse.
    pub tone: f32,
    /// Radians per second squared of pull per radian of pose error.
    pub proportional_gain: f32,
    /// Drain on the relative spin the pull creates.
    pub derivative_gain: f32,
    /// Per-step angular-velocity kick cap, radians per second.
    pub maximum_kick_radians_per_second: f32,
    /// Window right after entry during which muscles run at `stun_floor`.
    pub stun_seconds: f32,
    /// Seconds of quintic rise from the stun floor back to full tone.
    pub rise_seconds: f32,
    /// Strength during the stun, as a fraction of full tone.
    pub stun_floor: f32,
    /// Age at which the muscles start letting go; zero never lets go.
    pub limp_at_seconds: f32,
    /// Seconds over which tone fades to zero once letting go.
    pub limp_seconds: f32,
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
    pub radius_metres: f32,
    pub length_metres: f32,
}

impl RagdollConfig {
    pub(super) fn validate(&self) -> Result<(), TacticalCombatConfigError> {
        let finite_nonnegative = |value: f32| value.is_finite() && value >= 0.0;
        let ragdoll = self;
        let capsules = ragdoll.capsules;
        if [
            capsules.pelvis,
            capsules.chest,
            capsules.head,
            capsules.thigh,
            capsules.shin,
            capsules.foot,
            capsules.upper_arm,
            capsules.forearm,
            capsules.hand,
        ]
        .into_iter()
        .any(|capsule| {
            !capsule.radius_metres.is_finite()
                || !capsule.length_metres.is_finite()
                || capsule.radius_metres <= 0.0
                || capsule.length_metres <= 0.0
        }) {
            return Err(TacticalCombatConfigError::Validation(
                "invalid ragdoll capsule tuning",
            ));
        }
        let muscles = ragdoll.muscles;
        if ![
            ragdoll.joint_relative_damping_per_second,
            ragdoll.joint_maximum_brake_radians_per_second,
            ragdoll.speculative_contact_margin_metres,
            ragdoll.maximum_seed_speed_metres_per_second,
            ragdoll.maximum_seed_spin_radians_per_second,
            ragdoll.terrain_horizontal_velocity_retention,
            muscles.tone,
            muscles.proportional_gain,
            muscles.derivative_gain,
            muscles.maximum_kick_radians_per_second,
            muscles.stun_seconds,
            muscles.rise_seconds,
            muscles.stun_floor,
            muscles.limp_at_seconds,
            muscles.limp_seconds,
        ]
        .into_iter()
        .all(finite_nonnegative)
            || muscles.stun_floor > 1.0
            || ragdoll.terrain_horizontal_velocity_retention > 1.0
        {
            return Err(TacticalCombatConfigError::Validation(
                "invalid ragdoll muscle tuning",
            ));
        }
        Ok(())
    }
}

impl Default for RagdollConfig {
    fn default() -> Self {
        Self {
            capsules: RagdollCapsulesConfig {
                pelvis: RagdollCapsuleConfig {
                    radius_metres: 0.18,
                    length_metres: 0.24,
                },
                chest: RagdollCapsuleConfig {
                    radius_metres: 0.18,
                    length_metres: 0.28,
                },
                head: RagdollCapsuleConfig {
                    radius_metres: 0.15,
                    length_metres: 0.16,
                },
                thigh: RagdollCapsuleConfig {
                    radius_metres: 0.10,
                    length_metres: 0.36,
                },
                shin: RagdollCapsuleConfig {
                    radius_metres: 0.085,
                    length_metres: 0.34,
                },
                foot: RagdollCapsuleConfig {
                    radius_metres: 0.09,
                    length_metres: 0.20,
                },
                upper_arm: RagdollCapsuleConfig {
                    radius_metres: 0.075,
                    length_metres: 0.27,
                },
                forearm: RagdollCapsuleConfig {
                    radius_metres: 0.065,
                    length_metres: 0.25,
                },
                hand: RagdollCapsuleConfig {
                    radius_metres: 0.07,
                    length_metres: 0.14,
                },
            },
            muscles: RagdollMusclesConfig {
                tone: 1.0,
                proportional_gain: 600.0,
                derivative_gain: 20.0,
                maximum_kick_radians_per_second: 60.0,
                stun_seconds: 0.15,
                rise_seconds: 0.5,
                stun_floor: 0.2,
                limp_at_seconds: 2.0,
                limp_seconds: 1.0,
            },
            joint_relative_damping_per_second: 8.0,
            joint_maximum_brake_radians_per_second: 30.0,
            speculative_contact_margin_metres: 0.5,
            maximum_seed_speed_metres_per_second: 10.0,
            maximum_seed_spin_radians_per_second: 30.0,
            terrain_horizontal_velocity_retention: 0.72,
        }
    }
}
