//! Physical cuboid contact against an axis-aligned candidate volume.
use crate::ResolvedSolid;
use bevy::math::{Quat, Vec3};

pub(crate) fn triangle_overlaps_solid(
    triangle: [Vec3; 3],
    solid: &ResolvedSolid,
    tolerance: f32,
) -> bool {
    let rotation = Quat::from_rotation_y(solid.yaw_radians)
        * Quat::from_rotation_x(solid.crossfall_radians)
        * Quat::from_rotation_z(solid.longfall_radians);
    triangle_overlaps_bounds(
        triangle.map(|p| rotation.inverse() * (p - solid.centre)),
        (-solid.size * 0.5, solid.size * 0.5),
        tolerance,
    )
}

pub(crate) fn triangle_overlaps_bounds(
    triangle: [Vec3; 3],
    bounds: (Vec3, Vec3),
    tolerance: f32,
) -> bool {
    let centre = (bounds.0 + bounds.1) * 0.5;
    let half = (bounds.1 - bounds.0) * 0.5;
    let points = triangle.map(|p| p - centre);
    let edges = [
        points[1] - points[0],
        points[2] - points[1],
        points[0] - points[2],
    ];
    let mut axes = vec![Vec3::X, Vec3::Y, Vec3::Z, edges[0].cross(edges[1])];
    for edge in edges {
        for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
            axes.push(edge.cross(axis));
        }
    }
    axes.into_iter()
        .filter(|a| a.length_squared() > 0.000_001)
        .all(|axis| {
            let axis = axis.normalize();
            let radius = half.dot(axis.abs());
            let projection = points.map(|p| p.dot(axis));
            let min = projection.into_iter().fold(f32::INFINITY, f32::min);
            let max = projection.into_iter().fold(f32::NEG_INFINITY, f32::max);
            min < radius - tolerance && max > -radius + tolerance
        })
}

pub(crate) fn overlaps_bounds(solid: &ResolvedSolid, bounds: (Vec3, Vec3), tolerance: f32) -> bool {
    let bounds_centre = (bounds.0 + bounds.1) * 0.5;
    let bounds_half = (bounds.1 - bounds.0) * 0.5;
    let rotation = Quat::from_rotation_y(solid.yaw_radians)
        * Quat::from_rotation_x(solid.crossfall_radians)
        * Quat::from_rotation_z(solid.longfall_radians);
    let solid_axes = [rotation * Vec3::X, rotation * Vec3::Y, rotation * Vec3::Z];
    let world_axes = [Vec3::X, Vec3::Y, Vec3::Z];
    let solid_half = solid.size * 0.5;
    let delta = bounds_centre - solid.centre;
    let mut axes = Vec::with_capacity(15);
    axes.extend(world_axes);
    axes.extend(solid_axes);
    for world in world_axes {
        for local in solid_axes {
            let cross = world.cross(local);
            if cross.length_squared() > 0.000_001 {
                axes.push(cross.normalize());
            }
        }
    }
    axes.into_iter().all(|axis| {
        let solid_radius = solid_half.x * solid_axes[0].dot(axis).abs()
            + solid_half.y * solid_axes[1].dot(axis).abs()
            + solid_half.z * solid_axes[2].dot(axis).abs();
        let bounds_radius = bounds_half.x * axis.x.abs()
            + bounds_half.y * axis.y.abs()
            + bounds_half.z * axis.z.abs();
        solid_radius + bounds_radius - delta.dot(axis).abs() > tolerance
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roof_triangle_contact_checks_corners_and_rejects_empty_bounds_overlap() {
        let bounds = (-Vec3::ONE, Vec3::ONE);
        assert!(triangle_overlaps_bounds(
            [
                Vec3::new(0.5, -0.5, 0.0),
                Vec3::new(2.0, 2.0, 0.0),
                Vec3::new(2.0, 2.0, 2.0)
            ],
            bounds,
            0.001
        ));
        assert!(triangle_overlaps_bounds(
            [
                Vec3::new(-0.5, 0.0, -0.5),
                Vec3::new(0.5, 0.0, -0.5),
                Vec3::new(0.0, 0.0, 0.5)
            ],
            bounds,
            0.001
        ));
        assert!(!triangle_overlaps_bounds(
            [
                Vec3::new(0.8, 2.0, 0.0),
                Vec3::new(2.0, 0.8, 0.0),
                Vec3::new(2.0, 2.0, 0.0)
            ],
            bounds,
            0.001
        ));
    }
}
