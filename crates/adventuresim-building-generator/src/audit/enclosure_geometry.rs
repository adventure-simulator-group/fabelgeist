//! Continuous vertical sections through exact convex wall and timber solids.
use crate::{ResolvedSolid, ResolvedSolidShape};
use bevy::math::{Quat, Vec2, Vec3};

fn planes(solid: &ResolvedSolid) -> Vec<(Vec3, f32)> {
    match solid.shape {
        ResolvedSolidShape::Cuboid => {
            let rotation = Quat::from_rotation_y(solid.yaw_radians)
                * Quat::from_rotation_x(solid.crossfall_radians)
                * Quat::from_rotation_z(solid.longfall_radians);
            [Vec3::X, Vec3::Y, Vec3::Z]
                .into_iter()
                .zip(solid.size.to_array())
                .flat_map(|(axis, length)| {
                    let normal = rotation * axis;
                    [
                        (normal, -normal.dot(solid.centre) - length * 0.5),
                        (-normal, normal.dot(solid.centre) - length * 0.5),
                    ]
                })
                .collect()
        }
        ResolvedSolidShape::TimberPanelPrism {
            vertices,
            outward,
            depth_metres,
        } => {
            let normal = Vec3::new(outward.x, 0.0, outward.y);
            let centre = vertices.into_iter().sum::<Vec3>() / 3.0;
            let mut planes = vec![
                (normal, -normal.dot(centre) - depth_metres * 0.5),
                (-normal, normal.dot(centre) - depth_metres * 0.5),
            ];
            for index in 0..3 {
                let a = vertices[index];
                let b = vertices[(index + 1) % 3];
                let mut side = (b - a).cross(normal).normalize();
                if side.dot(centre - a) > 0.0 {
                    side = -side;
                }
                planes.push((side, -side.dot(a)));
            }
            planes
        }
        _ => Vec::new(),
    }
}

/// Project the clipped solid's vertical section onto elevation. Unioning these
/// intervals proves coverage at every height, including gaps between samples.
pub(super) fn vertical_interval(
    solid: &ResolvedSolid,
    origin: Vec3,
    direction: Vec3,
    depth: f32,
    base: f32,
    top: f32,
) -> Option<(f32, f32)> {
    let planes = planes(solid);
    if planes.is_empty() {
        return None;
    }
    let mut polygon = vec![
        Vec2::new(-depth, base),
        Vec2::new(depth, base),
        Vec2::new(depth, top),
        Vec2::new(-depth, top),
    ];
    for (normal, constant) in planes {
        let signed = |p: Vec2| normal.dot(origin + direction * p.x + Vec3::Y * p.y) + constant;
        let mut next = Vec::new();
        for (&a, &b) in polygon
            .iter()
            .zip(polygon.iter().cycle().skip(1))
            .take(polygon.len())
        {
            let da = signed(a);
            let db = signed(b);
            if da <= 0.0 {
                next.push(a);
            }
            if (da < 0.0 && db > 0.0) || (da > 0.0 && db < 0.0) {
                next.push(a.lerp(b, da / (da - db)));
            }
        }
        polygon = next;
    }
    (!polygon.is_empty()).then(|| {
        (
            polygon.iter().map(|p| p.y).fold(f32::INFINITY, f32::min),
            polygon
                .iter()
                .map(|p| p.y)
                .fold(f32::NEG_INFINITY, f32::max),
        )
    })
}
