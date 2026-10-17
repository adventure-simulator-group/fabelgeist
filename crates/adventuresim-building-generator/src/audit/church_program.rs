use super::*;
pub(super) fn audit(
    plan: &BuildingPlan,
    church: &crate::ChurchAssembly,
    issues: &mut Vec<AuditIssue>,
) {
    let program = church.program;
    if !matches!(
        plan.archetype,
        BuildingArchetype::Cathedral | BuildingArchetype::ParishChurch
    ) || program != crate::ChurchProgram::URBAN_BRICK_BASILICA
        || plan.small_church.is_some()
    {
        issues.push(issue(
            "invalid_church_program",
            "church is not the frozen east-oriented 4-bay cruciform basilica type".to_owned(),
        ));
    }
    if plan
        .storeys
        .iter()
        .any(|storey| !storey.walls.is_empty() || !storey.openings.is_empty())
    {
        issues.push(issue(
            "legacy_church_authority",
            "church still contains generic cell walls or overlay openings".to_owned(),
        ));
    }
    let strictly_increasing =
        |values: &[f32]| values.windows(2).all(|pair| pair[1] > pair[0] + 0.10);
    if church.nave_axes_metres.len() != usize::from(program.nave_bays)
        || church.choir_axes_metres.len() != usize::from(program.choir_bays)
        || !strictly_increasing(&church.nave_axes_metres)
        || !strictly_increasing(&church.choir_axes_metres)
        || church
            .nave_axes_metres
            .last()
            .is_none_or(|axis| *axis >= church.crossing_axis_metres)
        || church
            .choir_axes_metres
            .first()
            .is_none_or(|axis| *axis <= church.crossing_axis_metres)
    {
        issues.push(issue(
            "invalid_church_bay_axes",
            "nave/crossing/choir axes are missing, unordered, or blocked".to_owned(),
        ));
    }
}
