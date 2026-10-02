//! Verify shed enclosure against actual roof planes and owned weathering.
use super::*;
use crate::{RoofAssembly, RoofChildAssembly, RoofChildKind, RoofFace, WallSourceId};

const CONTACT_TOLERANCE_METRES: f32 = 0.03;

pub(super) fn audit(plan: &BuildingPlan, issues: &mut Vec<AuditIssue>) {
    for parent in &plan.roof_assemblies {
        for link in parent
            .children
            .iter()
            .filter(|link| link.kind == RoofChildKind::ShedDormer)
        {
            if !enclosed(plan, parent, link) {
                issues.push(issue("unseated_shed_dormer", format!(
                    "shed roof {} must enclose both cheeks and seat its head weathering on the parent",
                    link.child.0)));
            }
        }
    }
}

fn height(face: &RoofFace, p: Vec2) -> f32 {
    -(face.plane.normal.x * p.x + face.plane.normal.z * p.y + face.plane.constant)
        / face.plane.normal.y
}

fn enclosed(plan: &BuildingPlan, parent: &RoofAssembly, link: &RoofChildAssembly) -> bool {
    let Some(child) = plan
        .roof_assemblies
        .iter()
        .find(|roof| roof.id == link.child)
    else {
        return false;
    };
    let Some(wall) = plan
        .wall_assemblies
        .iter()
        .find(|wall| wall.source == WallSourceId::RoofChildFront { roof: child.id })
    else {
        return false;
    };
    let [face] = child.faces.as_slice() else {
        return false;
    };
    let normal = face.plane.normal.normalize();
    let outward = wall.frame.outward;
    if Vec2::new(normal.x, normal.z)
        .normalize_or_zero()
        .dot(outward)
        < 0.999
    {
        return false;
    }
    let ceiling = |p: Vec2| height(face, p) - face.thickness_metres / normal.y;
    let parent_height = |p: Vec2| {
        parent
            .faces
            .iter()
            .filter(|face| roof_face_contains_plan_point_inclusive(face, p))
            .map(|face| height(face, p))
            .max_by(f32::total_cmp)
    };
    // Each cheek must span from its actual front wall to the rear seam;
    // points must lie on either the parent covering or child underside.
    let cheeks = child.enclosure_faces.iter().collect::<Vec<_>>();
    if cheeks.len() != 2
        || cheeks.iter().any(|cheek| {
            cheek.polygon.len() < 3
                || cheek.polygon.iter().any(|p| !p.is_finite())
                || (cheek.polygon[1] - cheek.polygon[0])
                    .cross(cheek.polygon[2] - cheek.polygon[0])
                    .length_squared()
                    < 0.000_001
        })
    {
        return false;
    }
    for side in [-1.0, 1.0] {
        let front = wall.frame.origin + wall.frame.tangent * side * wall.length_metres * 0.5;
        let Some(cheek) = cheeks.iter().find(|cheek| {
            cheek.polygon.iter().all(|p| {
                (Vec2::new(p.x, p.z) - front).dot(wall.frame.tangent).abs()
                    < CONTACT_TOLERANCE_METRES
            })
        }) else {
            return false;
        };
        let rear = cheek
            .polygon
            .iter()
            .map(|p| Vec2::new(p.x, p.z))
            .min_by(|a, b| a.dot(outward).total_cmp(&b.dot(outward)))
            .unwrap();
        let Some(parent_y) = parent_height(rear) else {
            return false;
        };
        if (parent_y - ceiling(rear)).abs() > CONTACT_TOLERANCE_METRES {
            return false;
        }
        let point = |p: Vec2, y: f32| Vec3::new(p.x, y, p.y);
        let Some(front_base) = parent_height(front) else {
            return false;
        };
        for expected in [
            point(front, front_base),
            point(front, ceiling(front)),
            point(rear, ceiling(rear)),
        ] {
            if !cheek
                .polygon
                .iter()
                .any(|p| p.distance(expected) < CONTACT_TOLERANCE_METRES)
            {
                return false;
            }
        }
        if cheek.polygon.iter().any(|p| {
            let xy = Vec2::new(p.x, p.z);
            (p.y - ceiling(xy)).abs() > CONTACT_TOLERANCE_METRES
                && parent_height(xy).is_none_or(|y| (y - p.y).abs() > CONTACT_TOLERANCE_METRES)
        }) {
            return false;
        }
    }
    seated_head_flashing(plan, parent, link, outward)
}

fn seated_head_flashing(
    plan: &BuildingPlan,
    parent: &RoofAssembly,
    link: &RoofChildAssembly,
    outward: Vec2,
) -> bool {
    let Some(edge) = parent
        .edges
        .iter()
        .filter(|edge| {
            edge.flashing
                .is_some_and(|id| link.flashing_ids.contains(&id))
        })
        .min_by(|a, b| {
            Vec2::new(a.start.x + a.end.x, a.start.z + a.end.z)
                .dot(outward)
                .total_cmp(&Vec2::new(b.start.x + b.end.x, b.start.z + b.end.z).dot(outward))
        })
    else {
        return false;
    };
    let Some(solid) = plan
        .resolved_geometry
        .solids
        .iter()
        .find(|solid| Some(solid.id) == edge.flashing)
    else {
        return false;
    };
    let midpoint = (edge.start + edge.end) * 0.5;
    let rotation = Quat::from_euler(
        bevy::math::EulerRot::YXZ,
        solid.yaw_radians,
        solid.crossfall_radians,
        solid.longfall_radians,
    );
    let along = (edge.end - edge.start).normalize();
    let across = Vec3::Y.cross(along).normalize();
    let local_seam = rotation.inverse() * (midpoint - solid.centre);
    (rotation * Vec3::X).dot(along).abs() > 0.999
        && (rotation * Vec3::Z * solid.size.z).dot(across).abs() > CONTACT_TOLERANCE_METRES
        && local_seam
            .abs()
            .cmple(solid.size * 0.5 + Vec3::splat(0.001))
            .all()
        && solid.centre.distance(midpoint) < CONTACT_TOLERANCE_METRES
        && (solid.size.x - (edge.end - edge.start).length()).abs() < 0.15
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shed_audit_rejects_open_cheeks_raised_coverings_and_displaced_head_weathering() {
        let source = crate::generate(&crate::BuildingProgram::fixture(
            BuildingArchetype::HallHouse,
            42,
        ))
        .unwrap();
        let parent = &source.roof_assemblies[0];
        let link = parent
            .children
            .iter()
            .find(|link| link.kind == RoofChildKind::ShedDormer)
            .unwrap();
        let child = source
            .roof_assemblies
            .iter()
            .position(|roof| roof.id == link.child)
            .unwrap();
        let head = parent
            .edges
            .iter()
            .filter(|edge| {
                edge.flashing
                    .is_some_and(|id| link.flashing_ids.contains(&id))
            })
            .min_by(|a, b| (a.start.x + a.end.x).total_cmp(&(b.start.x + b.end.x)))
            .unwrap()
            .flashing
            .unwrap();
        for mutation in 0..6 {
            let mut plan = source.clone();
            match mutation {
                0 => {
                    plan.roof_assemblies[child].enclosure_faces.pop();
                }
                1 => {
                    let face = &mut plan.roof_assemblies[child].faces[0];
                    face.plane.constant -= face.plane.normal.y * 0.3;
                    for p in &mut face.polygon {
                        p.y += 0.3;
                    }
                }
                2 => {
                    for point in &mut plan.roof_assemblies[child].enclosure_faces[0].polygon {
                        point.x += 0.4;
                    }
                }
                3 => {
                    plan.resolved_geometry
                        .solids
                        .iter_mut()
                        .find(|solid| solid.id == head)
                        .unwrap()
                        .centre
                        .y += 0.5
                }
                4 => plan.roof_assemblies[child].enclosure_faces[0]
                    .polygon
                    .clear(),
                _ => {
                    plan.resolved_geometry
                        .solids
                        .iter_mut()
                        .find(|solid| solid.id == head)
                        .unwrap()
                        .yaw_radians += std::f32::consts::FRAC_PI_2
                }
            }
            let mut issues = Vec::new();
            audit(&plan, &mut issues);
            assert!(!issues.is_empty(), "mutation {mutation}");
        }
    }
}
