//! Local member coordinates keep longitudinal fibers aligned through rotated joinery.
use bevy::math::{Quat, Vec2, Vec3};

use crate::BUILDING_DETAIL_UV_METRES_PER_UNIT;

pub(crate) fn grain_axis(size: Vec3) -> Vec3 {
    if size.x >= size.y && size.x >= size.z {
        Vec3::X
    } else if size.y >= size.z {
        Vec3::Y
    } else {
        Vec3::Z
    }
}

pub(crate) fn member_uvs(
    positions: [Vec3; 4],
    centre: Vec3,
    size: Vec3,
    rotation: Quat,
    normal: Vec3,
) -> [Vec2; 4] {
    let grain = grain_axis(size);
    // Hewn oak's longitudinal coordinate is V. End faces use a transverse basis.
    let along = if grain.dot(normal).abs() < 0.5 {
        grain
    } else if grain == Vec3::Y {
        Vec3::Z
    } else {
        Vec3::Y
    };
    let across = along.cross(normal);
    // A member's position chooses a repeat phase, without world-space rotation stretching it.
    let phase = Vec2::new(centre.dot(Vec3::new(0.37, 0.61, 0.83)), centre.length());
    positions.map(|point| {
        let local = rotation.inverse() * (point - centre);
        Vec2::new(local.dot(across), local.dot(along)) / BUILDING_DETAIL_UV_METRES_PER_UNIT + phase
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fibers_follow_a_rotated_member_without_changing_scale() {
        for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
            for rotation in [Quat::IDENTITY, Quat::from_rotation_z(0.63)] {
                let normal = if axis == Vec3::Z { Vec3::X } else { Vec3::Z };
                let across = axis.cross(normal);
                let centre = Vec3::new(4.0, 2.0, -3.0);
                let points = [Vec3::ZERO, axis, axis + across * 0.1, across * 0.1]
                    .map(|p| centre + rotation * p);
                let uv = member_uvs(points, centre, Vec3::splat(0.1) + axis, rotation, normal);
                let longitudinal = uv[1] - uv[0];
                assert!(longitudinal.x.abs() < 0.00001);
                assert!(
                    (longitudinal.y - 1.0 / BUILDING_DETAIL_UV_METRES_PER_UNIT).abs() < 0.00001
                );
                assert!((uv[2] - uv[1]).y.abs() < 0.00001);
            }
        }
    }
}
