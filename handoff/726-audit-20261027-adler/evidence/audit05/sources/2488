//! PD muscles and relative joint damping, stepped once per physics tick.
//!
//! Each joint chases the pose the body fell in with angular-velocity kicks,
//! equal and opposite across the pair. Avian's spherical joints have no
//! motor, so external kicks are the one uniform mechanism for every joint.
//! The kinematic pelvis is never written: it belongs to the authoritative
//! controller, and its children absorb the whole correction.

use adventuresim_tactical_core::animation::{AnimationValueResult, MuscleTone};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

use super::{Ragdolling, definition::PELVIS_BODY};

/// Muscle tone over a ragdoll's life: overwhelmed at impact, gathering
/// itself over the rise, then letting go so the body settles instead of
/// bracing forever.
pub(super) fn muscle_strength(
    age: AnimationElapsed,
    muscles: &RagdollMusclesConfig,
) -> AnimationValueResult<MuscleTone> {
    let age_seconds = age.seconds();
    let rise = quintic_rise(
        (age_seconds - muscles.stun_seconds.seconds()) / muscles.rise_seconds.seconds().max(1e-3),
    );
    let limp = match muscles.limp_at_seconds {
        MuscleRelease::After(start) => {
            1.0 - ((age_seconds - start.seconds()) / muscles.limp_seconds.seconds().max(1e-3))
                .clamp(0.0, 1.0)
        }
        MuscleRelease::Never => 1.0,
    };
    MuscleTone::from_envelope(
        muscles.tone.gain()
            * (muscles.stun_floor.fraction() + (1.0 - muscles.stun_floor.fraction()) * rise)
            * limp,
    )
}

pub(in crate::animation) fn apply_muscles(
    time: Res<Time>,
    mut ragdolls: Query<&mut Ragdolling>,
    mut bodies: Query<(&Rotation, &mut AngularVelocity)>,
) {
    let config = ragdoll_tuning();
    let elapsed = AnimationElapsed::from(time.delta());
    let delta_seconds = elapsed.seconds();
    for mut ragdoll in &mut ragdolls {
        let age = match ragdoll.age_seconds.checked_add(elapsed) {
            Ok(age) => age,
            Err(error) => {
                warn!(%error, "Rejected presentation ragdoll age");
                continue;
            }
        };
        let strength = match muscle_strength(age, &config.muscles) {
            Ok(strength) => strength,
            Err(error) => {
                warn!(%error, "Rejected presentation muscle envelope");
                continue;
            }
        };
        ragdoll.age_seconds = age;
        if strength.gain() <= 1e-4 {
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
            let kick = ((error.to_scaled_axis()
                * config.muscles.proportional_gain.per_second_squared()
                - relative_spin * config.muscles.derivative_gain.per_second())
                * (strength.gain() * delta_seconds))
                .clamp_length_max(
                    config
                        .muscles
                        .maximum_kick_radians_per_second
                        .radians_per_second(),
                );
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
    let factor = match config
        .joint_relative_damping_per_second
        .step_fraction(AnimationElapsed::from(time.delta()))
    {
        Ok(factor) => factor,
        Err(error) => {
            warn!(%error, "Rejected ragdoll joint damping");
            return;
        }
    };
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
            let brake = ((child_spin.0 - parent_spin.0) * factor.fraction()).clamp_length_max(
                config
                    .joint_maximum_brake_radians_per_second
                    .radians_per_second(),
            );
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

fn ragdoll_tuning() -> RagdollConfig {
    runtime_animation_config().ragdoll
}

/// Minimum-jerk rise from zero to one.
fn quintic_rise(u: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    u * u * u * (u * (u * 6.0 - 15.0) + 10.0)
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

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_tactical_core::animation::{
        AngularSpeedRadians, AngularStiffness, PositiveAnimationDuration, ResponseRate,
        UnitInterval,
    };

    fn muscles() -> RagdollMusclesConfig {
        RagdollMusclesConfig {
            tone: MuscleTone::from_gain(1.0).unwrap(),
            proportional_gain: AngularStiffness::from_per_second_squared(600.0).unwrap(),
            derivative_gain: ResponseRate::from_per_second(20.0).unwrap(),
            maximum_kick_radians_per_second: AngularSpeedRadians::from_radians_per_second(60.0)
                .unwrap(),
            stun_seconds: AnimationElapsed::from_seconds(0.15).unwrap(),
            rise_seconds: AnimationElapsed::from_seconds(0.5).unwrap(),
            stun_floor: UnitInterval::from_fraction(0.2).unwrap(),
            limp_at_seconds: MuscleRelease::After(
                PositiveAnimationDuration::try_from(2.0).unwrap(),
            ),
            limp_seconds: AnimationElapsed::from_seconds(1.0).unwrap(),
        }
    }

    #[test]
    fn muscle_envelope_is_stunned_then_gathers_itself_then_lets_go() {
        let config = muscles();
        assert!(
            (muscle_strength(AnimationElapsed::from_seconds(0.0).unwrap(), &config)
                .unwrap()
                .gain()
                - 0.2)
                .abs()
                < 1e-6
        );
        assert!(
            (muscle_strength(AnimationElapsed::from_seconds(0.1).unwrap(), &config)
                .unwrap()
                .gain()
                - 0.2)
                .abs()
                < 1e-6
        );
        let rising = muscle_strength(AnimationElapsed::from_seconds(0.4).unwrap(), &config)
            .unwrap()
            .gain();
        assert!(rising > 0.2 && rising < 1.0);
        assert!(
            (muscle_strength(AnimationElapsed::from_seconds(1.0).unwrap(), &config)
                .unwrap()
                .gain()
                - 1.0)
                .abs()
                < 1e-6
        );
        assert!(
            (muscle_strength(AnimationElapsed::from_seconds(2.5).unwrap(), &config)
                .unwrap()
                .gain()
                - 0.5)
                .abs()
                < 1e-6
        );
        assert_eq!(
            muscle_strength(AnimationElapsed::from_seconds(4.0).unwrap(), &config)
                .unwrap()
                .gain(),
            0.0
        );
    }

    #[test]
    fn muscles_that_never_let_go_hold_full_tone() {
        let config = RagdollMusclesConfig {
            limp_at_seconds: MuscleRelease::Never,
            ..muscles()
        };
        assert!(
            (muscle_strength(AnimationElapsed::from_seconds(30.0).unwrap(), &config)
                .unwrap()
                .gain()
                - 1.0)
                .abs()
                < 1e-6
        );
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
