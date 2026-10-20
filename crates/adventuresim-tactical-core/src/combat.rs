use crate::animation::{
    AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult,
};
use bevy_enhanced_input::prelude::InputAction;

const MELEE_LUNGE_DISTANCE_EPSILON_METRES: f32 = 1.0e-5;

/// Finite forward drive produced by the conservative lunge motor. Nonpositive
/// drive means a moving lunge cannot arrive; it does not prevent an in-range swing.
#[derive(Clone, Copy, Debug)]
pub struct ForwardLungeAcceleration(f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeleeContactTiming {
    pub delay: AnimationElapsed,
    pub contact: AnimationElapsed,
}

#[derive(Debug, InputAction, Default)]
#[action_output(f32)]
pub struct Attack;

#[derive(Debug, InputAction, Default)]
#[action_output(f32)]
pub struct Dodge;

/// Assisted contact outcome. A non-arriving lunge leaves ordinary attacks valid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeleeLungeArrival {
    NotNeeded,
    ArrivesIn(AnimationElapsed),
    CannotArrive,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeleeLunge {
    None,
    Forward { distance_metres: f32 },
    Quickstep { distance_metres: f32 },
}

impl ForwardLungeAcceleration {
    pub fn from_metres_per_second_squared(value: f32) -> AnimationValueResult<Self> {
        if !value.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::LungeAcceleration,
            });
        }
        Ok(Self(value))
    }
    pub fn metres_per_second_squared(self) -> f32 {
        self.0
    }
}

impl MeleeLungeArrival {
    /// Client/server unassisted swings keep their authored contact timing.
    pub fn contact_timing(
        self,
        authored_windup: AnimationElapsed,
    ) -> AnimationValueResult<MeleeContactTiming> {
        // This is the original contact/delay kernel, including its zero-sign
        // normalization. The outcome itself remains explicit at every caller.
        let predicted = match self {
            Self::ArrivesIn(arrival) => arrival.seconds(),
            Self::NotNeeded | Self::CannotArrive => 0.0,
        };
        let authored = authored_windup.seconds();
        let contact = authored.max(predicted).max(0.0);
        let delay = (contact - authored.max(0.0)).max(0.0);
        Ok(MeleeContactTiming {
            delay: AnimationElapsed::from_seconds(delay)?,
            contact: AnimationElapsed::from_seconds(contact)?,
        })
    }
}

pub fn melee_lunge_range_window_metres() -> f32 {
    crate::combat_config::runtime_melee_authority_config().lunge_range_window_metres
}

pub fn melee_lunge_quickstep_threshold_metres() -> f32 {
    crate::combat_config::runtime_melee_authority_config().lunge_quickstep_threshold_metres
}

#[must_use]
pub fn melee_interaction_range(arm_reach: f32, weapon_reach: f32) -> f32 {
    arm_reach.max(0.0) + weapon_reach.max(0.0)
}

pub fn melee_lunge_arrival(
    lunge: MeleeLunge,
    forward_speed_metres_per_second: adventuresim_core::linear_speed::NonnegativeSpeed,
    forward_acceleration_metres_per_second_squared: ForwardLungeAcceleration,
    quickstep_duration: crate::animation::PositiveAnimationDuration,
) -> AnimationValueResult<MeleeLungeArrival> {
    let fixed_tick_safety = 1.0 / crate::animation::locomotion_sample_rate().hertz();
    let seconds = match lunge {
        MeleeLunge::None => return Ok(MeleeLungeArrival::NotNeeded),
        MeleeLunge::Forward { distance_metres } => {
            if distance_metres <= 0.0 {
                return Ok(MeleeLungeArrival::ArrivesIn(
                    AnimationElapsed::from_seconds(fixed_tick_safety)?,
                ));
            }
            let speed = forward_speed_metres_per_second.metres_per_second();
            let acceleration =
                forward_acceleration_metres_per_second_squared.metres_per_second_squared();
            if !distance_metres.is_finite() {
                return Err(AnimationValueError::NonFinite {
                    quantity: AnimationQuantity::Elapsed,
                });
            }
            if speed <= 0.0 || acceleration <= 0.0 {
                return Ok(MeleeLungeArrival::CannotArrive);
            }
            let acceleration_seconds = speed / acceleration;
            let acceleration_distance = 0.5 * acceleration * acceleration_seconds.powi(2);
            let travel_seconds = if distance_metres <= acceleration_distance {
                (2.0 * distance_metres.max(0.0) / acceleration).sqrt()
            } else {
                acceleration_seconds + (distance_metres - acceleration_distance) / speed
            };
            travel_seconds + fixed_tick_safety
        }
        MeleeLunge::Quickstep { .. } => quickstep_duration.seconds().max(0.0) + fixed_tick_safety,
    };
    if !seconds.is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::Elapsed,
        });
    }
    Ok(MeleeLungeArrival::ArrivesIn(
        AnimationElapsed::from_seconds(seconds)?,
    ))
}

#[must_use]
pub fn conservative_forward_lunge_acceleration(
    motor: &crate::combat_config::CharacterMotorConfig,
) -> AnimationValueResult<ForwardLungeAcceleration> {
    let minimum_healthy_strength_scale = 1.0 / motor.reference_leg_strength.max(1.0);
    let drive = motor.reference_ground_drive_force_newtons * minimum_healthy_strength_scale
        / motor.fallback_character_mass_kg.max(1.0);
    ForwardLungeAcceleration::from_metres_per_second_squared(
        drive.min(motor.gravity_metres_per_second_squared * motor.traction_coefficient),
    )
}

/// Chooses the attack movement needed to bring a target into ordinary melee
/// range. Targets farther away than one complete quickstep remain valid attack
/// attempts, but do not cause movement.
#[must_use]
pub fn melee_lunge(
    separation_metres: f32,
    arm_reach: f32,
    weapon_reach: f32,
    quickstep_distance_metres: f32,
) -> MeleeLunge {
    if !separation_metres.is_finite() || !quickstep_distance_metres.is_finite() {
        return MeleeLunge::None;
    }
    let gap = (separation_metres - melee_interaction_range(arm_reach, weapon_reach)).max(0.0);
    if gap <= melee_lunge_range_window_metres() + MELEE_LUNGE_DISTANCE_EPSILON_METRES
        || gap > quickstep_distance_metres.max(0.0) + MELEE_LUNGE_DISTANCE_EPSILON_METRES
    {
        MeleeLunge::None
    } else if gap > melee_lunge_quickstep_threshold_metres() + MELEE_LUNGE_DISTANCE_EPSILON_METRES {
        MeleeLunge::Quickstep {
            distance_metres: gap,
        }
    } else {
        MeleeLunge::Forward {
            distance_metres: gap,
        }
    }
}

#[must_use]
pub fn maximum_melee_lunge_range(
    arm_reach: f32,
    weapon_reach: f32,
    quickstep_distance_metres: f32,
) -> f32 {
    melee_interaction_range(arm_reach, weapon_reach) + quickstep_distance_metres.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn melee_lunge_respects_window_mode_threshold_and_maximum_travel() {
        let range = melee_interaction_range(0.55, 0.8);
        assert_eq!(melee_lunge(range + 0.10, 0.55, 0.8, 1.0), MeleeLunge::None);
        assert!(matches!(
            melee_lunge(range + 0.11, 0.55, 0.8, 1.0),
            MeleeLunge::Forward { distance_metres } if (distance_metres - 0.11).abs() < 1.0e-5
        ));
        assert!(matches!(
            melee_lunge(range + 0.51, 0.55, 0.8, 1.0),
            MeleeLunge::Quickstep { distance_metres } if (distance_metres - 0.51).abs() < 1.0e-5
        ));
        assert_eq!(melee_lunge(range + 1.01, 0.55, 0.8, 1.0), MeleeLunge::None);
        assert_eq!(maximum_melee_lunge_range(0.55, 0.8, 1.0), range + 1.0);
    }

    #[test]
    fn fist_and_weapon_ranges_add_anatomy_exactly_once() {
        assert!((melee_interaction_range(0.526_801, 0.0) - 0.526_801).abs() < 1.0e-6);
        assert!((melee_interaction_range(0.526_801, 0.8) - 1.326_801).abs() < 1.0e-6);
    }

    #[test]
    fn lunge_delay_uses_existing_forward_and_quickstep_motion_timing() {
        assert_eq!(
            melee_lunge_arrival(
                MeleeLunge::None,
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(4.0)
                    .unwrap(),
                crate::combat::ForwardLungeAcceleration::from_metres_per_second_squared(4.0)
                    .unwrap(),
                crate::animation::PositiveAnimationDuration::try_from(0.5).unwrap()
            )
            .unwrap(),
            MeleeLungeArrival::NotNeeded
        );
        assert!(
            ((match melee_lunge_arrival(
                MeleeLunge::Forward {
                    distance_metres: 0.4
                },
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(4.0)
                    .unwrap(),
                crate::combat::ForwardLungeAcceleration::from_metres_per_second_squared(4.0)
                    .unwrap(),
                crate::animation::PositiveAnimationDuration::try_from(0.5).unwrap(),
            )
            .unwrap()
            {
                MeleeLungeArrival::ArrivesIn(elapsed) => elapsed.seconds(),
                other => panic!("expected lunge arrival: {other:?}"),
            }) - ((0.2_f32).sqrt() + 1.0 / crate::animation::locomotion_sample_rate().hertz()))
            .abs()
                < 1.0e-6
        );
        assert!(
            ((match melee_lunge_arrival(
                MeleeLunge::Quickstep {
                    distance_metres: 0.8
                },
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(4.0)
                    .unwrap(),
                crate::combat::ForwardLungeAcceleration::from_metres_per_second_squared(4.0)
                    .unwrap(),
                crate::animation::PositiveAnimationDuration::try_from(0.5).unwrap(),
            )
            .unwrap()
            {
                MeleeLungeArrival::ArrivesIn(elapsed) => elapsed.seconds(),
                other => panic!("expected lunge arrival: {other:?}"),
            }) - (0.5 + 1.0 / crate::animation::locomotion_sample_rate().hertz()))
            .abs()
                < 1.0e-6
        );
        let config = crate::combat_config::TacticalCombatConfig::default();
        let default_acceleration =
            conservative_forward_lunge_acceleration(&config.movement.motor).unwrap();
        for distance in [0.404, 0.441] {
            assert!(
                (match melee_lunge_arrival(
                    MeleeLunge::Forward {
                        distance_metres: distance,
                    },
                    config.movement.speeds_metres_per_second.run,
                    default_acceleration,
                    config.movement.maneuvers.quickstep_duration_seconds,
                )
                .unwrap()
                {
                    MeleeLungeArrival::ArrivesIn(elapsed) => elapsed.seconds(),
                    other => panic!("expected lunge arrival: {other:?}"),
                }) > 0.18
            );
        }
    }
}
