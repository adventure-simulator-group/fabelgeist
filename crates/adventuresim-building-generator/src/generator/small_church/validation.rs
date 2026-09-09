use super::*;

pub(crate) fn audit_small_church(plan: &BuildingPlan, issues: &mut Vec<AuditIssue>) {
    let Some(church) = &plan.small_church else {
        return;
    };
    if plan.archetype != BuildingArchetype::ParishChurch || plan.church.is_some() {
        issues.push(AuditIssue {
            code: "small_church_wrong_authority",
            message: "modest church and cathedral assemblies cannot share authority".to_owned(),
        });
    }
    for solid in &plan.resolved_geometry.solids {
        if solid.role == SolidRole::OpeningClosure {
            continue;
        }
        let min = solid.centre - solid.size * 0.5;
        let max = solid.centre + solid.size * 0.5;
        if (max.min(church.public_route.max) - min.max(church.public_route.min)).min_element()
            > 0.025
        {
            issues.push(AuditIssue {
                code: "small_church_blocked_aisle",
                message: format!("solid {} blocks the real public aisle", solid.id.0),
            });
        }
    }
}
