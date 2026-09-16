use super::issue;
use crate::{AuditIssue, BuildingArchetype, BuildingPlan, WallSourceId};

pub(super) fn expected_wall_count(plan: &BuildingPlan) -> usize {
    if let Some(workplace) = &plan.workplace {
        workplace.walls.len()
    } else if let Some(church) = &plan.small_church {
        plan.storeys
            .iter()
            .map(|storey| storey.walls.len())
            .sum::<usize>()
            + church.bearing_walls.len()
    } else if let Some(church) = &plan.church {
        usize::from(church.program.nave_bays) * 2
            + usize::from(church.program.nave_bays) * 2
            + usize::from(church.program.choir_bays) * 2
            + 8
            + usize::from(church.program.apse_sides)
            + 8
            + 4
    } else {
        plan.storeys
            .iter()
            .map(|storey| storey.walls.len())
            .sum::<usize>()
            + if matches!(
                plan.archetype,
                BuildingArchetype::CastleGatehouse
                    | BuildingArchetype::CourtyardCastle
                    | BuildingArchetype::WalledKeep
                    | BuildingArchetype::ArtilleryRondelCastle
            ) {
                plan.towers.len()
            } else {
                0
            }
            + plan
                .square_towers
                .iter()
                .filter(|tower| tower.bell_openings)
                .count()
                * 8
            + if plan.archetype == BuildingArchetype::Cathedral {
                2
            } else {
                0
            }
            + plan.roof_dormers.len()
            + plan
                .roof_assemblies
                .iter()
                .flat_map(|roof| &roof.enclosure_faces)
                .map(|face| face.inset_walls.len())
                .sum::<usize>()
            + plan
                .artillery_castle
                .as_ref()
                .map_or(0, |castle| castle.stations.len())
    }
}

pub(super) fn audit_opening_count(plan: &BuildingPlan, issues: &mut Vec<AuditIssue>) {
    let source_openings = plan
        .storeys
        .iter()
        .map(|storey| storey.openings.len())
        .sum::<usize>();
    let replaced_openings = plan
        .wall_assemblies
        .iter()
        .filter(|wall| wall.replaced_by_owner.is_some())
        .filter(|wall| match wall.source {
            WallSourceId::StoreyWall {
                storey_level,
                wall_index,
            } => plan
                .storeys
                .get(storey_level as usize)
                .is_some_and(|storey| {
                    storey
                        .openings
                        .iter()
                        .any(|opening| opening.wall == wall_index)
                }),
            _ => false,
        })
        .count();
    let bell_openings = plan
        .square_towers
        .iter()
        .filter(|tower| tower.bell_openings)
        .count()
        * 8;
    let roof_openings = plan.roof_dormers.len()
        + plan
            .roof_assemblies
            .iter()
            .flat_map(|roof| &roof.enclosure_faces)
            .map(|face| face.inset_walls.len())
            .sum::<usize>();
    let church_portals = usize::from(plan.church.is_some()) * 2;
    let church_windows = plan.church.as_ref().map_or(0, |church| {
        usize::from(church.program.nave_bays) * 4
            + usize::from(church.program.choir_bays) * 2
            + 2
            + usize::from(church.program.apse_sides.saturating_sub(1))
    });
    let artillery_openings = plan
        .artillery_castle
        .as_ref()
        .map_or(0, |castle| castle.stations.len());
    if plan.opening_assemblies.len() + replaced_openings
        != source_openings
            + bell_openings
            + roof_openings
            + church_portals
            + church_windows
            + artillery_openings
    {
        issues.push(issue(
            "legacy_opening_not_migrated",
            format!(
                "resolved {} of {} openings",
                plan.opening_assemblies.len(),
                source_openings
                    + bell_openings
                    + roof_openings
                    + church_portals
                    + church_windows
                    + artillery_openings
            ),
        ));
    }
}
