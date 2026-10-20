//! Joint native clearance and owner-motion budget projection for a run foot.

use super::*;

pub(in crate::animation::procedural) fn advance_run_airborne_world_target(
    previous_owner: Option<Vec3>,
    desired_world: Vec3,
    rig_origin: Vec3,
    rig_rotation: Quat,
    delta_seconds: AnimationElapsed,
    speed: NonnegativeSpeed,
    clearance_floor_at: impl Fn(ScenePlanPoint) -> AnimationValueResult<Option<FootClearanceFloor>>,
) -> AnimationValueResult<Vec3> {
    let Some(previous_owner) = previous_owner.filter(|target| target.is_finite()) else {
        let mut target = desired_world;
        if let Some(floor) = clearance_floor_at(checked_scene_point_metres(target.xz())?)? {
            target.y = target.y.max(floor.minimum_height()?.metres());
        }
        return checked_foot_target(target);
    };
    let start_world = rig_origin + rig_rotation * previous_owner;
    let maximum_step = speed.metres_per_second() * delta_seconds.seconds();
    let cleared_at = |progress: f32| {
        let mut target = start_world.lerp(desired_world, progress.clamp(0.0, 1.0));
        if let Some(floor) = clearance_floor_at(checked_scene_point_metres(target.xz())?)? {
            target.y = target.y.max(floor.minimum_height()?.metres());
        }
        checked_foot_target(target)
    };
    let desired = cleared_at(1.0)?;
    if desired.distance(start_world) <= maximum_step {
        return Ok(desired);
    }

    // Clearance and the 3D budget are solved jointly. On an uphill release,
    // both full owner transport (terrain rise) and literal world hold (root
    // displacement) can be outside the sphere while an intermediate XZ is
    // feasible. That set is not monotone from either endpoint, so first scan
    // deterministically for its farthest feasible interval, then refine only
    // the local exit boundary.
    const SEARCH_SAMPLES: usize = 64;
    let mut best = None;
    let mut best_progress = 0.0;
    for index in 0..=SEARCH_SAMPLES {
        let progress = index as f32 / SEARCH_SAMPLES as f32;
        let candidate = cleared_at(progress)?;
        if candidate.distance(start_world) <= maximum_step + 0.000001 {
            best = Some(candidate);
            best_progress = progress;
        }
    }
    let Some(mut best) = best else {
        // The straight owner-transport-to-world-plant segment can cross a
        // locally high triangle even when a small lateral/fore-aft detour is
        // inside the same 3D motion sphere. Search that complete horizontal
        // disk before accepting an over-budget fallback. This matters at
        // toe-off: the semantic sole floor and a terrain rise can make both
        // line endpoints invalid while a nearby downhill point remains valid.
        if let Some(feasible) = terrain_feasible_target_in_step(
            start_world,
            desired_world,
            maximum_step,
            &clearance_floor_at,
        )? {
            return Ok(feasible);
        }
        // A true vertical discontinuity can leave no valid point in the full
        // disk. Keep clearance truthful and choose the least-discontinuous
        // line sample; support/reach ownership handles that rare fallback.
        let fallback = (0..=SEARCH_SAMPLES)
            .map(|index| cleared_at(index as f32 / SEARCH_SAMPLES as f32))
            .collect::<AnimationValueResult<Vec<_>>>()?
            .into_iter()
            .min_by(|left, right| {
                left.distance_squared(start_world)
                    .total_cmp(&right.distance_squared(start_world))
            });
        return match fallback {
            Some(target) => Ok(target),
            None => cleared_at(0.0),
        };
    };
    let mut low = best_progress;
    let mut high = (best_progress + 1.0 / SEARCH_SAMPLES as f32).min(1.0);
    for _ in 0..12 {
        let middle = (low + high) * 0.5;
        let candidate = cleared_at(middle)?;
        if candidate.distance(start_world) <= maximum_step + 0.000001 {
            low = middle;
            best = candidate;
        } else {
            high = middle;
        }
    }
    Ok(best)
}

pub(in crate::animation::procedural) fn terrain_feasible_target_in_step(
    start_world: Vec3,
    desired_world: Vec3,
    maximum_step: f32,
    clearance_floor_at: &impl Fn(ScenePlanPoint) -> AnimationValueResult<Option<FootClearanceFloor>>,
) -> AnimationValueResult<Option<Vec3>> {
    if maximum_step <= 0.0 {
        return Ok(None);
    }
    let mut best: Option<(f32, Vec3)> = None;
    let mut center = start_world.xz();
    let mut radius = maximum_step;
    // A deterministic square-disk search followed by local refinements finds
    // terrain-feasible points off the direct swing chord without introducing
    // frame-rate-dependent iteration or lateral drift.
    for refinement in 0..4 {
        const HALF_GRID: i32 = 12;
        let spacing = radius / HALF_GRID as f32;
        for x in -HALF_GRID..=HALF_GRID {
            for z in -HALF_GRID..=HALF_GRID {
                let xz = center + Vec2::new(x as f32 * spacing, z as f32 * spacing);
                let offset = xz - start_world.xz();
                if offset.length_squared() > maximum_step * maximum_step + 0.000001 {
                    continue;
                }
                let chord = desired_world.xz() - start_world.xz();
                let chord_progress = if chord.length_squared() > 0.0 {
                    offset.dot(chord) / chord.length_squared()
                } else {
                    0.0
                }
                .clamp(0.0, 1.0);
                let nearest_chord = start_world.xz() + chord * chord_progress;
                if xz.distance(nearest_chord) > 0.04 {
                    continue;
                }
                let Some(floor) = clearance_floor_at(checked_scene_point_metres(xz)?)? else {
                    return Ok(None);
                };
                let candidate = Vec3::new(xz.x, desired_world.y.max(floor.minimum_height()?.metres()), xz.y);
                if candidate.distance(start_world) > maximum_step + 0.000001 {
                    continue;
                }
                let score = candidate.distance_squared(desired_world);
                if best.is_none_or(|(best_score, _)| score < best_score) {
                    best = Some((score, candidate));
                }
            }
        }
        let Some((_, candidate)) = best else {
            break;
        };
        if refinement < 3 {
            center = candidate.xz();
            radius = spacing * 2.0;
        }
    }
    Ok(best.map(|(_, candidate)| candidate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captured_run_swing_step_keeps_target_inside_knee_budget_margin() {
        let previous_root = Vec3::new(0.0, 3.0811288, -4.46875);
        let next_root = Vec3::new(0.0, 3.0736096, -4.5546875);
        let previous_target = Vec3::new(-0.11504456, 2.3028326, -3.8614511);
        let desired_target = Vec3::new(-0.1152586, 2.310206, -4.0351343);
        let previous_owner = previous_target - previous_root;
        let advanced = advance_run_airborne_world_target(
            Some(previous_owner),
            desired_target,
            next_root,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                ik_tuning().run_airborne_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz(),
            )
            .unwrap(),
            |_| Ok(None),
        )
        .unwrap();
        let target_step = (advanced - next_root).distance(previous_owner);
        assert!(target_step <= 0.0875 + 0.0001);
        assert!(target_step > 5.5 / ik_tuning().continuity_sample_hz.hertz());
        assert!(target_step < 0.089);
    }

    #[test]
    fn first_run_release_searches_off_chord_for_terrain_clearance() {
        let start = Vec3::ZERO;
        let desired = Vec3::new(0.0, 0.0, 0.08);
        let maximum_step = 0.094;
        let minimum_y = |xz: Vec2| {
            // The direct chord is a raised ridge; a lateral point within the
            // same motion sphere satisfies both clearance and continuity.
            Some(if xz.x.abs() < 0.02 { 0.12 } else { 0.02 })
        };
        let target = advance_run_airborne_world_target(
            Some(start),
            desired,
            Vec3::ZERO,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(maximum_step)
                .unwrap(),
            |point| {
                minimum_y(point.metres())
                    .map(|height| checked_scene_elevation(height).map(FootClearanceFloor::Minimum))
                    .transpose()
            },
        )
        .unwrap();
        assert!(target.x.abs() >= 0.02);
        assert!(target.y + 0.0001 >= minimum_y(target.xz()).unwrap());
        assert!(target.distance(start) <= maximum_step + 0.0001);
    }

    #[test]
    fn uphill_airborne_projection_preserves_clearance_and_step_budget() {
        let previous_owner = Vec3::new(0.0, 0.15, 0.0);
        let desired = Vec3::new(0.0, 0.2, -0.3);
        let minimum_y = |xz: Vec2| Some(0.15 + (-xz.y).max(0.0) * 0.4);
        let resolved = advance_run_airborne_world_target(
            Some(previous_owner),
            desired,
            Vec3::ZERO,
            Quat::IDENTITY,
            AnimationElapsed::from_seconds(1.0 / ik_tuning().continuity_sample_hz.hertz()).unwrap(),
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                ik_tuning().run_airborne_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz(),
            )
            .unwrap(),
            |point| {
                minimum_y(point.metres())
                    .map(|height| checked_scene_elevation(height).map(FootClearanceFloor::Minimum))
                    .transpose()
            },
        )
        .unwrap();
        assert!(resolved.is_finite());
        assert!(resolved.y + 0.000001 >= minimum_y(resolved.xz()).unwrap());
        assert!(
            resolved.distance(previous_owner)
                <= (ik_tuning().run_airborne_owner_step_metres.metres()
                    * ik_tuning().continuity_sample_hz.hertz())
                    / ik_tuning().continuity_sample_hz.hertz()
                    + 0.0001
        );
    }
}
