//! Scene input failures preserve bounded support diagnostics without large stack results.
use thiserror::Error;
/// Scene input validation and generation retain the owning admission error.
pub type SceneInputResult<T> = std::result::Result<T, SceneInputError>;

#[derive(Debug, Error)]
pub enum SceneInputError {
    #[error(transparent)]
    BoundaryAdmission(#[from] crate::city_layout::grounding::enclosure::BoundaryAdmissionError),
    #[error(transparent)]
    SupportCollider(#[from] crate::city_layout::grounding::SupportColliderError),
    #[error(transparent)]
    TerrainAdmission(#[from] crate::scene::TerrainAdmissionError),
    #[error(transparent)]
    BoundaryGeometry(#[from] crate::city_layout::BoundaryGeometryError),
    #[error("building {building_id} interior: {cause}")]
    Interior {
        building_id: crate::scene_input::SceneBuildingId,
        #[source]
        cause: adventuresim_building_generator::interior::InteriorLayoutError,
    },
    #[error(transparent)]
    Furniture(#[from] adventuresim_building_generator::furniture::FurnitureRecipeError),
    #[error(transparent)]
    Geometry(#[from] adventuresim_building_generator::spatial_geometry::GeometryError),
    #[error(transparent)]
    Collision(#[from] adventuresim_building_generator::CollisionError),
    #[error(transparent)]
    Door(#[from] adventuresim_building_generator::DoorError),
    #[error(transparent)]
    Entrance(#[from] adventuresim_building_generator::EntranceError),
    #[error("scene input I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("scene input JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("scene input is invalid: {0}")]
    Validation(#[from] super::SceneValidationError),
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
