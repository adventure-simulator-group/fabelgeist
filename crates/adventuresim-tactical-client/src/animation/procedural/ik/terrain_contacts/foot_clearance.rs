//! Authored, run-flight and stop-settle foot-height geometry kernels.

use super::*;
use adventuresim_building_generator::spatial_geometry::Elevation;
use adventuresim_tactical_core::scene_coordinates::Scene;

/// Foot-height requirements retain either a checked minimum or the
/// complete terrain/sole/clearance sum until the final point correction.
#[derive(Clone, Copy, Debug)]
pub(in crate::animation::procedural) enum FootClearanceFloor {
    Minimum(Elevation<Scene>),
    Terrain {
        surface: SupportElevation,
        clearance: SignedLength,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::animation::procedural) enum FootDescent {
    RaiseOnly,
    FollowFloor,
}

impl FootDescent {
    pub(in crate::animation::procedural) fn permits_descent(self) -> bool {
        matches!(self, Self::FollowFloor)
    }
}

impl FootClearanceFloor {
    pub(in crate::animation::procedural) fn minimum_height(self) -> AnimationValueResult<Elevation<Scene>> {
        match self {
            Self::Minimum(height) => Ok(height),
            Self::Terrain { surface, clearance } => checked_scene_elevation(
                surface.metres() + measured_ankle_sole_offset_metres().metres() + clearance.metres(),
            ),
        }
    }

    pub(in crate::animation::procedural) fn apply(
        self,
        mut target: Vec3,
        descent: FootDescent,
    ) -> AnimationValueResult<Vec3> {
        let required = self.minimum_height()?.metres();
        target.y = if descent.permits_descent() {
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
    height: SupportElevation,
    weight: UnitInterval,
) -> AnimationValueResult<Vec3> {
    let minimum = checked_scene_elevation(
        height.metres() + measured_ankle_sole_offset_metres().metres()
            - ik_tuning().sole_contact_margin_metres.metres(),
    )?;
    let target = foot_position.with_y(foot_position.y.max(minimum.metres()));
    checked_foot_target(foot_position.lerp(target, weight.fraction().clamp(0.0, 1.0)))
}

pub(in crate::animation::procedural) fn run_swing_clearance(
    phase_to_contact: CycleDelta,
    planned_progress: Option<UnitInterval>,
) -> AnimationValueResult<SignedLength> {
    checked_signed_length(native_run_swing_clearance(
        phase_to_contact,
        planned_progress,
    ))
}

pub(in crate::animation::procedural) fn run_airborne_clearance(
    phase_to_contact: CycleDelta,
    planned_progress: Option<UnitInterval>,
    descent: FootDescent,
) -> AnimationValueResult<SignedLength> {
    let clearance = native_run_swing_clearance(phase_to_contact, planned_progress);
    // Admit the completed floor operation, so a capped temporary value is not
    // mistaken for a retained nonfinite measurement.
    checked_signed_length(if descent.permits_descent() {
        clearance
    } else {
        clearance.max(ik_tuning().run_swing_minimum_sole_clearance_metres.metres())
    })
}

pub(in crate::animation::procedural) fn run_airborne_clearance_for_sample(
    just_released: bool,
    phase_to_contact: CycleDelta,
    planned_progress: Option<UnitInterval>,
    descent: FootDescent,
) -> AnimationValueResult<SignedLength> {
    if just_released {
        Ok(ik_tuning().run_swing_minimum_sole_clearance_metres)
    } else {
        run_airborne_clearance(
            phase_to_contact,
            planned_progress,
            descent,
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

/// Complete the planned run swing's terrain and clearance correction together.
pub(in crate::animation::procedural) fn conform_run_swing_target(
    terrain: &SceneTerrain,
    desired_target: Vec3,
    phase_to_contact: CycleDelta,
    planned_progress: Option<UnitInterval>,
) -> AnimationValueResult<Vec3> {
    let point = checked_scene_point_metres(desired_target.xz())?;
    let Some(surface) = terrain.height_at(point) else {
        return Ok(desired_target);
    };
    FootClearanceFloor::Terrain {
        surface,
        clearance: run_swing_clearance(phase_to_contact, planned_progress)?,
    }
    .apply(desired_target, FootDescent::RaiseOnly)
}

/// Keep the settle arc, terrain blend and visible toe floor in one correction.
pub(in crate::animation::procedural) fn conform_settle_swing_target(
    terrain: &SceneTerrain,
    mut desired_target: Vec3,
    authored_foot: Vec3,
    progress: SettleProgress,
    terrain_blend: UnitInterval,
    rendered: Option<(Vec3, Vec3)>,
    elapsed: AnimationElapsed,
) -> AnimationValueResult<Vec3> {
    let point = checked_scene_point_metres(desired_target.xz())?;
    let Some(height) = terrain.height_at(point) else {
        return Ok(desired_target);
    };
    let minimum = checked_scene_elevation(
        height.metres()
            + measured_ankle_sole_offset_metres().metres()
            + (ik_tuning().swing_sole_clearance_metres.metres()
                * (1.0 - progress.fraction()))
            .max(ik_tuning().terrain_transition_flight_toe_clearance_metres.metres()),
    )?;
    desired_target.y = desired_target.y.max(
        authored_foot.y.lerp(minimum.metres(), terrain_blend.fraction()),
    );
    if let Some((ankle, toe)) = rendered
        && let Some(floor) = toe_aware_clearance_floor(
            ankle,
            toe,
            point,
            transition_toe_clearance_with_rotation_margin(ankle, toe, elapsed)?,
            |point| terrain.height_at(point),
        )?
    {
        desired_target = floor.apply(desired_target, FootDescent::RaiseOnly)?;
    }
    checked_foot_target(desired_target)
}

pub(in crate::animation::procedural) fn toe_aware_clearance_floor(
    rendered_ankle: Vec3,
    rendered_toe: Vec3,
    desired_ankle: ScenePlanPoint,
    minimum_toe_clearance: SignedLength,
    terrain_height_at: impl Fn(ScenePlanPoint) -> Option<SupportElevation>,
) -> AnimationValueResult<Option<FootClearanceFloor>> {
    let ankle_point = checked_scene_point_metres(rendered_ankle.xz())?;
    let Some(ankle_height) = terrain_height_at(ankle_point) else {
        return Ok(None);
    };
    let ankle_clearance = rendered_ankle.y - ankle_height.metres();
    let toe_point = checked_scene_point_metres(rendered_toe.xz())?;
    let Some(toe_height) = terrain_height_at(toe_point) else {
        return Ok(None);
    };
    let toe_clearance = rendered_toe.y - toe_height.metres();
    let ankle_above_toe = ankle_clearance - toe_clearance;
    let Some(desired_height) = terrain_height_at(desired_ankle) else {
        return Ok(None);
    };
    checked_scene_elevation(desired_height.metres() + ankle_above_toe + minimum_toe_clearance.metres())
        .map(|height| Some(FootClearanceFloor::Minimum(height)))
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
    checked_signed_length(
        ik_tuning()
            .terrain_transition_flight_toe_clearance_metres
            .metres()
            + rendered_ankle.distance(rendered_toe) * angular_step.sin(),
    )
}

/// Combine the same sole-first, optional toe-second transition floor for each
/// candidate of the native sphere/disk search, without recomputing a plan.
pub(in crate::animation::procedural) fn transition_clearance_floor(
    terrain: &SceneTerrain,
    rendered: Option<(Vec3, Vec3)>,
    elapsed: AnimationElapsed,
    point: ScenePlanPoint,
) -> AnimationValueResult<Option<FootClearanceFloor>> {
    let sole = terrain.height_at(point).map(|height| {
        height.metres()
            + measured_ankle_sole_offset_metres().metres()
            + ik_tuning()
                .terrain_transition_flight_toe_clearance_metres
                .metres()
    });
    let toe = rendered
        .map(|(ankle, toe)| -> AnimationValueResult<Option<f32>> {
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
            let ankle_point = checked_scene_point_metres(ankle.xz())?;
            let Some(ankle_support) = terrain.height_at(ankle_point) else {
                return Ok(None);
            };
            let ankle_clearance = ankle.y - ankle_support.metres();
            let toe_point = checked_scene_point_metres(toe.xz())?;
            let Some(toe_support) = terrain.height_at(toe_point) else {
                return Ok(None);
            };
            let toe_clearance = toe.y - toe_support.metres();
            let above_toe = ankle_clearance - toe_clearance;
            Ok(terrain.height_at(point).map(|height| height.metres() + above_toe + margin))
        })
        .transpose()?
        .flatten();
    // Complete the native sole/toe maximum before admitting a retained floor.
    sole.into_iter()
        .chain(toe)
        .reduce(f32::max)
        .map(|height| checked_scene_elevation(height).map(FootClearanceFloor::Minimum))
        .transpose()
}

pub(in crate::animation::procedural) fn terrain_clearance_floor(
    terrain: &SceneTerrain,
    clearance: SignedLength,
    point: ScenePlanPoint,
) -> AnimationValueResult<Option<FootClearanceFloor>> {
    terrain
        .height_at(point)
        .map(|height| {
            checked_scene_elevation(
                height.metres() + measured_ankle_sole_offset_metres().metres() + clearance.metres(),
            )
            .map(FootClearanceFloor::Minimum)
        })
        .transpose()
}

/// Admit completed native scene X/Z metres before application support queries.
pub(in crate::animation::procedural) fn checked_scene_point_metres(
    metres: Vec2,
) -> AnimationValueResult<ScenePlanPoint> {
    ScenePlanPoint::from_metres(metres).ok_or(AnimationValueError::Overflow {
        quantity: AnimationQuantity::WorldPosition,
    })
}

/// Admit a completed absolute ankle elevation in the scene's vertical datum.
pub(in crate::animation::procedural) fn checked_scene_elevation(
    metres: f32,
) -> AnimationValueResult<Elevation<Scene>> {
    Elevation::from_metres(metres).map_err(|_| AnimationValueError::Overflow {
        quantity: AnimationQuantity::WorldPosition,
    })
}

pub(in crate::animation::procedural) fn checked_signed_length(
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
