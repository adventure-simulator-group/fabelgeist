//! Exact extrema of playable terrain triangles clipped to a rotated footprint.
use adventuresim_tactical_core::prelude::SceneTerrain;
use bevy::math::Vec2;
use serde_json::{Value, json};
#[path = "footprint/vista.rs"]
mod vista;
pub(super) use vista::{
    extrema as vista_extrema, height_in_triangle, triangles as vista_triangles,
};

pub(super) fn terrain_extrema(terrain: &SceneTerrain, corners: [Vec2; 4]) -> Value {
    let half = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
    let minimum = corners
        .into_iter()
        .fold(Vec2::splat(f32::INFINITY), Vec2::min);
    let maximum = corners
        .into_iter()
        .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
    let grid_min = ((minimum + half) / terrain.grid_scale())
        .floor()
        .max(Vec2::ZERO);
    let grid_max = ((maximum + half) / terrain.grid_scale())
        .ceil()
        .min(Vec2::new(
            (terrain.grid_width() - 1) as f32,
            (terrain.grid_depth() - 1) as f32,
        ));
    let mut vertices = 0;
    let mut low = f32::INFINITY;
    let mut high = f32::NEG_INFINITY;
    let mut low_point = None;
    let mut high_point = None;
    for z in grid_min.y as usize..grid_max.y as usize {
        for x in grid_min.x as usize..grid_max.x as usize {
            let a = Vec2::new(x as f32, z as f32) * terrain.grid_scale() - half;
            let b = a + Vec2::X * terrain.grid_scale();
            let c = a + Vec2::ONE * terrain.grid_scale();
            let d = a + Vec2::Y * terrain.grid_scale();
            // SceneTerrain and Avian subdivide across b--d, not a--c.
            for triangle in [[a, b, d], [b, c, d]] {
                for point in clip(triangle.to_vec(), &corners) {
                    let Some(height) = terrain.height_at(point) else {
                        continue;
                    };
                    vertices += 1;
                    if height < low {
                        low = height;
                        low_point = Some(point);
                    }
                    if height > high {
                        high = height;
                        high_point = Some(point);
                    }
                }
            }
        }
    }
    json!({
        "complete_playable_coverage":corners.iter().all(|p| p.abs().cmple(half).all()),
        "intersection_vertices":vertices,
        "minimum_metres":low_point.map(|_|low), "minimum_location":low_point,
        "maximum_metres":high_point.map(|_|high), "maximum_location":high_point,
        "method":"Linear terrain-triangle extrema at clipped intersection vertices, including interior terrain vertices; outside-playable coverage is not certified.",
    })
}

pub(super) fn clip(mut polygon: Vec<Vec2>, corners: &[Vec2]) -> Vec<Vec2> {
    for index in 0..corners.len() {
        let a = corners[index];
        let edge = corners[(index + 1) % corners.len()] - a;
        let input = std::mem::take(&mut polygon);
        let Some(mut previous) = input.last().copied() else {
            break;
        };
        let mut previous_side = edge.perp_dot(previous - a);
        for point in input {
            let side = edge.perp_dot(point - a);
            if (side >= 0.0) != (previous_side >= 0.0) {
                polygon.push(previous.lerp(point, previous_side / (previous_side - side)));
            }
            if side >= 0.0 {
                polygon.push(point);
            }
            previous = point;
            previous_side = side;
        }
    }
    polygon
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotated_footprint_detects_an_interior_terrain_peak_absent_from_corners() {
        let mut heights = vec![0.0; 9 * 9];
        heights[4 * 9 + 4] = 2.0;
        let terrain = SceneTerrain::from_heightmap(9, 9, 1.0, heights).unwrap();
        let corners = [
            Vec2::new(0.0, -3.0),
            Vec2::new(3.0, 0.0),
            Vec2::new(0.0, 3.0),
            Vec2::new(-3.0, 0.0),
        ];
        assert!(corners.iter().all(|p| terrain.height_at(*p) == Some(0.0)));
        let report = terrain_extrema(&terrain, corners);
        assert_eq!(report["maximum_metres"], 2.0);
        assert_eq!(report["minimum_metres"], 0.0);
        assert_eq!(report["complete_playable_coverage"], true);
    }

    #[test]
    fn intersections_follow_the_collision_heightfield_diagonal() {
        let terrain = SceneTerrain::from_heightmap(2, 2, 1.0, vec![0.0, 2.0, 2.0, 0.0]).unwrap();
        let corners = [
            Vec2::new(-0.1, -0.1),
            Vec2::new(0.1, -0.1),
            Vec2::new(0.1, 0.1),
            Vec2::new(-0.1, 0.1),
        ];
        let report = terrain_extrema(&terrain, corners);
        assert_eq!(report["maximum_metres"], 2.0);
        assert!((report["minimum_metres"].as_f64().unwrap() - 1.6).abs() < 0.001);
    }
}
