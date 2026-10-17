//! Measured contact of finished oriented cuboids, including thin roof sheets.
//! The bearing policy retains its own tolerances and Euler YXZ arithmetic.
//! CuboidCorners supplies topology only; ground-contact classification has a
//! separate computed-f32 policy and uses no bearing tolerance.
use crate::spatial_geometry::{Displacement, Position, RigidRotation};
use crate::{Architectural, CollisionError, CuboidCorners, ResolvedSolid, SpatialBounds};
use bevy::math::{Quat, Vec3};
const CONTACT_TOLERANCE_METRES: f32 = 0.001;
const INTERFACE_DEPTH_METRES: f32 = 0.01;
fn rotation(s: &ResolvedSolid) -> Quat {
    Quat::from_euler(
        bevy::math::EulerRot::YXZ,
        s.yaw_radians.radians(),
        s.crossfall_radians.radians(),
        s.longfall_radians.radians(),
    )
}
fn corners(s: &ResolvedSolid) -> Result<CuboidCorners<Architectural>, CollisionError> {
    let construct = || {
        CuboidCorners::from_pose(
            s.centre,
            s.size,
            RigidRotation::from_quaternion(rotation(s))?,
        )
    };
    construct().map_err(|cause| CollisionError {
        source_id: s.id,
        cause,
    })
}
fn clip(
    start: Vec3,
    end: Vec3,
    solid: &ResolvedSolid,
) -> Result<Option<[Vec3; 2]>, CollisionError> {
    let construct = || {
        let inverse = rotation(solid).inverse();
        let a =
            Displacement::<Architectural>::from_metres(inverse * (start - solid.centre.metres()))?
                .metres();
        let b =
            Displacement::<Architectural>::from_metres(inverse * (end - solid.centre.metres()))?
                .metres();
        let delta = Displacement::<Architectural>::from_metres(b - a)?.metres();
        let half = solid.size.metres() * 0.5;
        let mut enter = 0.0_f32;
        let mut leave = 1.0_f32;
        for axis in 0..3 {
            if delta[axis].abs() < 0.000001 {
                if a[axis].abs() > half[axis] + CONTACT_TOLERANCE_METRES {
                    return Ok(None);
                }
            } else {
                let lower = (-half[axis] - a[axis]) / delta[axis];
                let upper = (half[axis] - a[axis]) / delta[axis];
                enter = enter.max(lower.min(upper));
                leave = leave.min(lower.max(upper));
                if enter > leave + CONTACT_TOLERANCE_METRES {
                    return Ok(None);
                }
            }
        }
        Ok(Some([
            Position::<Architectural>::from_metres(start.lerp(end, enter.clamp(0.0, 1.0)))?
                .metres(),
            Position::<Architectural>::from_metres(start.lerp(end, leave.clamp(0.0, 1.0)))?
                .metres(),
        ]))
    };
    construct().map_err(|cause| CollisionError {
        source_id: solid.id,
        cause,
    })
}
pub(super) fn measured(
    a: &ResolvedSolid,
    b: &ResolvedSolid,
) -> Result<Option<SpatialBounds<Architectural>>, crate::GenerationError> {
    let mut points = Vec::new();
    for (source, target) in [(a, b), (b, a)] {
        for edge in corners(source)?.edges() {
            if let Some(segment) = clip(edge.start.metres(), edge.end.metres(), target)? {
                points.extend(segment);
            }
        }
    }
    let Some(&first) = points.first() else {
        return Ok(None);
    };
    let mut bounds = SpatialBounds::<Architectural>::at(Position::from_metres(first)?);
    for &point in &points {
        bounds = bounds.including(Position::from_metres(point)?);
    }
    const MINIMUM_BEARING_AREA_SQUARE_METRES: f32 = 0.000001;
    let positive_area = points.iter().any(|a| {
        points.iter().any(|b| {
            (*a - first).cross(*b - first).length_squared()
                > 4.0 * MINIMUM_BEARING_AREA_SQUARE_METRES.powi(2)
        })
    });
    if !positive_area {
        return Ok(None);
    }
    let margin = Vec3::splat(INTERFACE_DEPTH_METRES);
    Ok(Some(SpatialBounds::from_metres(
        bounds.min().metres() - margin,
        bounds.max().metres() + margin,
    )?))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    #[test]
    fn diagonal_edge_contact_does_not_support_a_sheet_or_masonry() {
        let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);
        let a = crate::ResolvedSolid::new(
            crate::CollisionCuboid::<crate::Architectural>::from_metres(
                ResolvedItemId(1),
                Vec3::ZERO,
                Vec3::splat(2.0),
                std::f32::consts::FRAC_PI_4,
                0.0,
                0.0,
            )
            .unwrap(),
            GeometryOwnerId(1),
            SolidRole::DomesticHeating,
            ResolvedSolidShape::Cuboid,
            vec![],
        );
        let mut b = a.clone();
        b.id = ResolvedItemId(2);
        b.centre = crate::spatial_geometry::Position::<crate::Architectural>::from_metres(
            rotation * Vec3::new(2.0, 2.0, 0.0),
        )
        .unwrap();
        assert!(measured(&a, &b).unwrap().is_none());
        b.centre = crate::spatial_geometry::Position::<crate::Architectural>::from_metres(
            rotation * Vec3::new(2.0, 1.5, 0.0),
        )
        .unwrap();
        assert!(measured(&a, &b).unwrap().is_some());
    }
}
