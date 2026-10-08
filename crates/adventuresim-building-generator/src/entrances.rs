//! Ground entrances project exact openings and authored workplace passages.
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{Architectural, GeometryError, PlanDirection};
use crate::*;
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
#[cfg(test)]
mod admission_tests;

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

/// Selects architectural-floor or natural-terrain support for an entrance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingEntranceSupport {
    ArchitecturalFloor,
    NaturalTerrain,
}

/// An exterior entrance in architectural X/Z coordinates. Closure state does
/// not decide whether the entrance exists.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct BuildingEntrance {
    pub id: BuildingEntranceId,
    pub support: BuildingEntranceSupport,
    pub threshold_metres: ArchitecturalPlanPoint,
    pub outward: PlanDirection<Architectural>,
}

impl<'de> Deserialize<'de> for BuildingEntrance {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct NativeEntrance {
            id: BuildingEntranceId,
            support: BuildingEntranceSupport,
            threshold_metres: Vec2,
            outward: Vec2,
        }
        let value = NativeEntrance::deserialize(d)?;
        Self::from_metres(
            value.id,
            value.support,
            value.threshold_metres,
            value.outward,
        )
        .map_err(serde::de::Error::custom)
    }
}

pub fn compile_ground_entrances(
    plan: &BuildingPlan,
) -> Result<Vec<BuildingEntrance>, EntranceError> {
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
        .map(|opening| {
            BuildingEntrance::from_metres(
                BuildingEntranceId::Opening(opening.id),
                BuildingEntranceSupport::ArchitecturalFloor,
                opening.frame.origin,
                opening.frame.outward,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
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
            let centre = (passage.bounds.min().metres() + passage.bounds.max().metres()) * 0.5;
            let limits = workplace.plot_dimensions_metres.metres();
            for (side, boundary, limit, threshold, outward) in [
                (
                    PassageEntranceSide::Front,
                    passage.bounds.min().metres().z,
                    0.0,
                    Vec2::new(centre.x, passage.bounds.min().metres().z),
                    -Vec2::Y,
                ),
                (
                    PassageEntranceSide::Back,
                    passage.bounds.max().metres().z,
                    limits.y,
                    Vec2::new(centre.x, passage.bounds.max().metres().z),
                    Vec2::Y,
                ),
                (
                    PassageEntranceSide::Left,
                    passage.bounds.min().metres().x,
                    0.0,
                    Vec2::new(passage.bounds.min().metres().x, centre.z),
                    -Vec2::X,
                ),
                (
                    PassageEntranceSide::Right,
                    passage.bounds.max().metres().x,
                    limits.x,
                    Vec2::new(passage.bounds.max().metres().x, centre.z),
                    Vec2::X,
                ),
            ] {
                if boundary == limit {
                    entries.push(BuildingEntrance::from_metres(
                        BuildingEntranceId::Workplace {
                            passage: passage.id,
                            side,
                        },
                        support,
                        threshold,
                        outward,
                    )?);
                }
            }
        }
    }
    entries.sort_by_key(|entry| entry.id);
    Ok(entries)
}

impl BuildingEntrance {
    pub fn from_metres(
        id: BuildingEntranceId,
        support: BuildingEntranceSupport,
        threshold: Vec2,
        outward: Vec2,
    ) -> Result<Self, EntranceError> {
        let construct = || {
            Ok(Self {
                id,
                support,
                threshold_metres: ArchitecturalPlanPoint::from_metres(threshold)?,
                outward: PlanDirection::from_normalized(outward)?,
            })
        };
        construct().map_err(|cause| EntranceError { id, cause })
    }
}
#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
#[error("entrance {id:?}: {cause}")]
pub struct EntranceError {
    pub id: BuildingEntranceId,
    #[source]
    pub cause: GeometryError,
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
            fabelgeist_determinism::Seed::from_u64(101),
            Some(ServiceBuildingSize::Small),
        )
        .unwrap();
        let plan = generate(&program).unwrap();
        assert!(compile_operable_doors(&plan).unwrap().is_empty());
        let entries = compile_ground_entrances(&plan).unwrap();
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
                    Vec2::new(
                        (route.bounds.min().metres().x + route.bounds.max().metres().x) * 0.5,
                        route.bounds.min().metres().z,
                    ),
                    -Vec2::Y,
                ),
                PassageEntranceSide::Back => (
                    Vec2::new(
                        (route.bounds.min().metres().x + route.bounds.max().metres().x) * 0.5,
                        route.bounds.max().metres().z,
                    ),
                    Vec2::Y,
                ),
                _ => panic!("the recorded bakehouse has front and rear passage faces"),
            };
            assert_eq!(entry.threshold_metres.metres(), expected);
            assert_eq!(entry.outward.vector(), outward);
        }
        assert_eq!(
            entries,
            compile_ground_entrances(&generate(&program).unwrap()).unwrap()
        );
    }
}
