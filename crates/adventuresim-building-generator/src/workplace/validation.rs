use super::*;
use crate::{AuditIssue, BuildingPlan};

const CONTACT_TOLERANCE_METRES: f32 = 0.04;
const PLOT_EDGE_ALLOWANCE_METRES: f32 = 0.25;

pub(crate) fn audit_workplace(plan: &BuildingPlan, issues: &mut Vec<AuditIssue>) {
    let Some(workplace) = &plan.workplace else {
        return;
    };
    super::horse_mill::audit_circuit(plan, issues);
    let solids = &plan.resolved_geometry.solids;
    for part in &workplace.parts {
        let Some(solid) = solids.iter().find(|solid| solid.id == part.solid) else {
            issues.push(AuditIssue {
                code: "workplace_missing_part",
                message: format!("missing {:?} geometry", part.feature),
            });
            continue;
        };
        let bounds = super::assembly::contact::bounds(solid);
        let min = bounds.min;
        let max = bounds.max;
        if min.x < -PLOT_EDGE_ALLOWANCE_METRES
            || min.z < -PLOT_EDGE_ALLOWANCE_METRES
            || max.x > workplace.plot_dimensions_metres.x + PLOT_EDGE_ALLOWANCE_METRES
            || max.z > workplace.plot_dimensions_metres.y + PLOT_EDGE_ALLOWANCE_METRES
        {
            issues.push(AuditIssue {
                code: "workplace_outside_plot",
                message: format!("{:?} extends outside its reserved plot", part.feature),
            });
        }
        if min.y > CONTACT_TOLERANCE_METRES
            && !solids.iter().any(|other| {
                other.id != solid.id
                    && super::assembly::contact::touches(solid, other, CONTACT_TOLERANCE_METRES)
            })
        {
            issues.push(AuditIssue {
                code: "workplace_floating_part",
                message: format!("{:?} has no physical bearing or attachment", part.feature),
            });
        }
        if workplace.passages.iter().any(|passage| {
            (max.min(passage.max) - min.max(passage.min)).min_element() > CONTACT_TOLERANCE_METRES
        }) {
            issues.push(AuditIssue {
                code: "workplace_blocked_passage",
                message: format!("{:?} blocks a working passage", part.feature),
            });
        }
    }
    let essential = match workplace.kind {
        WorkplaceKind::Barn => WorkplaceFeature::StorageBin,
        WorkplaceKind::Stable => WorkplaceFeature::Stall,
        WorkplaceKind::Granary => WorkplaceFeature::Hoist,
        WorkplaceKind::Smithy => WorkplaceFeature::Forge,
        WorkplaceKind::Bakehouse => WorkplaceFeature::Oven,
        WorkplaceKind::MarketHall => WorkplaceFeature::Counter,
        WorkplaceKind::TimberYard => WorkplaceFeature::TimberStack,
        WorkplaceKind::Carpenter => WorkplaceFeature::SawBench,
        WorkplaceKind::Brewery => WorkplaceFeature::Vat,
        WorkplaceKind::Malthouse => WorkplaceFeature::Kiln,
        WorkplaceKind::Warehouse => WorkplaceFeature::LoadingHoist,
        WorkplaceKind::Dyer => WorkplaceFeature::DyeKettle,
        WorkplaceKind::Tannery => WorkplaceFeature::SoakingTank,
        WorkplaceKind::HorseMill => WorkplaceFeature::Millstone,
    };
    if !workplace.parts.iter().any(|part| part.feature == essential) {
        issues.push(AuditIssue {
            code: "workplace_missing_function",
            message: format!("{:?} lacks {essential:?}", workplace.kind),
        });
    }
}
