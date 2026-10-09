use super::*;
pub(super) fn audit(defense: &crate::ProjectedDefenseAssembly, issues: &mut Vec<AuditIssue>) {
    let expected_material_phase = match defense.kind {
        ProjectedDefenseKind::Hoarding => {
            defense.material == ProjectedDefenseMaterial::Timber
                && defense.phase == ProjectedDefensePhase::TemporaryCampaignWork
                && matches!(
                    defense.deployment,
                    ProjectedDefenseDeployment::SocketsOnly | ProjectedDefenseDeployment::Deployed
                )
        }
        _ => {
            defense.material == ProjectedDefenseMaterial::Masonry
                && defense.phase == ProjectedDefensePhase::PermanentMainWork
                && defense.deployment == ProjectedDefenseDeployment::Permanent
        }
    };
    if !expected_material_phase {
        issues.push(issue(
            "projected_defense_phase_material_mismatch",
            format!(
                "projected defense owner {} has an incoherent material, phase, or deployment",
                defense.owner.0
            ),
        ));
    }
    let target_matches_installation = match defense.kind {
        ProjectedDefenseKind::Machicolation => {
            defense.tactical_target == ProjectedDefenseTarget::GateApproach
        }
        ProjectedDefenseKind::Breteche => {
            defense.tactical_target == ProjectedDefenseTarget::ThreatenedWallFoot
        }
        ProjectedDefenseKind::Hoarding => {
            defense.tactical_target == ProjectedDefenseTarget::CampaignSiegeFront
        }
        ProjectedDefenseKind::Bartizan => {
            defense.tactical_target == ProjectedDefenseTarget::ThreatenedCorner
        }
    };
    if !target_matches_installation {
        issues.push(issue(
            "projected_defense_tactical_target_mismatch",
            format!(
                "projected defense owner {} lacks a coherent named tactical target",
                defense.owner.0
            ),
        ));
    }
}

pub(super) fn host_portal_is_cut(
    plan: &BuildingPlan,
    defense: &crate::ProjectedDefenseAssembly,
) -> bool {
    defense.host_portal_void.is_none_or(|id| {
        plan.resolved_geometry.voids.iter().any(|void| {
            void.id == id
                && void.owner == defense.host_owner
                && void.subtracts_from == defense.host_owner
                && void.role == VoidRole::AccessPortal
        })
    })
}

pub(super) fn host_bond_is_physical(
    plan: &BuildingPlan,
    defense: &crate::ProjectedDefenseAssembly,
) -> bool {
    defense.host_bond.is_none_or(|id| {
        plan.resolved_geometry.junction_bonds.iter().any(|bond| {
            bond.id == id
                && bond.owners.contains(&defense.owner)
                && bond.owners.contains(&defense.host_owner)
        })
    })
}
