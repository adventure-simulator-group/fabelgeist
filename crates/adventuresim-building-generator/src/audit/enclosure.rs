//! Required enclosure interfaces are inferred before examining generated bonds.
use bevy::math::{Vec2, Vec3};

use crate::{
    AuditIssue, BuildingPlan, CELL_SIZE_METRES, SolidRole, WallAssembly, WallSegment, WallSourceId,
};

use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{Architectural, Elevation, PlanDirection, Position, PositiveLength};

struct CornerGap {
    position: Position<Architectural>,
    walls: [crate::WallAssemblyId; 2],
}

use super::{Result, enclosure_sections, issue};

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

pub(super) fn audit(plan: &BuildingPlan, issues: &mut Vec<AuditIssue>) -> Result<()> {
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
                if let Some(gap) =
                    corner_gap(plan, ArchitecturalPlanPoint::from_metres(corner)?, a, b)?
                {
                    issues.push(issue(
                        WALL_GAP,
                        format!(
                            "walls {} and {} leave an exterior corner open at {:?}",
                            gap.walls[0].0,
                            gap.walls[1].0,
                            gap.position.metres()
                        ),
                    ));
                }
            }
        }
    }
    super::gable_enclosure::audit(plan, issues)?;

    Ok(())
}

fn corner_gap(
    plan: &BuildingPlan,
    corner: ArchitecturalPlanPoint,
    a: &WallAssembly,
    b: &WallAssembly,
) -> Result<Option<CornerGap>> {
    let wall_normals = [
        PlanDirection::<Architectural>::from_normalized(a.frame.outward)?,
        PlanDirection::<Architectural>::from_normalized(b.frame.outward)?,
    ];
    let outward = wall_normals[0].vector() + wall_normals[1].vector();
    let projection = plan.upper_storey_projection_metres * f32::from(a.storey_level.min(1));
    let centre = corner.metres() + outward * projection;
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
        let mut intervals = Vec::new();
        for solid in &solids {
            intervals.extend(enclosure_sections::intervals(
                plan,
                solid,
                Position::from_metres(point)?,
                wall_normals,
                PositiveLength::from_metres(depth)?,
                Elevation::from_metres(base)?,
                Elevation::from_metres(top)?,
            )?);
        }
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
                    enclosure_sections::ElevationInterval::from_metres(
                        opening.sill_elevation_metres,
                        opening.sill_elevation_metres + opening.profile.clear_height_metres(),
                    )
                })
                .collect::<Result<Vec<_>>>()?,
        );
        intervals.sort_by(|a, b| a.low.metres().total_cmp(&b.low.metres()));
        let mut covered_to = base;
        for interval in intervals {
            let (low, high) = (interval.low.metres(), interval.high.metres());
            if low > covered_to + JUNCTION_TOLERANCE_METRES {
                return Ok(Some(CornerGap {
                    position: Position::from_metres(point + Vec3::Y * ((low + covered_to) * 0.5))?,
                    walls: [a.id, b.id],
                }));
            }
            covered_to = covered_to.max(high);
        }
        if covered_to < top - JUNCTION_TOLERANCE_METRES {
            return Ok(Some(CornerGap {
                position: Position::from_metres(point + Vec3::Y * ((top + covered_to) * 0.5))?,
                walls: [a.id, b.id],
            }));
        }
    }
    Ok(None)
}
