//! Parent-child identity and native-source cardinality of the physical roof graph.
use super::*;
pub(super) fn audit(plan: &BuildingPlan, issues: &mut Vec<AuditIssue>) {
    for assembly in plan
        .roof_assemblies
        .iter()
        .filter(|assembly| assembly.parent.is_some())
    {
        let Some(parent) = assembly.parent else {
            continue;
        };
        let references = plan
            .roof_assemblies
            .iter()
            .filter(|candidate| candidate.id == parent)
            .flat_map(|candidate| &candidate.children)
            .filter(|child| child.child == assembly.id)
            .count();
        if references != 1 {
            issues.push(issue(
                "orphan_roof_child",
                format!(
                    "roof {} has {references} parent graph references, expected one",
                    assembly.id.0
                ),
            ));
        }
    }
    let expected = plan.roofs.len()
        + plan.roof_dormers.len()
        + plan
            .towers
            .iter()
            .filter(|tower| tower.roof.is_some())
            .count()
        + plan.square_towers.len();
    if expected != plan.roof_assemblies.len() {
        issues.push(issue(
            "legacy_roof_authority",
            format!(
                "expected {expected} resolved roof assemblies, found {}",
                plan.roof_assemblies.len()
            ),
        ));
    }
}
