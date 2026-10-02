//! A weather-face penetration cut through both roof skins and sealed with sheet lead.
use super::{assembly::Assembly, placement::roof_height};
use crate::*;
use bevy::math::{Mat3, Quat, Vec2, Vec3};
const WEATHER_OVERLAP_METRES: f32 = 0.06;
pub(super) const CUT_CLEARANCE_METRES: f32 = 0.015;
const LEAD_THICKNESS_METRES: f32 = 0.006;
// A shallow change of fall lets a continuous pan emerge from below the uphill
// tiles to above the downhill tiles without an upward step in its water route.
pub(in crate::heating) const PAN_FALL_ADJUSTMENT: f32 = 0.04;

pub(super) struct PenetrationFootprint {
    pub cut_min: Vec2,
    pub cut_max: Vec2,
    pub weather_min: Vec2,
    pub weather_max: Vec2,
}
impl PenetrationFootprint {
    pub fn new(face: &RoofFace, centre: Vec2) -> Self {
        let half = Vec2::splat(super::placement::SHAFT_HALF_WIDTH_METRES);
        let normal = face.plane.normal.normalize();
        let translation = Vec2::new(normal.x, normal.z) * face.thickness_metres;
        let cut_min =
            (centre - half).min(centre - half + translation) - Vec2::splat(CUT_CLEARANCE_METRES);
        let cut_max =
            (centre + half).max(centre + half + translation) + Vec2::splat(CUT_CLEARANCE_METRES);
        let overlap = Vec2::splat(WEATHER_OVERLAP_METRES)
            + Vec2::new(normal.x, normal.z).abs() * LEAD_THICKNESS_METRES * 1.5;
        Self {
            cut_min,
            cut_max,
            weather_min: cut_min - overlap,
            weather_max: cut_max + overlap,
        }
    }
}

pub(super) fn penetrate(a: &mut Assembly<'_>, roofs: &mut [RoofAssembly]) {
    let roof = roofs.iter_mut().find(|r| r.id == a.plan.roof.roof).unwrap();
    let face = roof
        .faces
        .iter_mut()
        .find(|f| f.id == a.plan.roof.face)
        .unwrap();
    let shaft = a.placement.shaft(0.0);
    let inner_min = Vec2::new(shaft.min.x, shaft.min.z);
    let inner_max = Vec2::new(shaft.max.x, shaft.max.z);
    // The inward skin is translated along the normal. Include that translation
    // in the weather cut, so its entire extruded hole clears the vertical shaft.
    let footprint = PenetrationFootprint::new(face, (inner_min + inner_max) * 0.5);
    let min = footprint.cut_min;
    let max = footprint.cut_max;
    let mut contour = [min, Vec2::new(min.x, max.y), max, Vec2::new(max.x, min.y)]
        .map(|p| Vec3::new(p.x, roof_height(face, p), p.y))
        .to_vec();
    let area = |p: &[Vec3]| {
        p.iter()
            .zip(p.iter().cycle().skip(1))
            .take(p.len())
            .map(|(a, b)| a.x * b.z - b.x * a.z)
            .sum::<f32>()
    };
    if area(&contour).signum() == area(&face.polygon).signum() {
        contour.reverse();
    }
    a.plan.roof.cutout_index = face.cutouts.len();
    face.cutouts.push(contour.clone());
    exclude_cut_samples(a.geometry, face.id, &contour);
    for index in 0..4 {
        let start = contour[index];
        let end = contour[(index + 1) % 4];
        let edge = a.id(11, index as u64 + 1);
        let p = Vec2::new(start.x, start.z);
        let q = Vec2::new(end.x, end.z);
        let mut lower = p.min(q) - Vec2::splat(WEATHER_OVERLAP_METRES);
        let mut upper = p.max(q) + Vec2::splat(WEATHER_OVERLAP_METRES);
        if (p.x - q.x).abs() < 0.001 {
            if p.x < inner_min.x {
                upper.x = inner_min.x + WEATHER_OVERLAP_METRES;
            } else {
                lower.x = inner_max.x - WEATHER_OVERLAP_METRES;
            }
        } else if p.y < inner_min.y {
            upper.y = inner_min.y + WEATHER_OVERLAP_METRES;
        } else {
            lower.y = inner_max.y - WEATHER_OVERLAP_METRES;
        }
        let flashing = sheet(a, face, lower, upper, (inner_min + inner_max) * 0.5);
        roof.edges.push(RoofEdge {
            id: edge,
            start,
            end,
            kind: RoofEdgeKind::OpeningCut,
            adjacent_faces: vec![face.id],
            flashing: Some(flashing),
            drainage_terminal: None,
        });
        a.plan.roof.edges.push(edge);
        a.plan.roof.flashing.push(flashing);
    }
    super::weathering::build(a, face, inner_min, inner_max);
}
fn sheet(
    a: &mut Assembly<'_>,
    face: &RoofFace,
    min: Vec2,
    max: Vec2,
    shaft_centre: Vec2,
) -> ResolvedItemId {
    let roof_normal = face.plane.normal.normalize();
    let downhill = Vec3::new(roof_normal.x, 0.0, roof_normal.z).normalize();
    let normal = (roof_normal - downhill * roof_normal.y * PAN_FALL_ADJUSTMENT).normalize();
    let x = Vec3::new(1.0, -normal.x / normal.y, 0.0).normalize();
    let z = x.cross(normal);
    let rotation = Quat::from_mat3(&Mat3::from_cols(x, normal, z));
    let plan_centre = (min + max) * 0.5;
    let height_adjustment = Vec3::new(
        plan_centre.x - shaft_centre.x,
        0.0,
        plan_centre.y - shaft_centre.y,
    )
    .dot(downhill)
        * PAN_FALL_ADJUSTMENT;
    let centre = Vec3::new(
        plan_centre.x,
        roof_height(face, plan_centre) + height_adjustment,
        plan_centre.y,
    );
    let size = Vec3::new(
        (max.x - min.x) / x.x,
        LEAD_THICKNESS_METRES,
        (max.y - min.y) / z.z,
    );
    a.oriented_part(
        HeatingPartKind::RoofFlashing,
        BuildingLodMaterial::LeadAlloy,
        centre,
        size,
        rotation,
    )
}

// Drainage origins follow remaining weather-face material with the same
// containment tolerance used when the original roof grid is generated.
pub(super) fn exclude_cut_samples(
    geometry: &mut ResolvedGeometry,
    face: ResolvedItemId,
    cut: &[Vec3],
) {
    let polygon = cut.iter().map(|p| Vec2::new(p.x, p.z)).collect::<Vec<_>>();
    for network in geometry
        .roof_drainage_networks
        .iter_mut()
        .filter(|n| n.face == face)
    {
        network.samples.retain(|sample| {
            let point = Vec2::new(sample.surface_point.x, sample.surface_point.z);
            !crate::generator::plan_point_in_convex_polygon(point, &polygon)
        });
    }
}
