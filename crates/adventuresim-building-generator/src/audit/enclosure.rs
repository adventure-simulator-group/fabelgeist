//! Required enclosure interfaces are inferred before examining generated bonds.
use bevy::math::{Vec2, Vec3};

use crate::{
    AuditIssue, BuildingPlan, CELL_SIZE_METRES, SolidRole, WallAssembly, WallSegment, WallSourceId,
};

use super::{enclosure_sections, issue};

pub(crate) const WALL_GAP: &str = "wall_corner_enclosure_gap";
pub(crate) const GABLE_GAP: &str = "roof_gable_enclosure_gap";
const JUNCTION_TOLERANCE_METRES: f32 = 0.001;
const SECTION_CLEARANCE_METRES: f32 = 0.005;

fn endpoints(wall: WallSegment) -> [Vec2; 2] {
    let tangent = if wall.is_horizontal() {
        Vec2::X
    } else {
        Vec2::Y
    };
    [-1.0, 1.0].map(|sign| wall.centre() + tangent * sign * CELL_SIZE_METRES * 0.5)
}

fn resolved_wall(plan: &BuildingPlan, level: u16, index: usize) -> Option<&WallAssembly> {
    plan.wall_assemblies.iter().find(|wall| {
        wall.source
            == WallSourceId::StoreyWall {
                storey_level: level,
                wall_index: index,
            }
            && wall.replaced_by_owner.is_none()
    })
}

pub(super) fn audit(plan: &BuildingPlan, issues: &mut Vec<AuditIssue>) {
    for storey in &plan.storeys {
        for (index, left) in storey
            .walls
            .iter()
            .enumerate()
            .filter(|(_, wall)| wall.exterior())
        {
            for (right_index, right) in storey
                .walls
                .iter()
                .enumerate()
                .skip(index + 1)
                .filter(|(_, wall)| wall.exterior() && wall.is_horizontal() != left.is_horizontal())
            {
                let corner = endpoints(*left).into_iter().find(|point| {
                    endpoints(*right)
                        .into_iter()
                        .any(|other| point.distance(other) < JUNCTION_TOLERANCE_METRES)
                });
                let (Some(corner), Some(a), Some(b)) = (
                    corner,
                    resolved_wall(plan, storey.level, index),
                    resolved_wall(plan, storey.level, right_index),
                ) else {
                    continue;
                };
                if let Some(point) = corner_gap(plan, corner, a, b) {
                    issues.push(issue(
                        WALL_GAP,
                        format!(
                            "walls {} and {} leave an exterior corner open at {point:?}",
                            a.id.0, b.id.0
                        ),
                    ));
                }
            }
        }
    }
    super::gable_enclosure::audit(plan, issues);
}

fn corner_gap(
    plan: &BuildingPlan,
    corner: Vec2,
    a: &WallAssembly,
    b: &WallAssembly,
) -> Option<Vec3> {
    let outward = a.frame.outward + b.frame.outward;
    let projection = plan.upper_storey_projection_metres * f32::from(a.storey_level.min(1));
    let centre = corner + outward * projection;
    let direction = Vec3::new(outward.x, 0.0, outward.y);
    let transverse = Vec3::new(-outward.y, 0.0, outward.x);
    let depth = a.thickness_metres.max(b.thickness_metres);
    let base = a.base_elevation_metres.max(b.base_elevation_metres);
    let top =
        (a.base_elevation_metres + a.height_metres).min(b.base_elevation_metres + b.height_metres);
    let solids = plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|solid| {
            a.host_solids.contains(&solid.id)
                || b.host_solids.contains(&solid.id)
                || solid.role == SolidRole::FramePost
        })
        .collect::<Vec<_>>();
    for offset in [-SECTION_CLEARANCE_METRES, 0.0, SECTION_CLEARANCE_METRES] {
        let point = Vec3::new(centre.x, 0.0, centre.y) + transverse * offset;
        let mut intervals = solids
            .iter()
            .flat_map(|solid| {
                enclosure_sections::intervals(plan, solid, point, direction, depth, base, top)
            })
            .collect::<Vec<_>>();
        // Apertures explicitly declare the heights at which enclosure is absent.
        intervals.extend(
            plan.opening_assemblies
                .iter()
                .filter(|opening| {
                    [a.id, b.id].contains(&opening.host_wall)
                        && (centre - opening.frame.origin)
                            .dot(opening.frame.tangent)
                            .abs()
                            <= opening.profile.interior_width_metres() * 0.5
                })
                .map(|opening| {
                    (
                        opening.sill_elevation_metres,
                        opening.sill_elevation_metres + opening.profile.clear_height_metres(),
                    )
                }),
        );
        intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut covered_to = base;
        for (low, high) in intervals {
            if low > covered_to + JUNCTION_TOLERANCE_METRES {
                return Some(point + Vec3::Y * ((low + covered_to) * 0.5));
            }
            covered_to = covered_to.max(high);
        }
        if covered_to < top - JUNCTION_TOLERANCE_METRES {
            return Some(point + Vec3::Y * ((top + covered_to) * 0.5));
        }
    }
    None
}
