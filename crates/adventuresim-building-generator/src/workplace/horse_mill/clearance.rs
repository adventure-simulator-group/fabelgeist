use super::*;
use crate::{AuditIssue, BuildingPlan, ResolvedSolid};

const TRACK_GROUND_CLEARANCE_METRES: f32 = 0.18;
const ANIMAL_HEADROOM_METRES: f32 = 2.2;
const PIVOT_JOINT_RADIUS_METRES: f32 = 0.4;
const MOVING_CLEARANCE_METRES: f32 = 0.02;

/// Conservative full-turn volume: rotation about the drive axis preserves height and radius.
/// Bounds come from every actual rotated cuboid, including the brace and low draw linkage.
struct SweptBand {
    inner: f32,
    outer: f32,
    bottom: f32,
    top: f32,
}

impl SweptBand {
    fn from_solid(solid: &ResolvedSolid) -> Self {
        let bounds = super::super::assembly::contact::bounds(solid);
        let min = Vec2::new(bounds.min.x, bounds.min.z);
        let max = Vec2::new(bounds.max.x, bounds.max.z);
        Self {
            inner: DRIVE_CENTRE.clamp(min, max).distance(DRIVE_CENTRE),
            outer: [min, max, Vec2::new(min.x, max.y), Vec2::new(max.x, min.y)]
                .into_iter()
                .map(|corner| corner.distance(DRIVE_CENTRE))
                .fold(0.0_f32, f32::max),
            bottom: bounds.min.y,
            top: bounds.max.y,
        }
    }

    fn intersects_sweep(&self, sweep: &Self) -> bool {
        self.top > sweep.bottom - MOVING_CLEARANCE_METRES
            && self.bottom < sweep.top + MOVING_CLEARANCE_METRES
            && self.inner < sweep.outer + MOVING_CLEARANCE_METRES
            && self.outer > sweep.inner.max(PIVOT_JOINT_RADIUS_METRES) - MOVING_CLEARANCE_METRES
    }
}

pub(in super::super) fn audit_circuit(plan: &BuildingPlan, issues: &mut Vec<AuditIssue>) {
    let Some(work) = plan
        .workplace
        .as_ref()
        .filter(|work| work.kind == WorkplaceKind::HorseMill)
    else {
        return;
    };
    let moving = work
        .parts
        .iter()
        .filter(|part| part.feature == WorkplaceFeature::MillSweep)
        .map(|part| part.solid)
        .collect::<Vec<_>>();
    let sweeps = plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|solid| moving.contains(&solid.id))
        .map(SweptBand::from_solid)
        .collect::<Vec<_>>();
    for sweep in &sweeps {
        if sweep.bottom < ANIMAL_HEADROOM_METRES
            && (sweep.inner < ANIMAL_TRACK_INNER_METRES || sweep.outer > ANIMAL_TRACK_OUTER_METRES)
        {
            issues.push(AuditIssue {
                code: "horse_mill_draw_link_outside_track",
                message: "low rotating draw linkage leaves the reserved animal circuit".to_owned(),
            });
        }
    }
    for solid in &plan.resolved_geometry.solids {
        if moving.contains(&solid.id) {
            continue;
        }
        let band = SweptBand::from_solid(solid);
        if band.top > TRACK_GROUND_CLEARANCE_METRES
            && band.bottom < ANIMAL_HEADROOM_METRES
            && band.inner < ANIMAL_TRACK_OUTER_METRES
            && band.outer > ANIMAL_TRACK_INNER_METRES
        {
            issues.push(AuditIssue {
                code: "horse_mill_track_obstruction",
                message: format!("solid {} intrudes into the animal circuit", solid.id.0),
            });
        }
        if sweeps.iter().any(|sweep| band.intersects_sweep(sweep)) {
            issues.push(AuditIssue {
                code: "horse_mill_sweep_obstruction",
                message: format!(
                    "solid {} blocks the drive sweep's full rotation",
                    solid.id.0
                ),
            });
        }
    }
}
