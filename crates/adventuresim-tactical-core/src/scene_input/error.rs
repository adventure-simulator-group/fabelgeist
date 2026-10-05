//! Scene input failures preserve bounded support diagnostics without large stack results.
use thiserror::Error;
#[derive(Debug, Error)]
pub enum SceneInputError {
    #[error("scene input I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("scene input JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("scene input is invalid: {0}")]
    Validation(String),
    #[error(transparent)]
    TerrainGrade(#[from] crate::scene::TerrainGradeError),
    #[error(transparent)]
    BoundarySupport(#[from] Box<crate::city_layout::grounding::BoundarySupportError>),
    #[error(transparent)]
    GardenSupport(#[from] super::GardenSupportError),
    #[error(transparent)]
    Grounding(#[from] Box<crate::city_layout::CityGroundingError>),
    #[error(transparent)]
    GroundingProjection(#[from] Box<crate::city_layout::CityGroundingProjectionError>),
    #[error(transparent)]
    SupportTerrain(#[from] Box<crate::city_layout::grounding::SettlementSupportError>),
}

impl From<crate::city_layout::CityGroundingError> for SceneInputError {
    fn from(error: crate::city_layout::CityGroundingError) -> Self {
        Self::Grounding(Box::new(error))
    }
}

impl From<crate::city_layout::CityGroundingProjectionError> for SceneInputError {
    fn from(error: crate::city_layout::CityGroundingProjectionError) -> Self {
        Self::GroundingProjection(Box::new(error))
    }
}

impl From<crate::city_layout::grounding::SettlementSupportError> for SceneInputError {
    fn from(error: crate::city_layout::grounding::SettlementSupportError) -> Self {
        Self::SupportTerrain(Box::new(error))
    }
}

impl From<crate::city_layout::grounding::BoundarySupportError> for SceneInputError {
    fn from(error: crate::city_layout::grounding::BoundarySupportError) -> Self {
        Self::BoundarySupport(Box::new(error))
    }
}
