use crate::animation::{
    AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult,
};
use bevy_enhanced_input::prelude::InputAction;
use fabelgeist_geometry::{GeometryError, SignedLength};

pub use attack_key::MeleeAttackKey;

mod attack_key;

pub const MELEE_LUNGE_DISTANCE_EPSILON_METRES: SignedLength = match SignedLength::from_metres(1.0e-5) {
    Ok(value) => value,
    Err(_) => panic!("authored melee lunge distance tolerance must be finite"),
};

pub type MeleeGeometryResult<T> = Result<T, MeleeGeometryError>;

/// Geometry and its contact schedule retain their original checked causes.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MeleeGeometryError {
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error(transparent)]
    Animation(#[from] AnimationValueError),
    #[error(transparent)]
    Timing(#[from] adventuresim_core::elapsed::ElapsedError),
    #[error(transparent)]
    Performance(#[from] adventuresim_core::unit_interval::FractionError),
}

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
    Forward { distance_metres: SignedLength },
    Quickstep { distance_metres: SignedLength },
}

/// Scene-plan separation, ordinary reach and maximum assisted travel share
/// checked metre leaves while retaining their distinct roles.
#[derive(Clone, Copy, Debug)]
pub struct MeleeLungeGeometry {
    pub separation: SignedLength,
    pub interaction_range: SignedLength,
    pub maximum_travel: SignedLength,
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

pub fn melee_lunge_range_window_metres() -> SignedLength {
    crate::combat_config::runtime_melee_authority_config().lunge_range_window_metres
}

pub fn melee_lunge_quickstep_threshold_metres() -> SignedLength {
    crate::combat_config::runtime_melee_authority_config().lunge_quickstep_threshold_metres
}

pub fn melee_interaction_range(arm_reach: SignedLength, weapon_reach: SignedLength) -> MeleeGeometryResult<SignedLength> {
    // Preserve the existing max-zero normalization and f32 sum at this kernel.
    checked_lunge_metres(arm_reach.metres().max(0.0) + weapon_reach.metres().max(0.0))
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
            let distance_metres = distance_metres.metres();
            if distance_metres <= 0.0 {
                return Ok(MeleeLungeArrival::ArrivesIn(
                    AnimationElapsed::from_seconds(fixed_tick_safety)?,
                ));
            }
            let speed = forward_speed_metres_per_second.metres_per_second();
            let acceleration =
                forward_acceleration_metres_per_second_squared.metres_per_second_squared();
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
pub fn melee_lunge(geometry: MeleeLungeGeometry) -> MeleeGeometryResult<MeleeLunge> {
    let gap = (geometry.separation.metres() - geometry.interaction_range.metres().max(0.0)).max(0.0);
    let gap = checked_lunge_metres(gap)?;
    Ok(if gap.metres() <= melee_lunge_range_window_metres().metres() + MELEE_LUNGE_DISTANCE_EPSILON_METRES.metres()
        || gap.metres() > geometry.maximum_travel.metres().max(0.0) + MELEE_LUNGE_DISTANCE_EPSILON_METRES.metres()
    {
        MeleeLunge::None
    } else if gap.metres() > melee_lunge_quickstep_threshold_metres().metres() + MELEE_LUNGE_DISTANCE_EPSILON_METRES.metres() {
        MeleeLunge::Quickstep { distance_metres: gap }
    } else {
        MeleeLunge::Forward { distance_metres: gap }
    })
}

pub fn maximum_melee_lunge_range(
    arm_reach: SignedLength,
    weapon_reach: SignedLength,
    quickstep_distance_metres: SignedLength,
) -> MeleeGeometryResult<SignedLength> {
    let reach = melee_interaction_range(arm_reach, weapon_reach)?;
    checked_lunge_metres(reach.metres() + quickstep_distance_metres.metres().max(0.0))
}

fn checked_lunge_metres(value: f32) -> MeleeGeometryResult<SignedLength> {
    // All input leaves are finite. A nonfinite completed arithmetic result is
    // overflow of this owned calculation, rather than invalid external input.
    SignedLength::from_metres(value).map_err(|_| AnimationValueError::Overflow {
        quantity: AnimationQuantity::Length,
    }.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn melee_lunge_respects_window_mode_threshold_and_maximum_travel() {
        let range = melee_interaction_range(SignedLength::from_metres(0.55).unwrap(), SignedLength::from_metres(0.8).unwrap()).unwrap().metres();
        assert_eq!(melee_lunge(MeleeLungeGeometry { separation: SignedLength::from_metres(range + 0.10).unwrap(), interaction_range: melee_interaction_range(SignedLength::from_metres(0.55).unwrap(), SignedLength::from_metres(0.8).unwrap()).unwrap(), maximum_travel: SignedLength::from_metres(1.0).unwrap() }).unwrap(), MeleeLunge::None);
        assert!(matches!(
            melee_lunge(MeleeLungeGeometry { separation: SignedLength::from_metres(range + 0.11).unwrap(), interaction_range: melee_interaction_range(SignedLength::from_metres(0.55).unwrap(), SignedLength::from_metres(0.8).unwrap()).unwrap(), maximum_travel: SignedLength::from_metres(1.0).unwrap() }).unwrap(),
            MeleeLunge::Forward { distance_metres } if (distance_metres.metres() - 0.11).abs() < 1.0e-5
        ));
        assert!(matches!(
            melee_lunge(MeleeLungeGeometry { separation: SignedLength::from_metres(range + 0.51).unwrap(), interaction_range: melee_interaction_range(SignedLength::from_metres(0.55).unwrap(), SignedLength::from_metres(0.8).unwrap()).unwrap(), maximum_travel: SignedLength::from_metres(1.0).unwrap() }).unwrap(),
            MeleeLunge::Quickstep { distance_metres } if (distance_metres.metres() - 0.51).abs() < 1.0e-5
        ));
        assert_eq!(melee_lunge(MeleeLungeGeometry { separation: SignedLength::from_metres(range + 1.01).unwrap(), interaction_range: melee_interaction_range(SignedLength::from_metres(0.55).unwrap(), SignedLength::from_metres(0.8).unwrap()).unwrap(), maximum_travel: SignedLength::from_metres(1.0).unwrap() }).unwrap(), MeleeLunge::None);
        assert_eq!(maximum_melee_lunge_range(SignedLength::from_metres(0.55).unwrap(), SignedLength::from_metres(0.8).unwrap(), SignedLength::from_metres(1.0).unwrap()).unwrap().metres(), range + 1.0);
    }

    #[test]
    fn fist_and_weapon_ranges_add_anatomy_exactly_once() {
        assert!((melee_interaction_range(SignedLength::from_metres(0.526_801).unwrap(), SignedLength::from_metres(0.0).unwrap()).unwrap().metres() - 0.526_801).abs() < 1.0e-6);
        assert!((melee_interaction_range(SignedLength::from_metres(0.526_801).unwrap(), SignedLength::from_metres(0.8).unwrap()).unwrap().metres() - 1.326_801).abs() < 1.0e-6);
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
                    distance_metres: SignedLength::from_metres(0.4).unwrap()
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
                    distance_metres: SignedLength::from_metres(0.8).unwrap()
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
                        distance_metres: SignedLength::from_metres(distance).unwrap(),
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
