//! Conversation positions use the furnished tactical building's circulation proof.
use adventuresim_building_generator::{
    CollisionCuboid, ResolvedItemId,
    interior::{InteriorLayout, StandingClearance, furniture_floor_height},
};
use adventuresim_tactical_core::prelude::GeneratedBuilding;
use bevy::prelude::*;

pub(super) const CONVERSATION_DISTANCE_METRES: f32 = 1.1;
const RESIDENT_SPACING_METRES: f32 = 0.65;

pub(super) fn positions(building: &GeneratedBuilding, layout: &InteriorLayout) -> Vec<Transform> {
    let field = crate::presentation::interior_lighting::InteriorField::from_plan(
        &building.plan,
        Vec3::ZERO,
    );
    let obstacles = obstacles(building, layout);
    let mut candidates = Vec::new();
    let mut clearances = std::collections::BTreeMap::new();
    // Circulation routes share long runs of points. Score each physical point
    // once, preserving first occurrence and the existing daylight ordering.
    let mut visited = std::collections::HashSet::new();
    for path in &layout.paths {
        for point in path.points.iter().filter(|point| point.storey == 0) {
            let centre = point.position_metres;
            if !visited.insert((centre.x.to_bits(), centre.y.to_bits())) {
                continue;
            }
            let height = floor_height(building, centre);
            let clearance = clearances
                .entry(height.to_bits())
                .or_insert_with(|| StandingClearance::new(&obstacles, height));
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
    let transform = super::buildings::transform(building);
    let origin = building.collision.bounds.centre();
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
    selected
}

fn floor_height(building: &GeneratedBuilding, point: Vec2) -> f32 {
    building
        .collision
        .cuboids
        .iter()
        .filter(|solid| solid.crossfall_radians == 0.0 && solid.longfall_radians == 0.0)
        .filter(|solid| {
            let relative = Quat::from_rotation_y(-solid.yaw_radians)
                * (Vec3::new(point.x, solid.centre.y, point.y) - solid.centre);
            relative.x.abs() <= solid.size.x * 0.5 && relative.z.abs() <= solid.size.z * 0.5
        })
        .map(|solid| solid.centre.y + solid.size.y * 0.5)
        .filter(|height| height.abs() <= 0.3)
        .max_by(f32::total_cmp)
        .unwrap_or(0.0)
}

fn obstacles(building: &GeneratedBuilding, layout: &InteriorLayout) -> Vec<CollisionCuboid> {
    let mut obstacles = building.collision.cuboids.clone();
    for placement in &layout.placements {
        let size = placement
            .key
            .interior_spec()
            .expect("furnished interior")
            .size_metres;
        obstacles.push(CollisionCuboid {
            source: ResolvedItemId(0),
            centre: Vec3::new(
                placement.centre_metres.x,
                furniture_floor_height(&building.plan, placement) + size.y * 0.5,
                placement.centre_metres.y,
            ),
            size,
            yaw_radians: placement.yaw_radians(),
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
        });
    }
    obstacles
}
