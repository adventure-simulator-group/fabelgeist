//! Continuous coverage of the gable section, independently clipped from planes.
use bevy::math::{Vec2, Vec3};
use geo::{Area, BooleanOps, Coord, LineString, MultiPolygon, Polygon};

use super::{enclosure::GABLE_GAP, issue};
use crate::{
    AuditIssue, BuildingPlan, ROOF_ENCLOSURE_THICKNESS_METRES, RidgeAxis, RoofAssembly, RoofKind,
    RoofPiece, WallSourceId,
};

const CONTACT_TOLERANCE_METRES: f32 = 0.002;
const AREA_TOLERANCE_SQUARE_METRES: f32 = 0.0001;
const PARALLEL_ALIGNMENT: f32 = 0.99;

fn polygon(points: &[Vec2]) -> Polygon<f32> {
    let mut coordinates = points
        .iter()
        .map(|p| Coord { x: p.x, y: p.y })
        .collect::<Vec<_>>();
    if let Some(first) = coordinates.first().copied() {
        coordinates.push(first);
    }
    Polygon::new(LineString::new(coordinates), Vec::new())
}

fn clip(points: &[Vec2], normal: Vec2, constant: f32) -> Vec<Vec2> {
    let mut result = Vec::new();
    for (&a, &b) in points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
    {
        let da = normal.dot(a) + constant;
        let db = normal.dot(b) + constant;
        if da <= 0.0 {
            result.push(a);
        }
        if (da < 0.0 && db > 0.0) || (da > 0.0 && db < 0.0) {
            result.push(a.lerp(b, da / (da - db)));
        }
    }
    result
}

struct GableSection {
    origin: Vec2,
    outward: Vec2,
    across: Vec2,
    low: f32,
    high: f32,
}

impl GableSection {
    fn new(plan: &BuildingPlan, recipe: &RoofPiece, sign: f32) -> Self {
        let (along, across, length, width) = match recipe.ridge_axis {
            RidgeAxis::Z => (Vec2::Y, Vec2::X, recipe.size.y, recipe.size.x),
            RidgeAxis::X => (Vec2::X, Vec2::Y, recipe.size.x, recipe.size.y),
        };
        let outward = along * sign;
        let nominal = recipe.centre + outward * length * 0.5;
        let mut section = Self {
            origin: nominal,
            outward,
            across,
            low: -width * 0.5,
            high: width * 0.5,
        };
        let hosts = plan
            .wall_assemblies
            .iter()
            .filter(|wall| {
                matches!(wall.source, WallSourceId::StoreyWall { .. })
                    && wall.replaced_by_owner.is_none()
                    && wall.frame.outside_room.is_none()
                    && wall.frame.outward.dot(outward) > PARALLEL_ALIGNMENT
                    && (wall.base_elevation_metres + wall.height_metres - recipe.base_height_metres)
                        .abs()
                        < CONTACT_TOLERANCE_METRES
                    && (wall.frame.origin - nominal).dot(along).abs() < recipe.eave_metres
                    && (wall.frame.origin - nominal).dot(across).abs() < width * 0.5
            })
            .collect::<Vec<_>>();
        if let Some(host) = hosts.first() {
            section.origin += outward
                * ((host.frame.origin - nominal).dot(outward)
                    + (host.thickness_metres * 0.5).min(ROOF_ENCLOSURE_THICKNESS_METRES * 0.5)
                    - ROOF_ENCLOSURE_THICKNESS_METRES * 0.5);
            section.low = hosts
                .iter()
                .map(|wall| {
                    (wall.frame.origin - nominal).dot(across)
                        - (wall.length_metres + wall.thickness_metres) * 0.5
                })
                .fold(f32::INFINITY, f32::min);
            section.high = hosts
                .iter()
                .map(|wall| {
                    (wall.frame.origin - nominal).dot(across)
                        + (wall.length_metres + wall.thickness_metres) * 0.5
                })
                .fold(f32::NEG_INFINITY, f32::max);
        }
        section
    }

    fn expected(&self, roof: &RoofAssembly, base: f32) -> Polygon<f32> {
        let top = roof
            .faces
            .iter()
            .flat_map(|face| &face.polygon)
            .map(|p| p.y)
            .fold(base, f32::max);
        let mut points = vec![
            Vec2::new(self.low, base + CONTACT_TOLERANCE_METRES),
            Vec2::new(self.high, base + CONTACT_TOLERANCE_METRES),
            Vec2::new(self.high, top),
            Vec2::new(self.low, top),
        ];
        for face in &roof.faces {
            let n = face.plane.normal;
            let slope = Vec2::new(n.x, n.z);
            points = clip(
                &points,
                Vec2::new(slope.dot(self.across), n.y),
                slope.dot(self.origin)
                    + face.plane.constant
                    + face.thickness_metres * n.length()
                    + CONTACT_TOLERANCE_METRES * n.y,
            );
        }
        polygon(&points)
    }

    fn coverage(&self, roof: &RoofAssembly) -> MultiPolygon<f32> {
        let mut union = MultiPolygon(Vec::new());
        for face in &roof.enclosure_faces {
            if face.polygon.len() < 3 {
                continue;
            }
            let n = (face.polygon[1] - face.polygon[0])
                .cross(face.polygon[2] - face.polygon[0])
                .normalize_or_zero();
            if Vec2::new(n.x, n.z).dot(self.outward) < PARALLEL_ALIGNMENT {
                continue;
            }
            let front = Vec2::new(face.polygon[0].x, face.polygon[0].z);
            let depth = (front - self.origin).dot(self.outward);
            if !(-CONTACT_TOLERANCE_METRES
                ..=ROOF_ENCLOSURE_THICKNESS_METRES + CONTACT_TOLERANCE_METRES)
                .contains(&depth)
            {
                continue;
            }
            let points = face
                .polygon
                .iter()
                .map(|p| Vec2::new((Vec2::new(p.x, p.z) - self.origin).dot(self.across), p.y))
                .collect::<Vec<_>>();
            union = union.union(&MultiPolygon(vec![polygon(&points)]));
        }
        union
    }
}

pub(super) fn audit(plan: &BuildingPlan, issues: &mut Vec<AuditIssue>) {
    for roof in plan
        .roof_assemblies
        .iter()
        .filter(|roof| roof.kind == RoofKind::Gable && roof.parent.is_none())
    {
        let Some(recipe) = roof
            .source_piece_index
            .and_then(|index| plan.roofs.get(index))
        else {
            continue;
        };
        for sign in [-1.0, 1.0] {
            let section = GableSection::new(plan, recipe, sign);
            let expected = section.expected(roof, recipe.base_height_metres);
            let missing = MultiPolygon(vec![expected]).difference(&section.coverage(roof));
            let area = missing.unsigned_area();
            if area > AREA_TOLERANCE_SQUARE_METRES {
                let witness = missing.0[0].exterior().0[0];
                let location = section.origin + section.across * witness.x;
                let point = Vec3::new(location.x, witness.y, location.y);
                issues.push(issue(GABLE_GAP, format!("roof {} gable toward {:?} leaves {area:.6} square metres open near {point:?}", roof.id.0, section.outward)));
            }
        }
    }
}
