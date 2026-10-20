//! Source observations at compiled structural bearings and external thresholds.
use super::source::InspectionSource;
use adventuresim_building_generator::{
    BuildingLodLevel, BuildingPlan, DoorError, OpeningAssemblyId, compile_operable_doors,
    spatial_geometry::{GeometryError, GeometryResult},
};
use adventuresim_tactical_core::{
    city_layout::grounding::SupportElevation,
    scene_coordinates::ScenePlanPoint,
};
use bevy::math::{Vec2, Vec3Swizzles};
use serde_json::{Value, json};

type ObservationResult<T> = Result<T, ObservationError>;

#[derive(Debug, thiserror::Error)]
pub(super) enum ObservationError {
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error(transparent)]
    Door(#[from] DoorError),
    #[error("compiled door refers to missing source opening {opening:?}")]
    MissingOpening { opening: OpeningAssemblyId },
    #[error("computed world elevation is not finite for {level:?}")]
    InvalidLodElevation { level: BuildingLodLevel },
}

// `world` completes the building-local to scene-plane numerical projection.
pub(super) fn grounded_nodes(
    plan: &BuildingPlan,
    world: &impl Fn(Vec2) -> GeometryResult<ScenePlanPoint>,
    elevation: SupportElevation,
    source: &InspectionSource<'_>,
) -> ObservationResult<Vec<Value>> {
    plan.resolved_geometry.structural_nodes.iter()
        .filter(|node| node.grounded)
        .map(|node| {
            let point = world(node.position.metres().xz())?;
            Ok(json!({
                "node": node.id, "kind": node.kind, "point": point,
                "world_elevation": node.position.metres().y + elevation.metres(),
                "terrain_elevation": source.height_at(point),
            }))
        })
        .collect()
}

pub(super) fn exterior_thresholds(
    plan: &BuildingPlan,
    world: &impl Fn(Vec2) -> GeometryResult<ScenePlanPoint>,
    elevation: SupportElevation,
    source: &InspectionSource<'_>,
) -> ObservationResult<Vec<Value>> {
    compile_operable_doors(plan)?.iter()
        .map(|door| {
            let opening = plan.opening_assemblies.iter()
                .find(|opening| opening.id == door.opening)
                .ok_or(ObservationError::MissingOpening { opening: door.opening })?;
            let point = world(door.closed_centre.metres().xz())?;
            Ok(json!({"id":opening.id, "point":point,
                "sill_elevation":opening.sill_elevation_metres + elevation.metres(),
                "terrain_elevation":source.height_at(point)}))
        })
        .collect()
}
