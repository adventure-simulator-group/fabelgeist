//! Strategic scene admission retains causes until the serialized status boundary.

use crate::presentation::{
    BackdropError, preparation::PreparationError as GenerationPreparationError,
};
use adventuresim_building_generator::{GenerationError, spatial_geometry::GeometryError};
use adventuresim_tactical_core::{
    scene::TerrainSupportError,
    scene_input::{SceneBuildingId, SceneInputError},
};
use bevy::ecs::error::BevyError;
use std::sync::Arc;

pub(super) type PreparationResult<T> = Result<T, PreparationError>;
pub(super) type SharedPreparationResult<T> = Result<T, Arc<PreparationError>>;

#[derive(Debug, thiserror::Error)]
pub(super) enum PreparationError {
    #[error(transparent)]
    SceneInput(#[from] Box<SceneInputError>),
    #[error(transparent)]
    Generation(#[from] Box<GenerationPreparationError>),
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error(transparent)]
    Backdrop(#[from] BackdropError),
    #[error(transparent)]
    Support(#[from] TerrainSupportError),
    #[error("strategic presentation failed: {0}")]
    Presentation(BevyError),
    #[error("invalid doorway orientation")]
    DoorwayOrientation,
    #[error("invalid relocated furniture orientation")]
    FurnitureOrientation,
    #[error("building {building} has no clear conversation position")]
    MissingConversationPosition { building: SceneBuildingId },
    #[error("building {building} has no prepared interior")]
    MissingInterior { building: SceneBuildingId },
    #[error("building {building} has no distant placement")]
    MissingBuildingPlacement { building: SceneBuildingId },
}

impl From<SceneInputError> for PreparationError {
    fn from(error: SceneInputError) -> Self {
        Self::SceneInput(Box::new(error))
    }
}

impl From<GenerationPreparationError> for PreparationError {
    fn from(error: GenerationPreparationError) -> Self {
        Self::Generation(Box::new(error))
    }
}

impl From<GenerationError> for PreparationError {
    fn from(error: GenerationError) -> Self {
        Self::from(GenerationPreparationError::from(error))
    }
}

impl From<BevyError> for PreparationError {
    fn from(error: BevyError) -> Self {
        Self::Presentation(error)
    }
}
