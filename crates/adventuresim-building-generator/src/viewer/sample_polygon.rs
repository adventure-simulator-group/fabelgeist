//! Projected section footprint used by screenshot photometry.
use super::*;

#[derive(Resource)]
pub(super) struct SampleBounds {
    pub(super) min: Vec3,
    pub(super) max: Vec3,
}

/// Measure the named surfaces, excluding unrelated ground and sky.
pub(super) fn prepare_sample(
    bounds: Option<Res<SampleBounds>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut state: ResMut<CaptureState>,
) {
    let Some(bounds) = bounds else {
        return;
    };
    let Some((camera, transform)) = cameras.iter().find(|(camera, _)| camera.is_active) else {
        return;
    };
    let mut points = Vec::new();
    for x in [bounds.min.x, bounds.max.x] {
        for y in [bounds.min.y, bounds.max.y] {
            for z in [bounds.min.z, bounds.max.z] {
                let Ok(pixel) = camera.world_to_viewport(transform, Vec3::new(x, y, z)) else {
                    return;
                };
                let fraction = pixel / Vec2::new(VIEW_WIDTH as f32, VIEW_HEIGHT as f32);
                points.push(fraction);
            }
        }
    }
    state.manifest.surface_sample_polygon =
        Some(sample_polygon::SamplePolygon::from_points(points));
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct SamplePolygon(Vec<[f32; 2]>);

impl SamplePolygon {
    /// Convex silhouette of the projected section envelope, in viewport fractions.
    pub(super) fn from_points(mut points: Vec<Vec2>) -> Self {
        points.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
        points.dedup();
        let mut hull = Vec::new();
        for point in &points {
            append_corner(&mut hull, *point);
        }
        let lower_len = hull.len();
        for point in points.iter().rev().skip(1) {
            while hull.len() > lower_len
                && turn(hull[hull.len() - 2], hull[hull.len() - 1], *point) <= 0.0
            {
                hull.pop();
            }
            hull.push(*point);
        }
        hull.pop();
        Self(hull.iter().map(Vec2::to_array).collect())
    }

    pub(super) fn contains(&self, point: Vec2) -> bool {
        self.0.len() >= 3
            && self.0.iter().enumerate().all(|(index, a)| {
                let b = self.0[(index + 1) % self.0.len()];
                turn(Vec2::from_array(*a), Vec2::from_array(b), point) >= 0.0
            })
    }
}

fn turn(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b - a).perp_dot(c - a)
}

fn append_corner(hull: &mut Vec<Vec2>, point: Vec2) {
    while hull.len() >= 2 && turn(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0 {
        hull.pop();
    }
    hull.push(point);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotated_section_excludes_background_corners_and_retains_interior() {
        let polygon = SamplePolygon::from_points(vec![
            Vec2::new(0.5, 0.1),
            Vec2::new(0.9, 0.5),
            Vec2::new(0.5, 0.9),
            Vec2::new(0.1, 0.5),
            Vec2::splat(0.5),
            Vec2::splat(0.5),
        ]);
        assert!(polygon.contains(Vec2::splat(0.5)));
        assert!(polygon.contains(Vec2::new(0.5, 0.15)));
        assert!(!polygon.contains(Vec2::splat(0.15)));
        assert!(!polygon.contains(Vec2::splat(0.85)));
        assert_eq!(polygon.0.len(), 4);
    }
}
