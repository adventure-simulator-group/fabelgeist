//! Project only the portion of a rotated architectural cuboid at usable height.
use super::geometry::Rect;
use bevy::math::{Vec2, Vec3};

pub(super) struct Obstruction<F: crate::spatial_geometry::GeometryFrame> {
    corners: crate::CuboidCorners<F>,
    bottom: f32,
    top: f32,
    footprint: Rect,
}
impl<F: crate::spatial_geometry::GeometryFrame> Obstruction<F> {
    pub fn new(solid: crate::CollisionCuboid<F>) -> Result<Self, crate::CollisionError> {
        let topology = solid.corners()?;
        // Height clipping is a private native arithmetic kernel. Admission and
        // traversal retain the input's frame and physical source identity.
        let corners = topology.points().map(|point| point.metres());
        let bounds = crate::SpatialBounds::<F>::from_metres(
            corners
                .into_iter()
                .fold(Vec3::splat(f32::INFINITY), Vec3::min),
            corners
                .into_iter()
                .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max),
        )
        .map_err(|cause| crate::CollisionError {
            source_id: solid.source,
            cause,
        })?;
        bounds
            .centre()
            .and_then(|_| bounds.extent())
            .map_err(|cause| crate::CollisionError {
                source_id: solid.source,
                cause,
            })?;
        let bottom = corners.iter().map(|c| c.y).fold(f32::INFINITY, f32::min);
        let top = corners
            .iter()
            .map(|c| c.y)
            .fold(f32::NEG_INFINITY, f32::max);
        let min = corners
            .iter()
            .map(|p| Vec2::new(p.x, p.z))
            .fold(Vec2::splat(f32::INFINITY), Vec2::min);
        let max = corners
            .iter()
            .map(|p| Vec2::new(p.x, p.z))
            .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
        Ok(Self {
            corners: topology,
            bottom,
            top,
            footprint: Rect::new((min + max) * 0.5, (max - min) * 0.5),
        })
    }
    pub fn intersects(&self, rect: Rect, bottom: f32, top: f32) -> bool {
        self.footprint.overlaps(rect)
            && self
                .projection(bottom, top)
                .is_some_and(|r| r.overlaps(rect))
    }
    pub fn projection(&self, bottom: f32, top: f32) -> Option<Rect> {
        if self.top <= bottom || self.bottom >= top {
            return None;
        }
        let mut points = self
            .corners
            .points()
            .iter()
            .map(|point| point.metres())
            .filter(|p| p.y >= bottom && p.y <= top)
            .collect::<Vec<_>>();
        for edge in self.corners.edges() {
            let [a, b] = [edge.start.metres(), edge.end.metres()];
            if (a.y - b.y).abs() < f32::EPSILON {
                continue;
            }
            for height in [bottom, top] {
                let t = (height - a.y) / (b.y - a.y);
                if (0.0..=1.0).contains(&t) {
                    points.push(a + (b - a) * t);
                }
            }
        }
        let min = points
            .iter()
            .map(|p| Vec2::new(p.x, p.z))
            .fold(Vec2::splat(f32::INFINITY), Vec2::min);
        let max = points
            .iter()
            .map(|p| Vec2::new(p.x, p.z))
            .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
        (!points.is_empty()).then_some(Rect::new((min + max) * 0.5, (max - min) * 0.5))
    }
}

#[test]
fn tilted_beam_only_blocks_where_it_intersects_head_height() {
    let beam = Obstruction::new(
        crate::CollisionCuboid::<crate::Architectural>::from_metres(
            crate::ResolvedItemId(7),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(4.0, 0.1, 0.1),
            0.0,
            0.0,
            std::f32::consts::FRAC_PI_4,
        )
        .unwrap(),
    )
    .unwrap();
    let projected = beam.projection(0.15, 1.8).unwrap();
    assert!(projected.contains(Vec2::new(-1.0, 0.0)));
    assert!(!projected.contains(Vec2::new(1.0, 0.0)));
}
