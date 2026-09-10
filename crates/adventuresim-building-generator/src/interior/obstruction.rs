//! Project only the portion of a rotated architectural cuboid at usable height.
use super::geometry::Rect;
use bevy::math::{Quat, Vec2, Vec3};

pub(super) struct Obstruction {
    corners: [Vec3; 8],
    bottom: f32,
    top: f32,
    footprint: Rect,
}
impl Obstruction {
    pub fn new(centre: Vec3, size: Vec3, yaw: f32, crossfall: f32, longfall: f32) -> Self {
        let rotation = Quat::from_rotation_y(yaw)
            * Quat::from_rotation_x(crossfall)
            * Quat::from_rotation_z(longfall);
        let corners = std::array::from_fn(|index| {
            let signs = Vec3::new(
                if index & 1 == 0 { -1.0 } else { 1.0 },
                if index & 2 == 0 { -1.0 } else { 1.0 },
                if index & 4 == 0 { -1.0 } else { 1.0 },
            );
            centre + rotation * (size * signs * 0.5)
        });
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
        Self {
            corners,
            bottom,
            top,
            footprint: Rect::new((min + max) * 0.5, (max - min) * 0.5),
        }
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
            .iter()
            .copied()
            .filter(|p| p.y >= bottom && p.y <= top)
            .collect::<Vec<_>>();
        for (index, &a) in self.corners.iter().enumerate() {
            for bit in [1, 2, 4] {
                if index & bit != 0 {
                    continue;
                }
                let b = self.corners[index | bit];
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
        Vec3::new(0.0, 2.0, 0.0),
        Vec3::new(4.0, 0.1, 0.1),
        0.0,
        0.0,
        std::f32::consts::FRAC_PI_4,
    );
    let projected = beam.projection(0.15, 1.8).unwrap();
    assert!(projected.contains(Vec2::new(-1.0, 0.0)));
    assert!(!projected.contains(Vec2::new(1.0, 0.0)));
}
