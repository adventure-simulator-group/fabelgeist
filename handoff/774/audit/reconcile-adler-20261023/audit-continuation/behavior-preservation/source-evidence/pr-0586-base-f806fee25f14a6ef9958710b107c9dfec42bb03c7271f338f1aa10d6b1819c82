//! Surfaces of revolution for turned wood, cast metal, and hollow vessels.
use bevy::math::{Vec2, Vec3};
use std::f32::consts::TAU;

use super::builder::Builder;
use crate::BuildingLodMaterial;

const SIDES: usize = 12;

impl Builder {
    /// A profile traverses the entire section, including its inner return for a hollow vessel.
    /// End caps close nonzero terminal radii. Collision is authored by the owning recipe.
    pub(crate) fn turned(
        &mut self,
        material: BuildingLodMaterial,
        origin: Vec3,
        profile: &[(f32, f32)],
    ) {
        for side in 0..SIDES {
            let directions = [side, side + 1].map(|i| {
                let angle = TAU * i as f32 / SIDES as f32;
                Vec3::new(angle.sin(), 0.0, angle.cos())
            });
            for pair in profile.windows(2) {
                let points = [
                    origin + Vec3::Y * pair[0].0 + directions[0] * pair[0].1,
                    origin + Vec3::Y * pair[0].0 + directions[1] * pair[0].1,
                    origin + Vec3::Y * pair[1].0 + directions[1] * pair[1].1,
                    origin + Vec3::Y * pair[1].0 + directions[0] * pair[1].1,
                ];
                let normal = (points[1] - points[0])
                    .cross(points[3] - points[0])
                    .normalize();
                self.quad(material, points, normal);
            }
            for (index, normal) in [(0, -Vec3::Y), (profile.len() - 1, Vec3::Y)] {
                let (height, radius) = profile[index];
                let centre = origin + Vec3::Y * height;
                let points = [
                    centre,
                    centre + directions[0] * radius,
                    centre + directions[1] * radius,
                ];
                self.mesh(material).push_triangle(
                    points,
                    normal,
                    points.map(|p| Vec2::new(p.x, p.z)),
                );
            }
        }
    }
}
