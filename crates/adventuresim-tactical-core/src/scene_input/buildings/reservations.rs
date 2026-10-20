//! Horizontal obstacle exclusions retain their scene origin and collision extents.

use super::BuildingOrientation;
use crate::scene_coordinates::ScenePlanPoint;
use crate::scene_input::GeneratedObstacle;
use adventuresim_building_generator::spatial_geometry::{GeometryResult, PlanExtents};
use bevy::math::Vec2;

const OBSTACLE_APRON_METRES: f32 = 5.5;

/// Collision reservations permit degenerate half extents, unlike occupied plots.
/// Their apron excludes obstacles without selecting a terrain or building floor.
#[derive(Clone, Copy, Debug)]
pub(in crate::scene_input) struct ObstacleReservation {
    pub(in crate::scene_input) centre: ScenePlanPoint,
    pub(in crate::scene_input) half_extents: PlanExtents,
    pub(in crate::scene_input) orientation: BuildingOrientation,
}

impl ObstacleReservation {
    fn contains_apron(self, point: ScenePlanPoint) -> bool {
        let offset = self.orientation.world_to_local(point.metres() - self.centre.metres()).abs();
        offset
            .cmple(self.half_extents.metres() + Vec2::splat(OBSTACLE_APRON_METRES))
            .all()
    }
}

pub(in crate::scene_input) fn obstacle_intersects_building(
    obstacle: GeneratedObstacle,
    obstacle_spacing: f32,
    terrain_extent: Vec2,
    reservations: &[ObstacleReservation],
) -> GeometryResult<bool> {
    let (x, z) = match obstacle {
        GeneratedObstacle::Tree { x, z } | GeneratedObstacle::Rock { x, z, .. } => (x, z),
    };
    // Decode the grid with the original multiply-then-subtract order.
    let point = ScenePlanPoint::try_from(
        Vec2::new(f32::from(x), f32::from(z)) * obstacle_spacing - terrain_extent * 0.5,
    )?;
    Ok(reservations.iter().any(|reservation| reservation.contains_apron(point)))
}
