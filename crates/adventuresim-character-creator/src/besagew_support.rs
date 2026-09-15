//! Support depth over the disc's actual convex polygon, in its local frame.
use bevy::math::{Vec2, Vec3};

/// Clip complete support triangles, so coarse tessellation and vertices beyond
/// the footprint cannot respectively miss support or push the disc outward.
pub(super) fn depth(triangles: impl Iterator<Item = [Vec3; 3]>, outline: &[Vec2]) -> Option<f32> {
    triangles
        .filter_map(|triangle| {
            let mut polygon = triangle.to_vec();
            for i in 0..outline.len() {
                let a = outline[i];
                let edge = outline[(i + 1) % outline.len()] - a;
                let distance = |p: Vec3| edge.perp_dot(p.truncate() - a);
                let mut clipped = Vec::new();
                for j in 0..polygon.len() {
                    let p = polygon[j];
                    let q = polygon[(j + 1) % polygon.len()];
                    let dp = distance(p);
                    let dq = distance(q);
                    if dp >= 0.0 {
                        clipped.push(p);
                    }
                    if (dp >= 0.0) != (dq >= 0.0) {
                        clipped.push(p.lerp(q, dp / (dp - dq)));
                    }
                }
                polygon = clipped;
                if polygon.is_empty() {
                    return None;
                }
            }
            polygon.iter().map(|p| p.z).reduce(f32::max)
        })
        .reduce(f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clips_support_to_footprint_without_requiring_a_vertex_inside() {
        for radius in [0.035, 0.058, 0.085] {
            for columns in [16, 32, 96] {
                let outline = (0..columns)
                    .map(|i| {
                        Vec2::from_angle(std::f32::consts::TAU * i as f32 / columns as f32) * radius
                    })
                    .collect::<Vec<_>>();
                let enclosing = [
                    Vec3::new(-1.0, -1.0, 0.2),
                    Vec3::new(1.0, -1.0, 0.2),
                    Vec3::new(0.0, 1.0, 0.2),
                ];
                let outside = [
                    Vec3::new(radius * 1.1, 0.0, 0.9),
                    Vec3::new(radius * 1.4, 0.0, 0.9),
                    Vec3::new(radius * 1.1, radius, 0.9),
                ];
                assert!(
                    (depth([enclosing, outside].into_iter(), &outline).unwrap() - 0.2).abs() < 1e-6
                );
                assert!(depth([outside].into_iter(), &outline).is_none());
                let slope = enclosing.map(|p| Vec3::new(p.x, p.y, p.x + 0.3));
                assert!(
                    (depth([slope].into_iter(), &outline).unwrap() - (radius + 0.3)).abs() < 1e-6
                );
            }
        }
    }
}
