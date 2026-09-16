use super::*;
pub(super) fn audit(plan: &BuildingPlan, h: &DomesticHeatingPlan, issues: &mut Vec<AuditIssue>) {
    let Some(roof) = plan.roof_assemblies.iter().find(|r| r.id == h.roof.roof) else {
        fail(
            issues,
            "missing_heating_roof_cut",
            "the flue has no roof owner",
        );
        return;
    };
    let Some(face) = roof.faces.iter().find(|f| f.id == h.roof.face) else {
        fail(
            issues,
            "missing_heating_roof_cut",
            "the flue has no roof face",
        );
        return;
    };
    let flues = h
        .parts
        .iter()
        .filter(|p| p.kind == HeatingPartKind::Flue)
        .filter_map(|p| {
            plan.resolved_geometry
                .solids
                .iter()
                .find(|s| s.id == p.solid)
        })
        .collect::<Vec<_>>();
    let Some(first) = flues.first() else {
        return;
    };
    let bounds = flues.iter().fold(first.cuboid_bounds(), |a, s| {
        let b = s.cuboid_bounds();
        ResolvedBounds {
            min: a.min.min(b.min),
            max: a.max.max(b.max),
        }
    });
    let shaft = rect(bounds);
    let actual_cut = face.cutouts.get(h.roof.cutout_index);
    let cut_valid = actual_cut.is_some_and(|cut| {
        let p = polygon(cut);
        let normal = face.plane.normal.normalize();
        let translation = Vec3::new(normal.x, 0.0, normal.z) * face.thickness_metres;
        let clearance = Vec3::new(
            super::super::roof::CUT_CLEARANCE_METRES,
            0.0,
            super::super::roof::CUT_CLEARANCE_METRES,
        );
        let required = rect(ResolvedBounds {
            min: bounds.min.min(bounds.min + translation) - clearance,
            max: bounds.max.max(bounds.max + translation) + clearance,
        });
        p.difference(&required).unsigned_area() < 0.00001
            && required.difference(&p).unsigned_area() < 0.00001
            && cut.iter().all(|v| {
                (face.plane.normal.dot(*v) + face.plane.constant).abs() < GEOMETRY_TOLERANCE_METRES
            })
            && weathering(plan, h, face, cut, bounds)
            && super::weathering::audit(plan, h, face, bounds)
            && super::weathering::continuous_pan(plan, h, face, bounds)
    });
    let blocked = crate::tessellate_roof_face(face)
        .iter()
        .any(|t| polygon(&t.positions).intersection(&shaft).unsigned_area() > 0.00001);
    let edges_valid = edges(roof, face, h, actual_cut);
    let high = [bounds.min.x, bounds.max.x]
        .into_iter()
        .flat_map(|x| [bounds.min.z, bounds.max.z].map(|z| Vec2::new(x, z)))
        .map(|p| super::super::placement::roof_height(face, p))
        .fold(0.0_f32, f32::max);
    if !cut_valid || blocked || !edges_valid || bounds.max.y < high + 0.5 {
        fail(
            issues,
            "invalid_heating_roof_penetration",
            "the shaft needs a bounded owned hole through both roof skins, weathering and an outdoor outlet",
        );
    }
}

fn weathering(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    face: &RoofFace,
    cut: &[Vec3],
    shaft: ResolvedBounds,
) -> bool {
    const MINIMUM_WEATHER_LAP_METRES: f32 = 0.02;
    let margin = Vec3::new(MINIMUM_WEATHER_LAP_METRES, 0.0, MINIMUM_WEATHER_LAP_METRES);
    let cut_bounds = cut.iter().fold(
        ResolvedBounds {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        },
        |a, p| ResolvedBounds {
            min: a.min.min(*p),
            max: a.max.max(*p),
        },
    );
    let required = rect(ResolvedBounds {
        min: cut_bounds.min - margin,
        max: cut_bounds.max + margin,
    })
    .difference(&rect(ResolvedBounds {
        min: shaft.min + margin,
        max: shaft.max - margin,
    }));
    let mut coverage = geo::MultiPolygon::new(vec![]);
    for id in &h.roof.flashing {
        let Some(solid) = plan.resolved_geometry.solids.iter().find(|s| s.id == *id) else {
            return false;
        };
        if !sheet_section(plan, solid, face, shaft) {
            return false;
        }
        for mesh in compile_solid_detail(plan, solid).meshes {
            for triangle in mesh.indices.as_chunks::<3>().0 {
                let points = triangle.map(|i| mesh.vertices[i as usize].position);
                coverage = coverage.union(&polygon(&points));
            }
        }
    }
    required.difference(&coverage).unsigned_area() < 0.00001
        && coverage.difference(&polygon(&face.polygon)).unsigned_area() < 0.00001
        && face
            .cutouts
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != h.roof.cutout_index)
            .all(|(_, other)| coverage.intersection(&polygon(other)).unsigned_area() < 0.00001)
}

fn edges(
    roof: &RoofAssembly,
    face: &RoofFace,
    h: &DomesticHeatingPlan,
    actual_cut: Option<&Vec<Vec3>>,
) -> bool {
    h.roof.edges.len() == 4
        && h.roof.flashing.len() == 4
        && h.roof.edges.iter().copied().collect::<BTreeSet<_>>().len() == 4
        && h.roof
            .flashing
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len()
            == 4
        && actual_cut.is_some_and(|cut| {
            cut.len() == 4
                && h.roof.edges.iter().enumerate().all(|(index, id)| {
                    roof.edges.iter().find(|e| e.id == *id).is_some_and(|e| {
                        e.kind == RoofEdgeKind::OpeningCut
                            && e.adjacent_faces == [face.id]
                            && e.start.distance(cut[index]) < GEOMETRY_TOLERANCE_METRES
                            && e.end.distance(cut[(index + 1) % 4]) < GEOMETRY_TOLERANCE_METRES
                            && e.flashing.is_some_and(|f| {
                                h.roof.flashing.contains(&f)
                                    && h.parts.iter().any(|p| {
                                        p.solid == f && p.kind == HeatingPartKind::RoofFlashing
                                    })
                            })
                    })
                })
        })
}

fn sheet_section(
    plan: &BuildingPlan,
    solid: &ResolvedSolid,
    face: &RoofFace,
    shaft: ResolvedBounds,
) -> bool {
    let rotation = bevy::math::Quat::from_euler(
        bevy::math::EulerRot::YXZ,
        solid.yaw_radians,
        solid.crossfall_radians,
        solid.longfall_radians,
    );
    if (rotation * Vec3::Y)
        .dot(face.plane.normal.normalize())
        .abs()
        < 0.9995
    {
        return false;
    }
    for vertex in compile_solid_detail(plan, solid)
        .meshes
        .iter()
        .flat_map(|m| &m.vertices)
    {
        if (face.plane.normal.dot(vertex.position) + face.plane.constant).abs()
            / face.plane.normal.length()
            > 0.035
        {
            return false;
        }
    }
    let distance = (face.plane.normal.dot(solid.centre) + face.plane.constant).abs()
        / face.plane.normal.length();
    if distance > 0.035 {
        return false;
    }
    let shaft_centre = (shaft.min + shaft.max) * 0.5;
    let downhill = Vec3::new(face.plane.normal.x, 0.0, face.plane.normal.z);
    let side = (solid.centre - shaft_centre).dot(downhill);
    let signed_distance =
        (face.plane.normal.dot(solid.centre) + face.plane.constant) / face.plane.normal.length();
    // Water leaves the tiles onto the backpan, then runs over the apron.
    if (side < -0.001 && signed_distance >= -0.001) || (side > 0.001 && signed_distance <= 0.001) {
        return false;
    }
    true
}
