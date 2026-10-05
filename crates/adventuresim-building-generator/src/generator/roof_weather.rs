//! Private native projection kernel for authored weather faces and their receiving cuts.
use super::*;
pub(super) fn contains(face: &RoofFace, target_plan: Vec2) -> bool {
    let outline = face
        .polygon
        .iter()
        .map(|point| Vec2::new(point.x, point.z))
        .collect::<Vec<_>>();
    plan_point_in_polygon(target_plan, &outline)
        && !face.cutouts.iter().any(|cutout| {
            let cutout = cutout
                .iter()
                .map(|point| Vec2::new(point.x, point.z))
                .collect::<Vec<_>>();
            plan_point_in_polygon(target_plan, &cutout)
        })
}
pub(super) fn recipient_offsets(kind: RoofKind) -> &'static [Vec2] {
    const ORDINARY: [Vec2; 13] = [
        Vec2::ZERO,
        Vec2::new(0.25, 0.0),
        Vec2::new(-0.25, -0.0),
        Vec2::new(0.0, 0.25),
        Vec2::new(-0.0, -0.25),
        Vec2::new(0.50, 0.0),
        Vec2::new(-0.50, -0.0),
        Vec2::new(0.0, 0.50),
        Vec2::new(-0.0, -0.50),
        Vec2::new(0.75, 0.0),
        Vec2::new(-0.75, -0.0),
        Vec2::new(0.0, 0.75),
        Vec2::new(-0.0, -0.75),
    ];
    const TOWER: [Vec2; 9] = [
        Vec2::ZERO,
        Vec2::new(0.50, 0.0),
        Vec2::new(-0.50, -0.0),
        Vec2::new(0.0, 0.50),
        Vec2::new(-0.0, -0.50),
        Vec2::new(0.75, 0.0),
        Vec2::new(-0.75, -0.0),
        Vec2::new(0.0, 0.75),
        Vec2::new(-0.0, -0.75),
    ];
    if kind == RoofKind::Pavilion {
        &TOWER
    } else {
        &ORDINARY
    }
}
