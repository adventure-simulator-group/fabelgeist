//! Physical cuboid contact against an axis-aligned candidate volume.
use crate::ResolvedSolid;
use bevy::math::{Quat, Vec3};

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
