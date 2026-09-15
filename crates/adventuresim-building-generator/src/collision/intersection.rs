use super::CollisionCuboid;
use bevy::math::{Quat, Vec3};

impl CollisionCuboid {
    /// Positive-volume intersection of the actual oriented cuboids, including
    /// pitched beams. Coplanar contact alone does not count as penetration.
    pub fn intersects(self, other: Self) -> bool {
        let axes = |solid: Self| {
            let rotation = Quat::from_rotation_y(solid.yaw_radians)
                * Quat::from_rotation_x(solid.crossfall_radians)
                * Quat::from_rotation_z(solid.longfall_radians);
            [rotation * Vec3::X, rotation * Vec3::Y, rotation * Vec3::Z]
        };
        let left = axes(self);
        let right = axes(other);
        let delta = other.centre - self.centre;
        let radius = |axis: Vec3, basis: [Vec3; 3], size: Vec3| {
            (0..3)
                .map(|i| axis.dot(basis[i]).abs() * size[i] * 0.5)
                .sum::<f32>()
        };
        left.into_iter()
            .chain(right)
            .chain(left.into_iter().flat_map(|a| right.map(|b| a.cross(b))))
            .filter(|axis| axis.length_squared() > f32::EPSILON)
            .all(|axis| {
                delta.dot(axis).abs()
                    < radius(axis, left, self.size) + radius(axis, right, other.size)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagonal_solids_require_oriented_intersection() {
        let beam = CollisionCuboid {
            source: crate::ResolvedItemId(1),
            centre: Vec3::ZERO,
            size: Vec3::new(4.0, 0.1, 0.1),
            yaw_radians: 0.7,
            crossfall_radians: 0.0,
            longfall_radians: 0.3,
        };
        let rotation = Quat::from_rotation_y(0.7) * Quat::from_rotation_z(0.3);
        assert!(!beam.intersects(CollisionCuboid {
            centre: rotation * Vec3::Z * 0.2,
            ..beam
        }));
        assert!(beam.intersects(CollisionCuboid {
            centre: rotation * Vec3::Z * 0.05,
            ..beam
        }));
    }
}
