use super::*;

/// Keep the exact outdoor stack and weathering in both distance representations.
pub(crate) fn compile_heating_lod(plan: &BuildingPlan) -> BuildingDetail {
    let mut detail = BuildingDetail { meshes: Vec::new() };
    if let Some(heating) = &plan.domestic_heating {
        for part in heating.parts.iter().filter(|p| {
            matches!(
                p.kind,
                crate::HeatingPartKind::Flue
                    | crate::HeatingPartKind::RoofFlashing
                    | crate::HeatingPartKind::RoofUpstand
                    | crate::HeatingPartKind::RoofCounterFlashing
            )
        }) {
            if let Some(solid) = plan
                .resolved_geometry
                .solids
                .iter()
                .find(|s| s.id == part.solid)
            {
                append_oriented_cuboid(&mut detail, part.material, solid, None);
            }
        }
    }
    detail
}
