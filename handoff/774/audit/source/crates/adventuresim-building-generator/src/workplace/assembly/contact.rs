//! Exact cuboid contact for architectural members and yawed vessel staves.
use crate::ResolvedSolid;
use crate::{Architectural, SpatialBounds};
use bevy::math::{Quat, Vec3};

pub(crate) fn rotation(solid: &ResolvedSolid) -> Quat {
    Quat::from_rotation_y(solid.yaw_radians.radians())
        * Quat::from_rotation_x(solid.crossfall_radians.radians())
        * Quat::from_rotation_z(solid.longfall_radians.radians())
}

pub(crate) fn bounds(
    solid: &ResolvedSolid,
) -> Result<SpatialBounds<Architectural>, crate::GenerationError> {
    let orientation = rotation(solid);
    let half = solid.size.metres() * 0.5;
    let extent = (orientation * Vec3::X).abs() * half.x
        + (orientation * Vec3::Y).abs() * half.y
        + (orientation * Vec3::Z).abs() * half.z;
    Ok(SpatialBounds::<Architectural>::from_metres(
        solid.centre.metres() - extent,
        solid.centre.metres() + extent,
    )?)
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
                .zip(solid.size.metres().to_array())
                .map(|(direction, length)| direction.dot(axis).abs() * length * 0.5)
                .sum::<f32>()
        };
        (a.centre.metres() - b.centre.metres()).dot(axis).abs()
            > radius(a, axes_a) + radius(b, axes_b) + tolerance
    };
    !axes_a.into_iter().chain(axes_b).any(separated)
        && !axes_a
            .into_iter()
            .any(|a| axes_b.into_iter().any(|b| separated(a.cross(b))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_geometry::Position;
    use crate::{GeometryOwnerId, ResolvedItemId, ResolvedSolidShape, SolidRole};

    #[test]
    fn parallel_diagonal_staves_do_not_bear_on_overlapping_axis_bounds() {
        let first = crate::ResolvedSolid::new(
            crate::CollisionCuboid::<crate::Architectural>::from_metres(
                ResolvedItemId(1),
                Vec3::ZERO,
                Vec3::new(2.0, 1.0, 0.1),
                std::f32::consts::FRAC_PI_4,
                0.0,
                0.0,
            )
            .unwrap(),
            GeometryOwnerId(1),
            SolidRole::WorkplacePart,
            ResolvedSolidShape::Cuboid,
            vec![],
        );
        let mut second = first.clone();
        second.centre =
            Position::<crate::Architectural>::from_metres(rotation(&first) * Vec3::Z * 0.3)
                .unwrap();
        let a = bounds(&first).unwrap();
        let b = bounds(&second).unwrap();
        assert!(
            (a.max().metres().min(b.max().metres()) - a.min().metres().max(b.min().metres()))
                .min_element()
                > 0.0
        );
        assert!(!touches(&first, &second, 0.04));
        second.centre =
            Position::<crate::Architectural>::from_metres(rotation(&first) * Vec3::Z * 0.1)
                .unwrap();
        assert!(touches(&first, &second, 0.001));
    }
}
