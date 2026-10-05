//! Stair contacts use the bearing face, not a convex corner separation normal.
use super::*;

pub(super) fn has_walkable_support(
    hit: MoveHitData,
    horizontal_travel: Vec3,
    query: &MoveAndSlide,
    config: &CharacterController,
) -> bool {
    if hit.normal1.y >= config.min_walk_cos {
        return true;
    }
    // A source-clipped convex cell can return a corner normal even when the
    // next tread is level. Confirm the physical face immediately inside the
    // same contact envelope, on the same body and at the same bearing height.
    // Clearance casts and the actual character candidate remain unchanged.
    let origin = hit.point1
        + Vec3::Y * config.step_size
        + horizontal_travel * config.move_and_slide.skin_width;
    query
        .spatial_query
        .cast_ray_predicate(
            origin,
            Dir3::NEG_Y,
            config.step_size + config.move_and_slide.skin_width,
            true,
            &config.filter,
            &|entity| entity == hit.entity && query.colliders.contains(entity),
        )
        .is_some_and(|face| {
            face.normal.y >= config.min_walk_cos
                && (config.step_size - face.distance).abs() <= config.move_and_slide.skin_width
        })
}
