//! Exact cuboid contact for architectural members and yawed vessel staves.
use crate::{ResolvedBounds, ResolvedSolid};
use bevy::math::{Quat, Vec3};

pub(crate) fn rotation(solid: &ResolvedSolid) -> Quat {
    Quat::from_rotation_y(solid.yaw_radians)
        * Quat::from_rotation_x(solid.crossfall_radians)
        * Quat::from_rotation_z(solid.longfall_radians)
}

pub(crate) fn bounds(solid: &ResolvedSolid) -> ResolvedBounds {
    let orientation = rotation(solid);
    let half = solid.size * 0.5;
    let extent = (orientation * Vec3::X).abs() * half.x
        + (orientation * Vec3::Y).abs() * half.y
        + (orientation * Vec3::Z).abs() * half.z;
    ResolvedBounds {
        min: solid.centre - extent,
        max: solid.centre + extent,
    }
}

pub(crate) fn touches(a: &ResolvedSolid, b: &ResolvedSolid, tolerance: f32) -> bool {
    let axes_a = [Vec3::X, Vec3::Y, Vec3::Z].map(|axis| rotation(a) * axis);
    let axes_b = [Vec3::X, Vec3::Y, Vec3::Z].map(|axis| rotation(b) * axis);
    let separated = |axis: Vec3| {
        let Some(axis) = axis.try_normalize() else {
            return false;
        };
        let radius = |solid: &ResolvedSolid, axes: [Vec3; 3]| {
            axes.into_iter()
                .zip(solid.size.to_array())
                .map(|(direction, length)| direction.dot(axis).abs() * length * 0.5)
                .sum::<f32>()
        };
        (a.centre - b.centre).dot(axis).abs() > radius(a, axes_a) + radius(b, axes_b) + tolerance
    };
    !axes_a.into_iter().chain(axes_b).any(separated)
        && !axes_a
            .into_iter()
            .any(|a| axes_b.into_iter().any(|b| separated(a.cross(b))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GeometryOwnerId, ResolvedItemId, ResolvedSolidShape, SolidRole};

    #[test]
    fn parallel_diagonal_staves_do_not_bear_on_overlapping_axis_bounds() {
        let first = ResolvedSolid {
            id: ResolvedItemId(1),
            owner: GeometryOwnerId(1),
            centre: Vec3::ZERO,
            size: Vec3::new(2.0, 1.0, 0.1),
            yaw_radians: std::f32::consts::FRAC_PI_4,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
            role: SolidRole::WorkplacePart,
            shape: ResolvedSolidShape::Cuboid,
            supported_by: vec![],
        };
        let mut second = first.clone();
        second.centre = rotation(&first) * Vec3::Z * 0.3;
        let a = bounds(&first);
        let b = bounds(&second);
        assert!((a.max.min(b.max) - a.min.max(b.min)).min_element() > 0.0);
        assert!(!touches(&first, &second, 0.04));
        second.centre = rotation(&first) * Vec3::Z * 0.1;
        assert!(touches(&first, &second, 0.001));
    }
}
