//! Measured contact of finished oriented cuboids, including thin roof sheets.
use crate::{ResolvedBounds, ResolvedSolid};
use bevy::math::{Quat, Vec3};
const CONTACT_TOLERANCE_METRES: f32 = 0.001;
const INTERFACE_DEPTH_METRES: f32 = 0.01;
fn rotation(s: &ResolvedSolid) -> Quat {
    Quat::from_euler(
        bevy::math::EulerRot::YXZ,
        s.yaw_radians,
        s.crossfall_radians,
        s.longfall_radians,
    )
}
fn corners(s: &ResolvedSolid) -> [Vec3; 8] {
    let r = rotation(s);
    std::array::from_fn(|i| {
        s.centre
            + r * (s.size
                * 0.5
                * Vec3::new(
                    if i & 1 == 0 { -1.0 } else { 1.0 },
                    if i & 2 == 0 { -1.0 } else { 1.0 },
                    if i & 4 == 0 { -1.0 } else { 1.0 },
                ))
    })
}
fn clip(start: Vec3, end: Vec3, solid: &ResolvedSolid) -> Option<[Vec3; 2]> {
    let inverse = rotation(solid).inverse();
    let a = inverse * (start - solid.centre);
    let b = inverse * (end - solid.centre);
    let delta = b - a;
    let half = solid.size * 0.5;
    let mut enter = 0.0_f32;
    let mut leave = 1.0_f32;
    for axis in 0..3 {
        if delta[axis].abs() < 0.000001 {
            if a[axis].abs() > half[axis] + CONTACT_TOLERANCE_METRES {
                return None;
            }
        } else {
            let lower = (-half[axis] - a[axis]) / delta[axis];
            let upper = (half[axis] - a[axis]) / delta[axis];
            enter = enter.max(lower.min(upper));
            leave = leave.min(lower.max(upper));
            if enter > leave + CONTACT_TOLERANCE_METRES {
                return None;
            }
        }
    }
    Some([
        start.lerp(end, enter.clamp(0.0, 1.0)),
        start.lerp(end, leave.clamp(0.0, 1.0)),
    ])
}
pub(super) fn measured(a: &ResolvedSolid, b: &ResolvedSolid) -> Option<ResolvedBounds> {
    let mut points = Vec::new();
    for (source, target) in [(a, b), (b, a)] {
        let corners = corners(source);
        for index in 0..8 {
            for axis in [1, 2, 4] {
                if index & axis == 0
                    && let Some(segment) = clip(corners[index], corners[index | axis], target)
                {
                    points.extend(segment);
                }
            }
        }
    }
    let first = *points.first()?;
    let bounds = points.iter().fold(
        ResolvedBounds {
            min: first,
            max: first,
        },
        |a, p| ResolvedBounds {
            min: a.min.min(*p),
            max: a.max.max(*p),
        },
    );
    const MINIMUM_BEARING_AREA_SQUARE_METRES: f32 = 0.000001;
    let positive_area = points.iter().any(|a| {
        points.iter().any(|b| {
            (*a - first).cross(*b - first).length_squared()
                > 4.0 * MINIMUM_BEARING_AREA_SQUARE_METRES.powi(2)
        })
    });
    if !positive_area {
        return None;
    }
    let margin = Vec3::splat(INTERFACE_DEPTH_METRES);
    Some(ResolvedBounds {
        min: bounds.min - margin,
        max: bounds.max + margin,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    #[test]
    fn diagonal_edge_contact_does_not_support_a_sheet_or_masonry() {
        let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);
        let a = ResolvedSolid {
            id: ResolvedItemId(1),
            owner: GeometryOwnerId(1),
            centre: Vec3::ZERO,
            size: Vec3::splat(2.0),
            yaw_radians: std::f32::consts::FRAC_PI_4,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
            role: SolidRole::DomesticHeating,
            shape: ResolvedSolidShape::Cuboid,
            supported_by: vec![],
        };
        let mut b = a.clone();
        b.id = ResolvedItemId(2);
        b.centre = rotation * Vec3::new(2.0, 2.0, 0.0);
        assert!(measured(&a, &b).is_none());
        b.centre = rotation * Vec3::new(2.0, 1.5, 0.0);
        assert!(measured(&a, &b).is_some());
    }
}
