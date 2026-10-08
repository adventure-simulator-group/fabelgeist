use super::*;
use crate::GenerationResult as Result;
use crate::plan_geometry::ArchitecturalPlanPoint;
const MIN_OUTLET_CLEARANCE_METRES: f32 = 0.5;
const MIN_SHEET_NORMAL_ALIGNMENT: f32 = 0.9995;
const SHEET_PLANE_TOLERANCE_METRES: f32 = 0.035;
const SHEET_SIDE_TOLERANCE_METRES: f32 = 0.001;

pub(super) fn audit(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    issues: &mut Vec<AuditIssue>,
) -> Result<()> {
    let Some(roof) = plan.roof_assemblies.iter().find(|r| r.id == h.roof.roof) else {
        fail(
            issues,
            "missing_heating_roof_cut",
            "the flue has no roof owner",
        );
        return Ok(());
    };
    let Some(face) = roof.faces.iter().find(|f| f.id == h.roof.face) else {
        fail(
            issues,
            "missing_heating_roof_cut",
            "the flue has no roof face",
        );
        return Ok(());
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
        return Ok(());
    };
    let mut bounds = first.cuboid_bounds()?;
    for solid in &flues {
        bounds = bounds.union(solid.cuboid_bounds()?);
    }
    let shaft = rect(bounds);
    super::roof_route::audit(plan, h, bounds, issues);
    let actual_cut = face.cutouts.get(h.roof.cutout_index);
    let cut_valid = crate::geometry_index::try_any(actual_cut, |cut| {
        let p = polygon(cut);
        let normal = face.plane.normal.normalize();
        let translation = Vec3::new(normal.x, 0.0, normal.z) * face.thickness_metres;
        let clearance = Vec3::new(
            super::super::roof::CUT_CLEARANCE_METRES,
            0.0,
            super::super::roof::CUT_CLEARANCE_METRES,
        );
        let required = rect(SpatialBounds::<Architectural>::from_metres(
            bounds
                .min()
                .metres()
                .min(bounds.min().metres() + translation)
                - clearance,
            bounds
                .max()
                .metres()
                .max(bounds.max().metres() + translation)
                + clearance,
        )?);
        Ok::<bool, crate::GenerationError>(
            p.difference(&required).unsigned_area() < AREA_TOLERANCE_SQUARE_METRES
                && required.difference(&p).unsigned_area() < AREA_TOLERANCE_SQUARE_METRES
                && cut.iter().all(|v| {
                    (face.plane.normal.dot(*v) + face.plane.constant).abs()
                        < GEOMETRY_TOLERANCE_METRES
                })
                && weathering(plan, h, face, cut, bounds)?
                && super::weathering::audit(plan, h, face, bounds)?
                && super::weathering::continuous_pan(plan, h, face, bounds)?,
        )
    })?;
    let blocked = crate::tessellate_roof_face(face).iter().any(|t| {
        polygon(&t.positions).intersection(&shaft).unsigned_area() > AREA_TOLERANCE_SQUARE_METRES
    });
    let edges_valid = edges(roof, face, h, actual_cut);
    let high = highest_roof_corner(face, bounds)?.metres();
    let _: () = if !cut_valid
        || blocked
        || !edges_valid
        || bounds.max().metres().y < high + MIN_OUTLET_CLEARANCE_METRES
    {
        fail(
            issues,
            "invalid_heating_roof_penetration",
            "the shaft needs a bounded owned hole through both roof skins, weathering and an outdoor outlet",
        );
    };
    Ok(())
}

fn weathering(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    face: &RoofFace,
    cut: &[Vec3],
    shaft: SpatialBounds<Architectural>,
) -> Result<bool> {
    const MINIMUM_WEATHER_LAP_METRES: f32 = 0.02;
    let margin = Vec3::new(MINIMUM_WEATHER_LAP_METRES, 0.0, MINIMUM_WEATHER_LAP_METRES);
    let Some(first) = cut.first() else {
        return Ok(false);
    };
    let mut cut_bounds = SpatialBounds::<Architectural>::from_metres(*first, *first)?;
    for point in cut.iter().skip(1) {
        cut_bounds = cut_bounds.union(SpatialBounds::<Architectural>::from_metres(*point, *point)?);
    }
    let required = rect(SpatialBounds::<Architectural>::from_metres(
        cut_bounds.min().metres() - margin,
        cut_bounds.max().metres() + margin,
    )?)
    .difference(&rect(SpatialBounds::<Architectural>::from_metres(
        shaft.min().metres() + margin,
        shaft.max().metres() - margin,
    )?));
    let mut coverage = geo::MultiPolygon::new(vec![]);
    for id in &h.roof.flashing {
        let Some(solid) = plan.resolved_geometry.solids.iter().find(|s| s.id == *id) else {
            return Ok(false);
        };
        if !sheet_section(plan, solid, face, shaft)? {
            return Ok(false);
        }
        for mesh in compile_solid_detail(plan, solid)?.meshes {
            for triangle in mesh.indices.as_chunks::<3>().0 {
                let points = triangle.map(|i| mesh.vertices[i as usize].position);
                coverage = coverage.union(&polygon(&points));
            }
        }
    }
    Ok(
        required.difference(&coverage).unsigned_area() < AREA_TOLERANCE_SQUARE_METRES
            && coverage.difference(&polygon(&face.polygon)).unsigned_area()
                < AREA_TOLERANCE_SQUARE_METRES
            && face
                .cutouts
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != h.roof.cutout_index)
                .all(|(_, other)| {
                    coverage.intersection(&polygon(other)).unsigned_area()
                        < AREA_TOLERANCE_SQUARE_METRES
                }),
    )
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
    shaft: SpatialBounds<Architectural>,
) -> Result<bool> {
    let rotation = bevy::math::Quat::from_euler(
        bevy::math::EulerRot::YXZ,
        solid.yaw_radians.radians(),
        solid.crossfall_radians.radians(),
        solid.longfall_radians.radians(),
    );
    let roof_normal = face.plane.normal.normalize();
    let downhill = Vec3::new(roof_normal.x, 0.0, roof_normal.z).normalize();
    // The continuous pan deliberately falls slightly less than the covering.
    // Check that authored section, including on a shallow shed slope.
    let pan_normal = (roof_normal
        - downhill * roof_normal.y * super::super::roof::PAN_FALL_ADJUSTMENT)
        .normalize();
    if (rotation * Vec3::Y).dot(pan_normal).abs() < MIN_SHEET_NORMAL_ALIGNMENT {
        return Ok(false);
    }
    for vertex in compile_solid_detail(plan, solid)?
        .meshes
        .iter()
        .flat_map(|m| &m.vertices)
    {
        if (face.plane.normal.dot(vertex.position) + face.plane.constant).abs()
            / face.plane.normal.length()
            > SHEET_PLANE_TOLERANCE_METRES
        {
            return Ok(false);
        }
    }
    let distance = (face.plane.normal.dot(solid.centre.metres()) + face.plane.constant).abs()
        / face.plane.normal.length();
    if distance > SHEET_PLANE_TOLERANCE_METRES {
        return Ok(false);
    }
    let shaft_centre = (shaft.min().metres() + shaft.max().metres()) * 0.5;
    let downhill = Vec3::new(face.plane.normal.x, 0.0, face.plane.normal.z);
    let side = (solid.centre.metres() - shaft_centre).dot(downhill);
    let signed_distance = (face.plane.normal.dot(solid.centre.metres()) + face.plane.constant)
        / face.plane.normal.length();
    // Water leaves the tiles onto the backpan, then runs over the apron.
    // Side strips may sit less than a millimetre above the covering on a
    // shallow roof. Their sign, full-section lap and continuous fall matter.
    if side.abs() > SHEET_SIDE_TOLERANCE_METRES && signed_distance * side <= 0.0 {
        return Ok(false);
    }
    Ok(true)
}

fn highest_roof_corner(
    face: &RoofFace,
    bounds: SpatialBounds<Architectural>,
) -> Result<crate::spatial_geometry::Elevation<Architectural>> {
    let high = [bounds.min().metres().x, bounds.max().metres().x]
        .into_iter()
        .flat_map(|x| [bounds.min().metres().z, bounds.max().metres().z].map(|z| Vec2::new(x, z)))
        .try_fold(0.0_f32, |high, p| {
            Ok::<_, crate::GenerationError>(
                high.max(
                    super::super::placement::roof_height(
                        face,
                        ArchitecturalPlanPoint::from_metres(p)?,
                    )?
                    .metres(),
                ),
            )
        })?;
    Ok(crate::spatial_geometry::Elevation::from_metres(high)?)
}
