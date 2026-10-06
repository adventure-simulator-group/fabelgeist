//! Conversation positions use the furnished tactical building's circulation proof.
use adventuresim_building_generator::{
    CollisionCuboid, ResolvedItemId,
    interior::{InteriorLayout, StandingClearance, furniture_floor_height},
};
use adventuresim_tactical_core::prelude::GeneratedBuilding;
use bevy::prelude::*;

pub(super) const CONVERSATION_DISTANCE_METRES: f32 = 1.1;
const RESIDENT_SPACING_METRES: f32 = 0.65;

pub(super) fn positions(
    building: &GeneratedBuilding,
    layout: &InteriorLayout,
) -> Result<Vec<Transform>> {
    let field = crate::presentation::interior_lighting::InteriorField::from_plan(
        &building.plan,
        Vec3::ZERO,
    );
    let obstacles = obstacles(building, layout)?;
    let mut candidates = Vec::new();
    let mut clearances = std::collections::BTreeMap::new();
    // Circulation routes share long runs of points. Score each physical point
    // once, preserving first occurrence and the existing daylight ordering.
    let mut visited = std::collections::HashSet::new();
    for path in &layout.paths {
        for point in path
            .points
            .iter()
            .filter(|point| point.storey == adventuresim_building_generator::StoreyIndex::GROUND)
        {
            let centre = point.position_metres.metres();
            if !visited.insert((centre.x.to_bits(), centre.y.to_bits())) {
                continue;
            }
            let height = floor_height(building, centre);
            let clearance = match clearances.entry(height.to_bits()) {
                std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(StandingClearance::new(
                        &obstacles,
                        adventuresim_building_generator::spatial_geometry::Elevation::from_metres(
                            height,
                        )?,
                    )?)
                }
            };
            for direction in [Vec2::X, Vec2::Y, Vec2::NEG_X, Vec2::NEG_Y] {
                let camera = centre + direction * CONVERSATION_DISTANCE_METRES;
                if !clearance.is_clear(centre, camera) {
                    continue;
                }
                let anchor = Vec3::new(centre.x, height, centre.y);
                let daylight = field.directional_daylight_at(
                    anchor + Vec3::Y,
                    Vec3::new(direction.x, 0.0, direction.y),
                );
                candidates.push((daylight, anchor, direction));
            }
        }
    }
    candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
    let transform = building.transform()?;
    let origin = building.collision.bounds.centre()?.metres();
    let mut selected: Vec<Transform> = Vec::new();
    for (_, point, direction) in candidates {
        let anchor = transform.transform_point(point - origin);
        if selected
            .iter()
            .any(|p| p.translation.distance(anchor) < RESIDENT_SPACING_METRES)
        {
            continue;
        }
        selected.push(Transform::from_translation(anchor).with_rotation(
            transform.rotation * Quat::from_rotation_y(direction.x.atan2(direction.y)),
        ));
    }
    Ok(selected)
}

fn floor_height(building: &GeneratedBuilding, point: Vec2) -> f32 {
    building
        .collision
        .cuboids
        .iter()
        .filter(|solid| {
            solid.crossfall_radians.radians() == 0.0 && solid.longfall_radians.radians() == 0.0
        })
        .filter(|solid| {
            let relative = Quat::from_rotation_y(-solid.yaw_radians.radians())
                * (Vec3::new(point.x, solid.centre.metres().y, point.y) - solid.centre.metres());
            relative.x.abs() <= solid.size.metres().x * 0.5
                && relative.z.abs() <= solid.size.metres().z * 0.5
        })
        .map(|solid| solid.centre.metres().y + solid.size.metres().y * 0.5)
        .filter(|height| height.abs() <= 0.3)
        .max_by(f32::total_cmp)
        .unwrap_or(0.0)
}

fn obstacles(
    building: &GeneratedBuilding,
    layout: &InteriorLayout,
) -> Result<Vec<CollisionCuboid<adventuresim_building_generator::spatial_geometry::Architectural>>>
{
    let mut obstacles = building.collision.cuboids.clone();
    for placement in &layout.placements {
        let size = placement.key.interior_spec()?.size_metres.metres();
        obstacles.push(CollisionCuboid::from_metres(
            ResolvedItemId(0),
            Vec3::new(
                placement.centre_metres.metres().x,
                furniture_floor_height(&building.plan, placement)?.metres() + size.y * 0.5,
                placement.centre_metres.metres().y,
            ),
            size,
            placement.yaw_radians(),
            0.0,
            0.0,
        )?);
    }
    Ok(obstacles)
}
