//! Authored, run-flight and stop-settle foot-height geometry kernels.

use super::*;

/// Native world-height requirements retain either a measured floor or the
/// complete terrain/sole/clearance sum until the final point correction.
#[derive(Clone, Copy, Debug)]
pub(in crate::animation::procedural) enum FootClearanceFloor {
    Measured(SignedLength),
    Terrain {
        surface: SignedLength,
        clearance: SignedLength,
    },
}

impl From<SignedLength> for FootClearanceFloor {
    fn from(height: SignedLength) -> Self {
        Self::Measured(height)
    }
}

impl FootClearanceFloor {
    pub(in crate::animation::procedural) fn apply(
        self,
        mut target: Vec3,
        support_eligible_for_descent: bool,
    ) -> AnimationValueResult<Vec3> {
        let required = match self {
            Self::Measured(height) => height.metres(),
            Self::Terrain { surface, clearance } => {
                surface.metres() + measured_ankle_sole_offset_metres().metres() + clearance.metres()
            }
        };
        target.y = if support_eligible_for_descent {
            required
        } else {
            target.y.max(required)
        };
        checked_foot_target(target)
    }
}

/// Conform only upward from the authored pose, then apply the solve confidence.
pub(in crate::animation::procedural) fn ordinary_terrain_target(
    foot_position: Vec3,
    height: SignedLength,
    weight: UnitInterval,
) -> AnimationValueResult<Vec3> {
    let target = foot_position.with_y(foot_position.y.max(
        height.metres() + measured_ankle_sole_offset_metres().metres()
            - ik_tuning().sole_contact_margin_metres.metres(),
    ));
    checked_foot_target(foot_position.lerp(target, weight.fraction().clamp(0.0, 1.0)))
}

pub(in crate::animation::procedural) fn run_swing_clearance(
    phase_to_contact: CycleDelta,
    planned_progress: Option<UnitInterval>,
) -> AnimationValueResult<SignedLength> {
    checked_world_height(native_run_swing_clearance(
        phase_to_contact,
        planned_progress,
    ))
}

pub(in crate::animation::procedural) fn run_airborne_clearance(
    phase_to_contact: CycleDelta,
    planned_progress: Option<UnitInterval>,
    support_eligible_for_descent: bool,
) -> AnimationValueResult<SignedLength> {
    let clearance = native_run_swing_clearance(phase_to_contact, planned_progress);
    // Admit the completed floor operation, so a capped temporary value is not
    // mistaken for a retained nonfinite measurement.
    checked_world_height(if support_eligible_for_descent {
        clearance
    } else {
        clearance.max(ik_tuning().run_swing_minimum_sole_clearance_metres.metres())
    })
}

pub(in crate::animation::procedural) fn run_airborne_clearance_for_sample(
    just_released: bool,
    phase_to_contact: CycleDelta,
    planned_progress: Option<UnitInterval>,
    support_eligible_for_descent: bool,
) -> AnimationValueResult<SignedLength> {
    if just_released {
        Ok(ik_tuning().run_swing_minimum_sole_clearance_metres)
    } else {
        run_airborne_clearance(
            phase_to_contact,
            planned_progress,
            support_eligible_for_descent,
        )
    }
}

pub(in crate::animation::procedural) fn settle_swing_target(
    start: Vec3,
    landing: Vec3,
    progress: SettleProgress,
) -> AnimationValueResult<Vec3> {
    let progress = progress.fraction().clamp(0.0, 1.0);
    let horizontal = smoothstep(0.0, 1.0, progress);
    let mut target = start.lerp(landing, horizontal);
    target.y +=
        (std::f32::consts::PI * progress).sin() * ik_tuning().settle_step_clearance_metres.metres();
    checked_foot_target(target)
}

pub(in crate::animation::procedural) fn toe_aware_minimum_ankle_y(
    rendered_ankle: Vec3,
    rendered_toe: Vec3,
    desired_ankle_xz: Vec2,
    minimum_toe_clearance: SignedLength,
    terrain_height_at: impl Fn(Vec2) -> Option<f32>,
) -> AnimationValueResult<Option<SignedLength>> {
    let Some(ankle_height) = terrain_height_at(rendered_ankle.xz()) else {
        return Ok(None);
    };
    let ankle_clearance = rendered_ankle.y - ankle_height;
    let Some(toe_height) = terrain_height_at(rendered_toe.xz()) else {
        return Ok(None);
    };
    let toe_clearance = rendered_toe.y - toe_height;
    let ankle_above_toe = ankle_clearance - toe_clearance;
    let Some(desired_height) = terrain_height_at(desired_ankle_xz) else {
        return Ok(None);
    };
    checked_world_height(desired_height + ankle_above_toe + minimum_toe_clearance.metres())
        .map(Some)
}

pub(in crate::animation::procedural) fn transition_toe_clearance_with_rotation_margin(
    rendered_ankle: Vec3,
    rendered_toe: Vec3,
    delta_seconds: AnimationElapsed,
) -> AnimationValueResult<SignedLength> {
    let angular_step = (ik_tuning()
        .airborne_foot_rotation_speed_degrees_per_second
        .degrees_per_second()
        * delta_seconds.seconds())
    .min(90.0)
    .to_radians();
    checked_world_height(
        ik_tuning()
            .terrain_transition_flight_toe_clearance_metres
            .metres()
            + rendered_ankle.distance(rendered_toe) * angular_step.sin(),
    )
}

/// Combine the same sole-first, optional toe-second transition floor for each
/// candidate of the native sphere/disk search, without recomputing a plan.
pub(in crate::animation::procedural) fn transition_minimum_ankle_y(
    terrain: &SceneTerrain,
    rendered: Option<(Vec3, Vec3)>,
    elapsed: AnimationElapsed,
    xz: Vec2,
) -> AnimationValueResult<Option<SignedLength>> {
    let sole = terrain.height_at(xz).map(|height| {
        height
            + measured_ankle_sole_offset_metres().metres()
            + ik_tuning()
                .terrain_transition_flight_toe_clearance_metres
                .metres()
    });
    let toe = rendered.and_then(|(ankle, toe)| {
        let angular_step = (ik_tuning()
            .airborne_foot_rotation_speed_degrees_per_second
            .degrees_per_second()
            * elapsed.seconds())
        .min(90.0)
        .to_radians();
        let margin = ik_tuning()
            .terrain_transition_flight_toe_clearance_metres
            .metres()
            + ankle.distance(toe) * angular_step.sin();
        let ankle_clearance = ankle.y - terrain.height_at(ankle.xz())?;
        let toe_clearance = toe.y - terrain.height_at(toe.xz())?;
        let above_toe = ankle_clearance - toe_clearance;
        Some(terrain.height_at(xz)? + above_toe + margin)
    });
    // Complete the native sole/toe maximum before admitting a retained height.
    sole.into_iter()
        .chain(toe)
        .reduce(f32::max)
        .map(checked_world_height)
        .transpose()
}

pub(in crate::animation::procedural) fn terrain_minimum_ankle_y(
    terrain: &SceneTerrain,
    clearance: SignedLength,
    xz: Vec2,
) -> AnimationValueResult<Option<SignedLength>> {
    terrain
        .height_at(xz)
        .map(|height| {
            checked_world_height(
                height + measured_ankle_sole_offset_metres().metres() + clearance.metres(),
            )
        })
        .transpose()
}

pub(in crate::animation::procedural) fn checked_world_height(
    metres: f32,
) -> AnimationValueResult<SignedLength> {
    SignedLength::from_metres(metres).map_err(|_| AnimationValueError::Overflow {
        quantity: AnimationQuantity::WorldPosition,
    })
}

pub(in crate::animation::procedural) fn checked_foot_target(
    target: Vec3,
) -> AnimationValueResult<Vec3> {
    if !target.is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::WorldPosition,
        });
    }
    Ok(target)
}

// Private constituent of the two completed clearance equations above. It
// carries no retained scalar or application policy handoff.
fn native_run_swing_clearance(
    phase_to_contact: CycleDelta,
    planned_progress: Option<UnitInterval>,
) -> f32 {
    if let Some(progress) = planned_progress {
        let progress = progress.fraction().clamp(0.0, 1.0);
        ik_tuning().run_swing_minimum_sole_clearance_metres.metres() * (1.0 - progress)
            + (std::f32::consts::PI * progress).sin()
                * ik_tuning().run_swing_sole_clearance_metres.metres()
    } else {
        let progress = (1.0 - phase_to_contact.cycles()).clamp(0.0, 1.0);
        ik_tuning().run_swing_minimum_sole_clearance_metres.metres()
            + (std::f32::consts::PI * progress).sin()
                * ik_tuning().run_swing_sole_clearance_metres.metres()
    }
}

#[cfg(test)]
mod tests {
    use super::settle_swing_target;
    use adventuresim_tactical_core::animation::SettleProgress;
    use bevy::prelude::Vec3;

    #[test]
    fn settle_swing_leaves_and_returns_to_ground_only_at_contact() {
        let start = Vec3::new(-0.12, 0.085, 0.25);
        let landing = Vec3::new(-0.12, 0.085, -0.35);
        assert_eq!(
            settle_swing_target(start, landing, SettleProgress::from_progress(0.0).unwrap())
                .unwrap(),
            start
        );
        assert!(
            settle_swing_target(start, landing, SettleProgress::from_progress(0.5).unwrap())
                .unwrap()
                .y
                > start.y + 0.09
        );
        assert!(
            settle_swing_target(start, landing, SettleProgress::from_progress(0.75).unwrap())
                .unwrap()
                .z
                < start.z
        );
        assert!(
            settle_swing_target(start, landing, SettleProgress::from_progress(1.0).unwrap())
                .unwrap()
                .abs_diff_eq(landing, 0.0001)
        );
    }
}
