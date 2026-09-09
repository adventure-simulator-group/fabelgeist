use crate::{BuildingArchetype, BuildingPlan};

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
                .artillery_castle
                .as_ref()
                .map_or(0, |castle| castle.stations.len())
    }
}
