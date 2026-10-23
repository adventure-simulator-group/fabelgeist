//! Required vista geometry fails before replacing its prepared presentation.
use adventuresim_building_generator::spatial_geometry::GeometryError;
use adventuresim_tactical_core::{
    scene::TerrainSupportError, scene_coordinates::ScenePlanPoint, scene_input::VistaLevelIndex,
};

pub(crate) type VistaMeshResult<T> = Result<T, VistaMeshError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum VistaMeshError {
    #[error("vista level {level} has invalid grid metadata")]
    InvalidLod { level: VistaLevelIndex },
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error(transparent)]
    Support(#[from] TerrainSupportError),
    #[error("vista vertex at {point:?} has no finite surface normal")]
    DegenerateNormal { point: ScenePlanPoint },
    #[error("vista vertex at {point:?} has no source pigment")]
    MissingColor { point: ScenePlanPoint },
}
