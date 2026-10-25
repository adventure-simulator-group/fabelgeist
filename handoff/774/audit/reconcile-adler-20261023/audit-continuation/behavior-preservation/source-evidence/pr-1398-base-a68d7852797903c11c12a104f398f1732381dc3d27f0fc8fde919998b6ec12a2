//! PD muscles and relative joint damping, stepped once per physics tick.
//!
//! Each joint chases the pose the body fell in with angular-velocity kicks,
//! equal and opposite across the pair. Avian's spherical joints have no
//! motor, so external kicks are the one uniform mechanism for every joint.
//! The kinematic pelvis is never written: it belongs to the authoritative
//! controller, and its children absorb the whole correction.

use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

use super::{Ragdolling, definition::PELVIS_BODY};

fn ragdoll_tuning() -> RagdollConfig {
    runtime_animation_config().ragdoll
}

/// Minimum-jerk rise from zero to one.
fn quintic_rise(u: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    u * u * u * (u * (u * 6.0 - 15.0) + 10.0)
}

/// Muscle tone over a ragdoll's life: overwhelmed at impact, gathering
/// itself over the rise, then letting go so the body settles instead of
/// bracing forever.
pub(super) fn muscle_strength(age_seconds: f32, muscles: &RagdollMusclesConfig) -> f32 {
    let rise = quintic_rise((age_seconds - muscles.stun_seconds) / muscles.rise_seconds.max(1e-3));
    let limp = if muscles.limp_at_seconds > 0.0 {
        1.0 - ((age_seconds - muscles.limp_at_seconds) / muscles.limp_seconds.max(1e-3))
            .clamp(0.0, 1.0)
    } else {
        1.0
    };
    muscles.tone * (muscles.stun_floor + (1.0 - muscles.stun_floor) * rise) * limp
}

/// Split an angular-velocity kick across a joint pair: equal and opposite,
/// unless the parent is the kinematic pelvis, which the child then absorbs
/// entirely.
fn split_kick(parent: usize, kick: Vec3) -> (Vec3, Vec3) {
    if parent == PELVIS_BODY {
        (Vec3::ZERO, kick)
    } else {
        (-kick * 0.5, kick * 0.5)
    }
}

pub(in crate::animation) fn apply_muscles(
    time: Res<Time>,
    mut ragdolls: Query<&mut Ragdolling>,
    mut bodies: Query<(&Rotation, &mut AngularVelocity)>,
) {
    let config = ragdoll_tuning();
    let delta_seconds = time.delta_secs();
    for mut ragdoll in &mut ragdolls {
        ragdoll.age_seconds += delta_seconds;
        let strength = muscle_strength(ragdoll.age_seconds, &config.muscles);
        if strength <= 1e-4 {
            continue;
        }
        for muscle in &ragdoll.muscles {
            let (Some(parent), Some(child)) =
                (ragdoll.bodies[muscle.parent], ragdoll.bodies[muscle.child])
            else {
                continue;
            };
            let Ok(
                [
                    (parent_rotation, mut parent_spin),
                    (child_rotation, mut child_spin),
                ],
            ) = bodies.get_many_mut([parent, child])
            else {
                continue;
            };
            // The rotation carrying the child onto its target pose relative
            // to the parent, along the short arc.
            let mut error = (parent_rotation.0 * muscle.relative) * child_rotation.0.inverse();
            if error.w < 0.0 {
                error = -error;
            }
            let relative_spin = child_spin.0 - parent_spin.0;
            let kick = ((error.to_scaled_axis() * config.muscles.proportional_gain
                - relative_spin * config.muscles.derivative_gain)
                * (strength * delta_seconds))
                .clamp_length_max(config.muscles.maximum_kick_radians_per_second);
            if kick.length_squared() <= 1e-8 {
                continue;
            }
            let (parent_delta, child_delta) = split_kick(muscle.parent, kick);
            if parent_delta != Vec3::ZERO {
                parent_spin.0 += parent_delta;
            }
            child_spin.0 += child_delta;
        }
    }
}

/// Drain the spin of each child relative to its parent so limbs stop
/// flopping while the body as a whole keeps tumbling. Runs after the
/// muscles so it also drains what the pull created.
pub(in crate::animation) fn damp_joints(
    time: Res<Time>,
    ragdolls: Query<&Ragdolling>,
    mut velocities: Query<&mut AngularVelocity>,
) {
    let config = ragdoll_tuning();
    let factor = (config.joint_relative_damping_per_second * time.delta_secs()).min(1.0);
    for ragdoll in &ragdolls {
        for muscle in &ragdoll.muscles {
            let (Some(parent), Some(child)) =
                (ragdoll.bodies[muscle.parent], ragdoll.bodies[muscle.child])
            else {
                continue;
            };
            let Ok([mut parent_spin, mut child_spin]) = velocities.get_many_mut([parent, child])
            else {
                continue;
            };
            let brake = ((child_spin.0 - parent_spin.0) * factor)
                .clamp_length_max(config.joint_maximum_brake_radians_per_second);
            // Only touch the `Mut` when there is something to drain.
            if brake.length_squared() <= 1e-8 {
                continue;
            }
            child_spin.0 -= brake;
            if muscle.parent != PELVIS_BODY {
                parent_spin.0 += brake * 0.5;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn muscles() -> RagdollMusclesConfig {
        RagdollMusclesConfig {
            tone: 1.0,
            proportional_gain: 600.0,
            derivative_gain: 20.0,
            maximum_kick_radians_per_second: 60.0,
            stun_seconds: 0.15,
            rise_seconds: 0.5,
            stun_floor: 0.2,
            limp_at_seconds: 2.0,
            limp_seconds: 1.0,
        }
    }

    #[test]
    fn muscle_envelope_is_stunned_then_gathers_itself_then_lets_go() {
        let config = muscles();
        assert!((muscle_strength(0.0, &config) - 0.2).abs() < 1e-6);
        assert!((muscle_strength(0.1, &config) - 0.2).abs() < 1e-6);
        let rising = muscle_strength(0.4, &config);
        assert!(rising > 0.2 && rising < 1.0);
        assert!((muscle_strength(1.0, &config) - 1.0).abs() < 1e-6);
        assert!((muscle_strength(2.5, &config) - 0.5).abs() < 1e-6);
        assert_eq!(muscle_strength(4.0, &config), 0.0);
    }

    #[test]
    fn muscles_that_never_let_go_hold_full_tone() {
        let config = RagdollMusclesConfig {
            limp_at_seconds: 0.0,
            ..muscles()
        };
        assert!((muscle_strength(30.0, &config) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn the_kinematic_pelvis_is_never_kicked() {
        let kick = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(split_kick(PELVIS_BODY, kick), (Vec3::ZERO, kick));
        let (parent, child) = split_kick(3, kick);
        assert_eq!(child - parent, kick);
        assert_eq!(parent + child, Vec3::ZERO);
    }
}
