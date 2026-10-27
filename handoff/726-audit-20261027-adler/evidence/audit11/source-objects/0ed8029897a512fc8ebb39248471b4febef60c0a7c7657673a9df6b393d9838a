//! Admission of scene-supported tactical entities precedes command publication.

use adventuresim_building_generator::spatial_geometry::GeometryError;
use adventuresim_tactical_core::{
    prelude::{CharacterId, SceneTerrain},
    scene::TerrainSupportError,
};
use bevy::prelude::Query;

pub(crate) type SceneSpawnResult<T> = Result<T, SceneSpawnError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum SceneSpawnError {
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error(transparent)]
    Equipment(#[from] crate::player_projection::projection_equipment::EquipmentProjectionError),
    #[error(transparent)]
    Support(#[from] TerrainSupportError),
    #[error("connected character {character:?} has invalid body mass: {cause:?}")]
    BodyMass {
        character: CharacterId,
        cause: adventuresim_core::physiology::BodyMassError,
    },
    #[error("multiple authoritative terrains cannot own one tactical spawn batch")]
    AmbiguousTerrain,
    #[error("seeded enemy {character:?} is absent from the loaded enemy fixture")]
    MissingEnemyFixture { character: CharacterId },
}

pub(crate) fn authoritative_terrain<'a>(
    terrains: &'a Query<&SceneTerrain>,
) -> SceneSpawnResult<Option<&'a SceneTerrain>> {
    let mut terrains = terrains.iter();
    let first = terrains.next();
    if terrains.next().is_some() {
        return Err(SceneSpawnError::AmbiguousTerrain);
    }
    Ok(first)
}
