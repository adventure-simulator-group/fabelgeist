//! Inspect final material triangles independently of roof-route selection.
use super::*;

pub(super) fn audit(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    shaft: ResolvedBounds,
    issues: &mut Vec<AuditIssue>,
) {
    let weather = h
        .parts
        .iter()
        .filter(|part| {
            matches!(
                part.kind,
                HeatingPartKind::RoofFlashing
                    | HeatingPartKind::RoofUpstand
                    | HeatingPartKind::RoofCounterFlashing
            )
        })
        .filter_map(|part| {
            plan.resolved_geometry
                .solids
                .iter()
                .find(|solid| solid.id == part.solid)
        })
        .collect::<Vec<_>>();
    if obstructed(plan, h.roof.face, shaft, &weather) {
        fail(
            issues,
            "blocked_heating_roof_route",
            "a non-host roof or enclosure crosses the chimney or its weathering",
        );
    }
}

pub(in crate::heating) fn obstructed(
    plan: &BuildingPlan,
    target: ResolvedItemId,
    shaft: ResolvedBounds,
    weather: &[&ResolvedSolid],
) -> bool {
    for roof in &plan.roof_assemblies {
        let triangles =
            roof.faces
                .iter()
                .filter(|face| face.id != target)
                .flat_map(crate::tessellate_roof_face)
                .chain(roof.enclosure_faces.iter().flat_map(|face| {
                    crate::tessellate_roof_enclosure(face, &plan.wall_assemblies)
                }));
        if triangles.into_iter().any(|triangle| {
            crate::solid_overlap::triangle_overlaps_bounds(
                triangle.positions,
                (shaft.min, shaft.max),
                GEOMETRY_TOLERANCE_METRES,
            ) || weather.iter().any(|solid| {
                crate::solid_overlap::triangle_overlaps_solid(
                    triangle.positions,
                    solid,
                    GEOMETRY_TOLERANCE_METRES,
                )
            })
        }) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parent_cut_drift_blocks_the_shed_flue_despite_its_intact_child_penetration() {
        let mut plan =
            generate(&BuildingProgram::fixture(BuildingArchetype::HallHouse, 2)).unwrap();
        let h = plan.domestic_heating.as_ref().unwrap();
        let target = plan
            .roof_assemblies
            .iter()
            .find(|roof| roof.id == h.roof.roof)
            .unwrap();
        let parent = target.parent.unwrap();
        let child_cut = target
            .faces
            .iter()
            .find(|face| face.id == h.roof.face)
            .unwrap()
            .cutouts[h.roof.cutout_index]
            .clone();
        let roof = plan
            .roof_assemblies
            .iter_mut()
            .find(|roof| roof.id == parent)
            .unwrap();
        for cut in roof.faces.iter_mut().flat_map(|face| &mut face.cutouts) {
            for point in cut {
                point.z += 10.0;
            }
        }
        assert_eq!(
            plan.roof_assemblies
                .iter()
                .find(|roof| roof.id == h.roof.roof)
                .unwrap()
                .faces
                .iter()
                .find(|face| face.id == h.roof.face)
                .unwrap()
                .cutouts[h.roof.cutout_index],
            child_cut
        );
        assert!(
            super::super::audit(&plan)
                .iter()
                .any(|issue| issue.code == "blocked_heating_roof_route")
        );
    }
}
