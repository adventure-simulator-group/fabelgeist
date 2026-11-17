//! Operator stations retain the occupied building's proven furniture access.

use std::collections::BTreeSet;

use adventuresim_building_generator::{
    StoreyIndex,
    furniture::{FurnitureAccessFace, FurnitureKind},
    interior::{FurnitureAccessPath, InteriorLayoutError, InteriorPlacement},
    spatial_geometry::{Architectural, GeometryError, Position},
};
use adventuresim_tactical_core::{
    player::CharacterId,
    scene_coordinates::{ArchitecturalPlanProjection, Scene, ScenePlanPoint},
    scene_input::{
        GeneratedBuilding, SceneBuildingId, SceneEstablishment, furniture::InteriorBuildingLayout,
    },
};
use adventuresim_world_schema::settlement_buildings::BusinessId;
use bevy::prelude::*;

pub(crate) type PlacementResult<T> = Result<T, ResidentPlacementError>;

#[derive(Resource, Default)]
pub(crate) struct OperatorStations(pub(crate) Vec<OperatorStation>);

pub(crate) struct OperatorStation {
    pub(crate) character: CharacterId,
    pub(crate) business: BusinessId,
    pub(crate) building: SceneBuildingId,
    pub(crate) approaches: Vec<ResidentApproach>,
}

pub(crate) struct ResidentApproach {
    pub(crate) arrival: Position<Scene>,
    pub(crate) via: Vec<ResidentRouteWaypoint>,
    role: StationRole,
}

/// Door thresholds are circulation references, not assertions that a building
/// floor slab supports an actor there. Preserve their planar intent and storey.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResidentRouteWaypoint {
    pub(crate) position: ScenePlanPoint,
    pub(crate) storey: StoreyIndex,
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum StationRole {
    CounterOperator,
    Workstation,
    Interior,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ResidentPlacementError {
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error("resident building {building}: {cause}")]
    Interior {
        building: SceneBuildingId,
        #[source]
        cause: InteriorLayoutError,
    },
    #[error("resident building {building}, waypoint {waypoint:?}: {cause}")]
    UnsupportedWaypoint {
        building: SceneBuildingId,
        waypoint: adventuresim_building_generator::interior::InteriorWaypoint,
        #[source]
        cause: InteriorLayoutError,
    },
    #[error("resident building {building} has an invalid furniture access path")]
    InvalidAccess { building: SceneBuildingId },
    #[error("resident {character:?} is assigned to more than one playable business")]
    DuplicateOperator { character: CharacterId },
}

impl OperatorStations {
    /// Distant establishments remain presentation. Only occupied, physically
    /// compiled buildings can supply a live operator station.
    pub(crate) fn prepare(
        buildings: &[GeneratedBuilding],
        interiors: &[InteriorBuildingLayout],
        establishments: &[SceneEstablishment],
    ) -> PlacementResult<Self> {
        let mut stations = Vec::new();
        let mut characters = BTreeSet::new();
        for establishment in establishments {
            let Some(building) = buildings
                .iter()
                .find(|building| building.placement.id == establishment.building_id)
            else {
                continue;
            };
            if !characters.insert(establishment.operator_character_id) {
                return Err(ResidentPlacementError::DuplicateOperator {
                    character: establishment.operator_character_id,
                });
            }
            let Some(interior) = interiors
                .iter()
                .find(|interior| interior.building_id == establishment.building_id)
            else {
                continue;
            };
            let mut approaches = Vec::new();
            let paths = adventuresim_building_generator::interior::validate_layout(
                &building.plan,
                &interior.layout,
            )
            .map_err(|cause| ResidentPlacementError::Interior {
                building: building.placement.id,
                cause,
            })?;
            for path in &paths {
                let placement = interior.layout.placements.get(path.placement_index).ok_or(
                    ResidentPlacementError::InvalidAccess {
                        building: building.placement.id,
                    },
                )?;
                if placement.storey != StoreyIndex::GROUND {
                    continue;
                }
                approaches.push(ResidentApproach::from_furniture(building, path, placement)?);
            }
            approaches.sort_by_key(|approach| approach.role);
            // A business without an admitted physical station cannot advertise
            // a live operator. It is never replaced by an arbitrary centre point.
            if approaches.is_empty() {
                continue;
            }
            stations.push(OperatorStation {
                character: establishment.operator_character_id,
                business: establishment.business_id.clone(),
                building: establishment.building_id,
                approaches,
            });
        }
        stations.sort_by_key(|station| station.character);
        Ok(Self(stations))
    }
}

impl ResidentApproach {
    fn from_furniture(
        building: &GeneratedBuilding,
        path: &FurnitureAccessPath,
        placement: &InteriorPlacement,
    ) -> PlacementResult<Self> {
        let (destination, via) =
            path.points
                .split_last()
                .ok_or(ResidentPlacementError::InvalidAccess {
                    building: building.placement.id,
                })?;
        let height = destination.floor_height(&building.plan).map_err(|cause| {
            ResidentPlacementError::UnsupportedWaypoint {
                building: building.placement.id,
                waypoint: *destination,
                cause,
            }
        })?;
        let point = destination.position_metres.metres();
        let arrival = building
            .geometry_datum()?
            .point(Position::<Architectural>::from_metres(Vec3::new(
                point.x,
                height.metres(),
                point.y,
            ))?)?;
        let projection = ArchitecturalPlanProjection::from_placement(
            &building.placement,
            building.collision.bounds,
        )?;
        let via = via
            .iter()
            .map(|waypoint| {
                Ok(ResidentRouteWaypoint {
                    position: projection.point(waypoint.position_metres)?,
                    storey: waypoint.storey,
                })
            })
            .collect::<std::result::Result<Vec<_>, GeometryError>>()?;
        let role = match (placement.key.kind(), path.face) {
            (
                FurnitureKind::Counter
                | FurnitureKind::CounterLeftEnd
                | FurnitureKind::CounterRightEnd
                | FurnitureKind::CounterCorner
                | FurnitureKind::DisplayCounter,
                FurnitureAccessFace::Back,
            ) => StationRole::CounterOperator,
            (FurnitureKind::Workbench, _) => StationRole::Workstation,
            _ => StationRole::Interior,
        };
        Ok(Self { arrival, via, role })
    }
}

#[cfg(test)]
pub(super) mod tests;
