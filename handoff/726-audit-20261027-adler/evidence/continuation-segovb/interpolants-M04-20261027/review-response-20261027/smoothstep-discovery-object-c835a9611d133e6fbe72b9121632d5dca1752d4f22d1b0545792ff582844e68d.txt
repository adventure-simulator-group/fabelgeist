//! Calibrated movement controls and their intentional exertion speeds.

use super::MovementPace;
use crate::animation::{BodyState, WeaponGuardState};
use crate::{combat_config::TacticalMovementConfig, player::TacticalPlayerView};
use adventuresim_core::linear_speed::{NonnegativeSpeed, SpeedResult};
use adventuresim_core::prelude::*;
use adventuresim_core::unit_interval::UnitInterval;
use bevy::prelude::Vec2;

pub const TACTICAL_WALK_SPEED_METRES_PER_SECOND: NonnegativeSpeed = authored_speed(1.7);
pub const BREATH_PER_METRE_PER_SECOND: f32 = 0.0034;
pub const TACTICAL_BREATH_RESPONSE_SCALE: f32 = 5.0;
pub(super) const REFERENCE_LEG_STRENGTH: f32 = 3.0;
pub(super) const REFERENCE_BURDEN_KG: f32 = 70.0;
pub(super) const MINIMUM_JOG_SPEED_METRES_PER_SECOND: NonnegativeSpeed = authored_speed(1.8);
pub(super) const ELITE_MARATHON_SPEED_METRES_PER_SECOND: NonnegativeSpeed = authored_speed(5.83);
pub(super) const JOG_ENDURANCE_CURVE_EXPONENT: f32 = 1.873_873;
pub(super) const SPRINT_CURVE_BASE_SPEED_METRES_PER_SECOND: NonnegativeSpeed = authored_speed(1.4);
pub(super) const REFERENCE_SPRINT_SPEED_METRES_PER_SECOND: NonnegativeSpeed = authored_speed(8.0);
pub(super) const ELITE_SPRINT_SPEED_METRES_PER_SECOND: NonnegativeSpeed = authored_speed(12.4);
/// Authored ordinary-run reference and fallback for entities without attributes.
pub const TACTICAL_RUN_SPEED_METRES_PER_SECOND: NonnegativeSpeed = authored_speed(5.5);
/// Maximum ordinary controller speed while the weapon guard is raised.
pub const TACTICAL_GUARD_SPEED_METRES_PER_SECOND: NonnegativeSpeed = authored_speed(2.0);
/// Deliberate prone crawl used by the walking control mode.
pub const TACTICAL_PRONE_WALK_SPEED_METRES_PER_SECOND: NonnegativeSpeed = authored_speed(0.45);
/// Maximum urgent unencumbered crawl while sprint is held.
pub const TACTICAL_PRONE_SPEED_METRES_PER_SECOND: NonnegativeSpeed = authored_speed(2.0);
/// Body-local lateral crawl is slower than longitudinal travel.
pub const TACTICAL_PRONE_LATERAL_SPEED_SCALE: f32 = 0.375;
/// Crawling costs three times upright effort; neutral prone jogging divides by this factor.
pub(super) const TACTICAL_PRONE_EFFORT_SCALE: f32 = 3.0;
pub const TACTICAL_ROLL_SPEED_METRES_PER_SECOND: NonnegativeSpeed = authored_speed(1.3);
pub const TACTICAL_DIVE_HORIZONTAL_SPEED_METRES_PER_SECOND: NonnegativeSpeed = authored_speed(7.0);
pub const TACTICAL_QUICKSTEP_SPEED_METRES_PER_SECOND: NonnegativeSpeed = authored_speed(3.5);

/// Existing controller fallback when an entity has no character attributes.
pub(super) const UNATTRIBUTED_JOG_SPEED: NonnegativeSpeed = authored_speed(3.75);

pub fn tactical_jog_speed(endurance: f32) -> SpeedResult<NonnegativeSpeed> {
    let endurance = endurance.clamp(0.0, 5.0);
    if endurance <= 1.0 {
        let smooth_endurance = UnitInterval::from_fraction(endurance)?.smoothstep();
        return TACTICAL_WALK_SPEED_METRES_PER_SECOND
            .checked_lerp(MINIMUM_JOG_SPEED_METRES_PER_SECOND, smooth_endurance);
    }
    let normalized = (endurance - 1.0) / 4.0;
    NonnegativeSpeed::from_metres_per_second(
        MINIMUM_JOG_SPEED_METRES_PER_SECOND.metres_per_second()
            + (ELITE_MARATHON_SPEED_METRES_PER_SECOND.metres_per_second()
                - MINIMUM_JOG_SPEED_METRES_PER_SECOND.metres_per_second())
                * normalized.powf(JOG_ENDURANCE_CURVE_EXPONENT),
    )
}

pub fn tactical_breath_recovery_per_second(endurance: f32) -> SpeedResult<f32> {
    Ok(tactical_jog_speed(endurance)?.metres_per_second() * BREATH_PER_METRE_PER_SECOND)
}

/// Intentional exertion, rather than externally imposed velocity, owns this rate.
pub fn tactical_exhaustion_change_per_second(
    effort_speed: NonnegativeSpeed,
    endurance: f32,
) -> SpeedResult<f32> {
    Ok(
        (effort_speed.metres_per_second() * BREATH_PER_METRE_PER_SECOND
            - tactical_breath_recovery_per_second(endurance)?)
            * TACTICAL_BREATH_RESPONSE_SCALE,
    )
}

/// Movement's additive contribution to tactical exhaustion. A full jog is
/// explicitly capped at neutral, while partial jog input and walking recover
/// exhaustion and sprinting accumulates it. Other exhaustion sources can add
/// their own rates without being cleared by the selected movement pace.
pub fn tactical_movement_exhaustion_change_per_second(
    movement: Option<Vec2>,
    pace: MovementPace,
    weapon_guard: WeaponGuardState,
    body: crate::animation::BodyState,
    endurance: f32,
    sprint_speed: NonnegativeSpeed,
) -> SpeedResult<f32> {
    let jog_speed = tactical_jog_speed(endurance)?;
    let effort_speed = if body == crate::animation::BodyState::Prone {
        tactical_prone_movement_speed_for_pace(movement, pace, jog_speed)?
            .checked_scale(TACTICAL_PRONE_EFFORT_SCALE)?
    } else {
        tactical_movement_speed_for_pace(movement, pace, weapon_guard, jog_speed, sprint_speed)?
    };
    let change = tactical_exhaustion_change_per_second(effort_speed, endurance)?;
    if pace == MovementPace::Jog {
        Ok(change.min(0.0))
    } else {
        Ok(change)
    }
}

pub fn tactical_sprint_speed(
    left_leg_strength: f32,
    right_leg_strength: f32,
    left_leg_health: f32,
    right_leg_health: f32,
    burden_kg: f32,
) -> SpeedResult<NonnegativeSpeed> {
    let effective_strength = ((left_leg_strength * left_leg_health.max(0.0)
        + right_leg_strength * right_leg_health.max(0.0))
        * 0.5)
        .max(0.0);
    let strength_ratio = effective_strength / REFERENCE_LEG_STRENGTH;
    let burden_ratio = REFERENCE_BURDEN_KG / burden_kg.max(1.0);
    let strength_speed = (REFERENCE_SPRINT_SPEED_METRES_PER_SECOND.metres_per_second()
        - SPRINT_CURVE_BASE_SPEED_METRES_PER_SECOND.metres_per_second())
        * strength_ratio;
    NonnegativeSpeed::from_metres_per_second(
        (SPRINT_CURVE_BASE_SPEED_METRES_PER_SECOND.metres_per_second()
            + strength_speed * burden_ratio.sqrt())
        .clamp(
            TACTICAL_WALK_SPEED_METRES_PER_SECOND.metres_per_second(),
            ELITE_SPRINT_SPEED_METRES_PER_SECOND.metres_per_second(),
        ),
    )
}

/// Preserves the analogue movement magnitude after Ahoy normalizes its wish
/// direction. Keyboard input has magnitude one; a half-deflected stick sets a
/// half-speed controller target.
pub fn tactical_movement_speed(movement: Option<Vec2>) -> SpeedResult<NonnegativeSpeed> {
    tactical_movement_speed_for_guard(movement, WeaponGuardState::Lowered)
}

/// Resolves the controller target speed at the server-side Ahoy seam. The
/// radial input magnitude is retained after Ahoy normalizes its wish direction.
pub fn tactical_movement_speed_for_guard(
    movement: Option<Vec2>,
    weapon_guard: WeaponGuardState,
) -> SpeedResult<NonnegativeSpeed> {
    let cap = match weapon_guard {
        WeaponGuardState::Lowered => TACTICAL_RUN_SPEED_METRES_PER_SECOND,
        WeaponGuardState::Raised => TACTICAL_GUARD_SPEED_METRES_PER_SECOND,
    };
    cap.checked_scale(movement.map_or(0.0, |movement| movement.length().clamp(0.0, 1.0)))
}

pub fn tactical_movement_speed_for_pace(
    movement: Option<Vec2>,
    pace: MovementPace,
    weapon_guard: WeaponGuardState,
    jog_speed: NonnegativeSpeed,
    sprint_speed: NonnegativeSpeed,
) -> SpeedResult<NonnegativeSpeed> {
    let requested = match pace {
        MovementPace::Walk => TACTICAL_WALK_SPEED_METRES_PER_SECOND,
        MovementPace::Jog => jog_speed,
        MovementPace::Sprint => sprint_speed,
    };
    let cap = if weapon_guard == WeaponGuardState::Raised {
        match pace {
            MovementPace::Sprint => jog_speed,
            _ => requested.min(TACTICAL_GUARD_SPEED_METRES_PER_SECOND),
        }
    } else {
        requested
    };
    cap.checked_scale(movement.map_or(0.0, |movement| movement.length().clamp(0.0, 1.0)))
}

/// Resolve configured ordinary pace, guard cap and prone effort before the native controller assignment.
pub(super) fn ordinary_controller_speed(
    movement: Option<Vec2>,
    pace: Option<MovementPace>,
    guard: WeaponGuardState,
    body: Option<BodyState>,
    player: Option<TacticalPlayerView<'_, '_, '_>>,
    config: &TacticalMovementConfig,
) -> SpeedResult<NonnegativeSpeed> {
    let speeds = &config.speeds_metres_per_second;
    let input_magnitude = movement.map_or(0.0, |movement| movement.length().clamp(0.0, 1.0));
    let (jog, sprint) = match player {
        Some(player) => {
            let burden = player.body_weight() + player.inventory_weight();
            let jog =
                tactical_jog_speed(player.raw_single_body_part_attr(SimpleAttribute::Endurance))?;
            let sprint = tactical_sprint_speed(
                player.raw_limb_attr(LimbAttribute::Strength, BodyPart::LeftLeg),
                player.raw_limb_attr(LimbAttribute::Strength, BodyPart::RightLeg),
                player.body_part_health(BodyPart::LeftLeg),
                player.body_part_health(BodyPart::RightLeg),
                burden,
            )?;
            (jog, sprint)
        }
        None => (UNATTRIBUTED_JOG_SPEED, speeds.run),
    };
    let requested = pace.map_or(speeds.run, |pace| match pace {
        MovementPace::Walk => speeds.walk,
        MovementPace::Jog => jog,
        MovementPace::Sprint => sprint,
    });
    let cap = if guard == WeaponGuardState::Raised {
        match pace {
            Some(MovementPace::Sprint) => jog,
            _ => requested.min(speeds.raised_guard),
        }
    } else {
        requested
    };
    // Keep the ordinary assignment's admission before the existing prone override.
    let ordinary = cap.checked_scale(input_magnitude)?;
    if matches!(body, Some(BodyState::Prone | BodyState::Supine)) {
        let source = match pace {
            None | Some(MovementPace::Walk) => speeds.prone_walk.metres_per_second(),
            Some(MovementPace::Jog) => jog.metres_per_second() / config.prone_effort_scale,
            Some(MovementPace::Sprint) => speeds.prone.metres_per_second(),
        };
        NonnegativeSpeed::from_metres_per_second(source * input_magnitude)
    } else {
        Ok(ordinary)
    }
}

pub(super) fn tactical_prone_movement_speed_for_pace(
    movement: Option<Vec2>,
    pace: MovementPace,
    upright_jog_speed: NonnegativeSpeed,
) -> SpeedResult<NonnegativeSpeed> {
    let speed = tactical_prone_speed_for_pace(pace, upright_jog_speed)?;
    let movement = movement.unwrap_or_default();
    speed.checked_scale(
        Vec2::new(movement.x * TACTICAL_PRONE_LATERAL_SPEED_SCALE, movement.y)
            .length()
            .clamp(0.0, 1.0),
    )
}

pub(super) fn tactical_prone_speed_for_pace(
    pace: MovementPace,
    upright_jog_speed: NonnegativeSpeed,
) -> SpeedResult<NonnegativeSpeed> {
    match pace {
        MovementPace::Walk => Ok(TACTICAL_PRONE_WALK_SPEED_METRES_PER_SECOND),
        MovementPace::Jog => NonnegativeSpeed::from_metres_per_second(
            upright_jog_speed.metres_per_second() / TACTICAL_PRONE_EFFORT_SCALE,
        ),
        MovementPace::Sprint => Ok(TACTICAL_PRONE_SPEED_METRES_PER_SECOND),
    }
}

const fn authored_speed(value: f32) -> NonnegativeSpeed {
    match NonnegativeSpeed::from_metres_per_second(value) {
        Ok(speed) => speed,
        Err(_) => panic!("invalid built-in movement speed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_paces_are_reference_normalized_and_guard_sprint_becomes_jog() {
        for (endurance, expected_speed) in [
            (
                0.0,
                TACTICAL_WALK_SPEED_METRES_PER_SECOND.metres_per_second(),
            ),
            (1.0, 1.8),
            (2.0, 2.10),
            (3.0, 2.90),
            (4.0, 4.15),
            (
                5.0,
                ELITE_MARATHON_SPEED_METRES_PER_SECOND.metres_per_second(),
            ),
        ] {
            assert!(
                (tactical_jog_speed(endurance).unwrap().metres_per_second() - expected_speed).abs()
                    < 0.015,
                "endurance {endurance} should jog near {expected_speed} m/s"
            );
        }
        assert_eq!(
            tactical_breath_recovery_per_second(3.0).unwrap(),
            tactical_jog_speed(3.0).unwrap().metres_per_second() * BREATH_PER_METRE_PER_SECOND
        );
        assert!(
            tactical_exhaustion_change_per_second(
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(8.0)
                    .unwrap(),
                3.0
            )
            .unwrap()
                > 0.0
        );
        assert_eq!(
            tactical_exhaustion_change_per_second(
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(8.0)
                    .unwrap(),
                3.0
            )
            .unwrap(),
            (8.0 - tactical_jog_speed(3.0).unwrap().metres_per_second())
                * BREATH_PER_METRE_PER_SECOND
                * TACTICAL_BREATH_RESPONSE_SCALE
        );
        assert_eq!(
            tactical_exhaustion_change_per_second(tactical_jog_speed(3.0).unwrap(), 3.0).unwrap(),
            0.0
        );
        assert!(
            tactical_exhaustion_change_per_second(
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(0.0)
                    .unwrap(),
                3.0
            )
            .unwrap()
                < 0.0
        );
        assert_eq!(
            tactical_movement_exhaustion_change_per_second(
                Some(Vec2::Y),
                MovementPace::Jog,
                WeaponGuardState::Lowered,
                crate::animation::BodyState::default(),
                3.0,
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(8.0)
                    .unwrap(),
            )
            .unwrap(),
            0.0
        );
        assert!(
            tactical_movement_exhaustion_change_per_second(
                Some(Vec2::splat(0.71)),
                MovementPace::Jog,
                WeaponGuardState::Lowered,
                crate::animation::BodyState::default(),
                3.0,
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(8.0)
                    .unwrap(),
            )
            .unwrap()
                <= 0.0
        );
        assert!(
            tactical_movement_exhaustion_change_per_second(
                Some(Vec2::Y),
                MovementPace::Sprint,
                WeaponGuardState::Lowered,
                crate::animation::BodyState::default(),
                3.0,
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(8.0)
                    .unwrap(),
            )
            .unwrap()
                > 0.0
        );
        assert_eq!(
            tactical_sprint_speed(
                REFERENCE_LEG_STRENGTH,
                REFERENCE_LEG_STRENGTH,
                1.0,
                1.0,
                REFERENCE_BURDEN_KG,
            )
            .unwrap(),
            REFERENCE_SPRINT_SPEED_METRES_PER_SECOND
        );
        assert_eq!(
            tactical_sprint_speed(5.0, 5.0, 1.0, 1.0, REFERENCE_BURDEN_KG).unwrap(),
            ELITE_SPRINT_SPEED_METRES_PER_SECOND
        );
        assert_eq!(
            tactical_movement_speed_for_pace(
                Some(Vec2::Y),
                MovementPace::Sprint,
                WeaponGuardState::Raised,
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(3.96)
                    .unwrap(),
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(6.5)
                    .unwrap(),
            )
            .unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(3.96)
                .unwrap()
        );
    }

    #[test]
    fn sprint_speed_retains_injury_and_burden_penalties() {
        let healthy = tactical_sprint_speed(3.0, 3.0, 1.0, 1.0, REFERENCE_BURDEN_KG).unwrap();
        let injured = tactical_sprint_speed(3.0, 3.0, 0.5, 0.5, REFERENCE_BURDEN_KG).unwrap();
        let burdened =
            tactical_sprint_speed(3.0, 3.0, 1.0, 1.0, REFERENCE_BURDEN_KG * 2.0).unwrap();
        assert!(injured < healthy);
        assert!(burdened < healthy);
    }

    #[test]
    fn prone_walk_is_fixed_jog_is_neutral_and_sprint_costs_exhaustion() {
        let body = crate::animation::BodyState::Prone;
        for endurance in [0.0, 1.0, 2.0, 3.0, 4.0, 5.0] {
            assert_eq!(
                tactical_prone_movement_speed_for_pace(
                    Some(Vec2::Y),
                    MovementPace::Walk,
                    tactical_jog_speed(endurance).unwrap(),
                )
                .unwrap()
                .metres_per_second(),
                TACTICAL_PRONE_WALK_SPEED_METRES_PER_SECOND.metres_per_second()
            );
            assert_eq!(
                tactical_prone_movement_speed_for_pace(
                    Some(Vec2::Y),
                    MovementPace::Jog,
                    tactical_jog_speed(endurance).unwrap(),
                )
                .unwrap()
                .metres_per_second(),
                tactical_jog_speed(endurance).unwrap().metres_per_second()
                    / TACTICAL_PRONE_EFFORT_SCALE
            );
            assert_eq!(
                tactical_prone_movement_speed_for_pace(
                    Some(Vec2::Y),
                    MovementPace::Sprint,
                    tactical_jog_speed(endurance).unwrap(),
                )
                .unwrap()
                .metres_per_second(),
                TACTICAL_PRONE_SPEED_METRES_PER_SECOND.metres_per_second()
            );
            assert_eq!(
                tactical_prone_movement_speed_for_pace(
                    Some(Vec2::X),
                    MovementPace::Sprint,
                    tactical_jog_speed(endurance).unwrap(),
                )
                .unwrap()
                .metres_per_second(),
                TACTICAL_PRONE_SPEED_METRES_PER_SECOND.metres_per_second()
                    * TACTICAL_PRONE_LATERAL_SPEED_SCALE
            );
            assert_eq!(
                tactical_movement_exhaustion_change_per_second(
                    Some(Vec2::Y),
                    MovementPace::Jog,
                    WeaponGuardState::Lowered,
                    body,
                    endurance,
                    REFERENCE_SPRINT_SPEED_METRES_PER_SECOND,
                )
                .unwrap(),
                0.0
            );
            assert!(
                tactical_movement_exhaustion_change_per_second(
                    Some(Vec2::Y),
                    MovementPace::Walk,
                    WeaponGuardState::Lowered,
                    body,
                    endurance,
                    REFERENCE_SPRINT_SPEED_METRES_PER_SECOND,
                )
                .unwrap()
                    < 0.0
            );
            assert!(
                tactical_movement_exhaustion_change_per_second(
                    Some(Vec2::Y),
                    MovementPace::Sprint,
                    WeaponGuardState::Lowered,
                    body,
                    endurance,
                    REFERENCE_SPRINT_SPEED_METRES_PER_SECOND,
                )
                .unwrap()
                    > 0.0
            );
        }
    }

    #[test]
    fn movement_speed_preserves_stick_magnitude_and_caps_diagonals() {
        for (guard, cap) in [
            (
                WeaponGuardState::Lowered,
                TACTICAL_RUN_SPEED_METRES_PER_SECOND,
            ),
            (
                WeaponGuardState::Raised,
                TACTICAL_GUARD_SPEED_METRES_PER_SECOND,
            ),
        ] {
            assert_eq!(
                tactical_movement_speed_for_guard(None, guard)
                    .unwrap()
                    .metres_per_second(),
                0.0
            );
            assert_eq!(
                tactical_movement_speed_for_guard(Some(Vec2::new(0.3, 0.4)), guard)
                    .unwrap()
                    .metres_per_second(),
                cap.metres_per_second() * 0.5
            );
            assert_eq!(
                tactical_movement_speed_for_guard(Some(Vec2::ONE), guard)
                    .unwrap()
                    .metres_per_second(),
                cap.metres_per_second()
            );
        }
        assert_eq!(
            tactical_movement_speed(Some(Vec2::new(0.3, 0.4)))
                .unwrap()
                .metres_per_second(),
            TACTICAL_RUN_SPEED_METRES_PER_SECOND.metres_per_second() * 0.5
        );
    }
}
