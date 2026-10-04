//! Ground entrances project exact openings and authored workplace passages.
use crate::*;
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

/// A workplace passage face remains distinct from an architectural opening.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassageEntranceSide {
    Front,
    Back,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingEntranceId {
    Opening(OpeningAssemblyId),
    Workplace {
        passage: WorkplacePassageId,
        side: PassageEntranceSide,
    },
}

/// An exterior entrance in architectural X/Z coordinates. Closure state does
/// not decide whether the entrance exists. Its floor retains the plan datum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingEntranceSupport {
    ArchitecturalFloor,
    NaturalTerrain,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BuildingEntrance {
    pub id: BuildingEntranceId,
    pub support: BuildingEntranceSupport,
    pub threshold_metres: Vec2,
    pub outward: Vec2,
}

pub fn compile_ground_entrances(plan: &BuildingPlan) -> Vec<BuildingEntrance> {
    let mut entries: Vec<_> = plan
        .opening_assemblies
        .iter()
        .filter(|opening| {
            matches!(opening.use_kind, OpeningUse::Door | OpeningUse::Gate)
                && opening.frame.outside_room.is_none()
                && plan
                    .wall_assemblies
                    .iter()
                    .any(|wall| wall.id == opening.host_wall && wall.storey_level == 0)
        })
        .map(|opening| BuildingEntrance {
            id: BuildingEntranceId::Opening(opening.id),
            support: BuildingEntranceSupport::ArchitecturalFloor,
            threshold_metres: opening.frame.origin,
            outward: opening.frame.outward,
        })
        .collect();
    if let Some(workplace) = &plan.workplace {
        // The passage reservation is the authored clear route. A face touching
        // the plot perimeter projects an exterior endpoint; interior cross
        // aisles do not acquire invented door identities or outward directions.
        for passage in &workplace.passages {
            let support = match passage.purpose {
                WorkplacePassagePurpose::GroundFloorCirculation => {
                    BuildingEntranceSupport::ArchitecturalFloor
                }
                WorkplacePassagePurpose::OutdoorRoute => BuildingEntranceSupport::NaturalTerrain,
                WorkplacePassagePurpose::UpperCirculation
                | WorkplacePassagePurpose::ServiceClearance => continue,
            };
            let centre = (passage.min + passage.max) * 0.5;
            let limits = workplace.plot_dimensions_metres;
            for (side, boundary, limit, threshold, outward) in [
                (
                    PassageEntranceSide::Front,
                    passage.min.z,
                    0.0,
                    Vec2::new(centre.x, passage.min.z),
                    -Vec2::Y,
                ),
                (
                    PassageEntranceSide::Back,
                    passage.max.z,
                    limits.y,
                    Vec2::new(centre.x, passage.max.z),
                    Vec2::Y,
                ),
                (
                    PassageEntranceSide::Left,
                    passage.min.x,
                    0.0,
                    Vec2::new(passage.min.x, centre.z),
                    -Vec2::X,
                ),
                (
                    PassageEntranceSide::Right,
                    passage.max.x,
                    limits.x,
                    Vec2::new(passage.max.x, centre.z),
                    Vec2::X,
                ),
            ] {
                if boundary == limit {
                    entries.push(BuildingEntrance {
                        id: BuildingEntranceId::Workplace {
                            passage: passage.id,
                            side,
                        },
                        support,
                        threshold_metres: threshold,
                        outward,
                    });
                }
            }
        }
    }
    entries.sort_by_key(|entry| entry.id);
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_world_schema::settlement_buildings::BuildingUse;

    #[test]
    fn workplace_entrance_binds_the_authored_passage_without_an_operable_door() {
        let program = BuildingProgram::validated_settlement(
            BuildingArchetype::Workplace,
            BuildingUse::Bakehouse,
            101,
            Some(ServiceBuildingSize::Small),
        )
        .unwrap();
        let plan = generate(&program).unwrap();
        assert!(compile_operable_doors(&plan).is_empty());
        let entries = compile_ground_entrances(&plan);
        assert!(!entries.is_empty());
        let work = plan.workplace.as_ref().unwrap();
        for entry in &entries {
            let BuildingEntranceId::Workplace { passage, side } = entry.id else {
                panic!("workplace endpoint must retain its passage identity")
            };
            let route = work
                .passages
                .iter()
                .find(|route| route.id == passage)
                .unwrap();
            let (expected, outward) = match side {
                PassageEntranceSide::Front => (
                    Vec2::new((route.min.x + route.max.x) * 0.5, route.min.z),
                    -Vec2::Y,
                ),
                PassageEntranceSide::Back => (
                    Vec2::new((route.min.x + route.max.x) * 0.5, route.max.z),
                    Vec2::Y,
                ),
                _ => panic!("the recorded bakehouse has front and rear passage faces"),
            };
            assert_eq!(entry.threshold_metres, expected);
            assert_eq!(entry.outward, outward);
        }
        assert_eq!(
            entries,
            compile_ground_entrances(&generate(&program).unwrap())
        );
    }
}
