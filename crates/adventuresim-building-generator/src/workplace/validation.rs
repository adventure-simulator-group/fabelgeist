use super::*;
use crate::{AuditIssue, BuildingPlan};

const CONTACT_TOLERANCE_METRES: f32 = 0.04;
const PLOT_EDGE_ALLOWANCE_METRES: f32 = 0.25;

pub(crate) fn audit_workplace(plan: &BuildingPlan, issues: &mut Vec<AuditIssue>) {
    let Some(workplace) = &plan.workplace else {
        return;
    };
    let solids = &plan.resolved_geometry.solids;
    for part in &workplace.parts {
        let Some(solid) = solids.iter().find(|solid| solid.id == part.solid) else {
            issues.push(AuditIssue {
                code: "workplace_missing_part",
                message: format!("missing {:?} geometry", part.feature),
            });
            continue;
        };
        let min = solid.centre - solid.size * 0.5;
        let max = solid.centre + solid.size * 0.5;
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
                    && touches(
                        min,
                        max,
                        other.centre - other.size * 0.5,
                        other.centre + other.size * 0.5,
                    )
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
    };
    if !workplace.parts.iter().any(|part| part.feature == essential) {
        issues.push(AuditIssue {
            code: "workplace_missing_function",
            message: format!("{:?} lacks {essential:?}", workplace.kind),
        });
    }
}

fn touches(a_min: Vec3, a_max: Vec3, b_min: Vec3, b_max: Vec3) -> bool {
    (a_max.min(b_max) - a_min.max(b_min)).min_element() >= -CONTACT_TOLERANCE_METRES
}
