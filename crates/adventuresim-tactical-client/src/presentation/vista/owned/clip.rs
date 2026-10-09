//! Rectangle intersections retain faces instead of dropping crossing triangles.
use super::*;

pub(super) struct PreparedTriangle {
    triangle: [Vec3; 3],
    low: Vec2,
    high: Vec2,
}

impl PreparedTriangle {
    pub(super) fn new(triangle: [Vec3; 3]) -> Self {
        let low = triangle
            .map(|p| p.xz())
            .into_iter()
            .fold(Vec2::splat(f32::INFINITY), Vec2::min);
        let high = triangle
            .map(|p| p.xz())
            .into_iter()
            .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
        Self {
            triangle,
            low,
            high,
        }
    }

    pub(super) fn in_rectangle(&self, minimum: Vec2, maximum: Vec2) -> Vec<[Vec3; 3]> {
        let Self {
            triangle,
            low,
            high,
        } = *self;
        if low.cmpge(maximum).any() || high.cmplt(minimum).any() {
            return Vec::new();
        }
        if low.cmpge(minimum).all() && high.cmple(maximum).all() {
            return vec![triangle];
        }
        let [a, b, c] = triangle.map(Vec3::as_dvec3);
        let normal = (b - a).cross(c - a);
        let represented = |point: DVec3| {
            let mut point = point.as_vec3();
            if normal.y.abs() > f64::EPSILON {
                // Height is evaluated at represented X/Z coordinates, avoiding a
                // false steep normal in tiny pieces at large geographic coordinates.
                point.y = (a.y
                    - (normal.x * (f64::from(point.x) - a.x)
                        + normal.z * (f64::from(point.z) - a.z))
                        / normal.y) as f32;
            }
            point
        };
        let mut polygon = vec![a, b, c];
        for (axis, bound, direction) in [
            (0, minimum.x, 1.0),
            (0, maximum.x, -1.0),
            (2, minimum.y, 1.0),
            (2, maximum.y, -1.0),
        ] {
            let distance = |point: DVec3| (point[axis] - f64::from(bound)) * direction;
            let mut clipped = Vec::new();
            for i in 0..polygon.len() {
                let a = polygon[i];
                let b = polygon[(i + 1) % polygon.len()];
                let da = distance(a);
                let db = distance(b);
                if da >= 0.0 {
                    clipped.push(a);
                }
                if (da >= 0.0) != (db >= 0.0) {
                    clipped.push(a.lerp(b, da / (da - db)));
                }
            }
            polygon = clipped;
            if polygon.is_empty() {
                return Vec::new();
            }
        }
        let polygon = polygon.into_iter().map(represented).collect::<Vec<_>>();
        (1..polygon.len().saturating_sub(1))
            .filter_map(|i| {
                let triangle = [polygon[0], polygon[i], polygon[i + 1]];
                ((triangle[1] - triangle[0])
                    .cross(triangle[2] - triangle[0])
                    .length_squared()
                    > f32::EPSILON)
                    .then_some(triangle)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crossing_triangle_has_complete_coverage_and_preserves_its_plane() {
        let vertex = |x, z| Vec3::new(x, 20.0 + x * 0.03 + z * 0.04, z);
        let triangles = [
            [
                vertex(-10.0, -10.0),
                vertex(-10.0, 10.0),
                vertex(10.0, -10.0),
            ],
            [vertex(10.0, -10.0), vertex(-10.0, 10.0), vertex(10.0, 10.0)],
        ];
        let pieces = triangles
            .into_iter()
            .flat_map(|t| {
                PreparedTriangle::new(t).in_rectangle(Vec2::splat(-1.0), Vec2::splat(1.0))
            })
            .collect::<Vec<_>>();
        let area: f32 = pieces
            .iter()
            .map(|t| {
                (t[1].xz() - t[0].xz())
                    .perp_dot(t[2].xz() - t[0].xz())
                    .abs()
                    * 0.5
            })
            .sum();
        assert!((area - 4.0).abs() < 0.00001);
        for p in pieces.into_iter().flatten() {
            assert!((p.y - vertex(p.x, p.z).y).abs() < 0.00001);
        }
    }
    #[test]
    fn retaining_face_is_clipped_without_disappearing() {
        let triangle = [
            Vec3::new(-2.0, -1.0, 0.0),
            Vec3::new(2.0, -1.0, 0.0),
            Vec3::new(2.0, 1.0, 0.0),
        ];
        let pieces =
            PreparedTriangle::new(triangle).in_rectangle(Vec2::splat(-0.5), Vec2::splat(0.5));
        assert!(!pieces.is_empty());
        assert!(
            pieces
                .into_iter()
                .flatten()
                .all(|p| p.x.abs() <= 0.5 && p.z == 0.0)
        );
    }
}
