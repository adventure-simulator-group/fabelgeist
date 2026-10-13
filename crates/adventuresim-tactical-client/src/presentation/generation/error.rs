//! Preparation failures retain their owning causes until the presentation boundary.
use adventuresim_building_generator::{
    GenerationError, interior::InteriorLayoutError, spatial_geometry::GeometryError,
};
use adventuresim_tactical_core::scene_input::{SceneBuildingId, SceneInputError};
use thiserror::Error;

pub(crate) type PreparationResult<T> = std::result::Result<T, PreparationError>;

/// Asset roles shared by dependency, residency and installation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProductKind {
    RegionalCity,
    Scene,
    Facade,
    Venue,
    VenueGeometry,
    Grass,
    Ground,
}

#[derive(Debug, Error)]
pub(crate) enum PreparationError {
    #[error("preparation identity sequence is exhausted")]
    SequenceExhausted,
    #[error("preparation no longer owns its product slot")]
    StalePreparation,
    #[error("preparation belongs to another presentation owner")]
    PreparationOwner,
    #[error("completed preparation belongs to another scene document")]
    PreparationInputMismatch,
    #[error("generation product residency is poisoned")]
    ResidencyPoisoned,
    #[error("generation JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("generation dependencies cannot be decoded: {0}")]
    DependenciesDecode(#[source] ciborium::de::Error<std::io::Error>),
    #[error("generation product cannot be decoded: {0}")]
    ProductDecode(#[source] ciborium::de::Error<std::io::Error>),
    #[error("generation dependencies cannot be encoded: {0}")]
    DependenciesEncode(#[source] ciborium::ser::Error<std::io::Error>),
    #[error("generation product cannot be encoded: {0}")]
    ProductEncode(#[source] ciborium::ser::Error<std::io::Error>),
    #[error(transparent)]
    SceneInput(#[from] Box<SceneInputError>),
    #[error(transparent)]
    Building(#[from] Box<GenerationError>),
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error(transparent)]
    Interior(#[from] Box<InteriorLayoutError>),
    // The graphics parser currently exposes prose at its external YAML boundary.
    // Keep that boundary classified here without branching on its message.
    #[error("generation graphics configuration is invalid: {message}")]
    GraphicsConfiguration { message: String },
    #[error("{product:?} was not prepared before use")]
    NotPrepared { product: ProductKind },
    #[error("generation job is missing its {product:?} dependencies")]
    MissingDependencies { product: ProductKind },
    #[error("generation product does not match its requested input")]
    ProductMismatch,
    #[error("scene transfer changed its exact occupied building bindings")]
    SceneBindingsMismatch,
    #[error("promoted venue {building} has no distant placement")]
    MissingDistantPlacement { building: SceneBuildingId },
    #[error("promoted venue {building} changed its physical placement or program")]
    PromotedPlacementMismatch { building: SceneBuildingId },
}

impl PreparationError {
    /// Stable browser failure classification; display prose remains diagnostic.
    #[cfg(target_family = "wasm")]
    pub(super) fn code(&self) -> &'static str {
        match self {
            Self::SequenceExhausted => "generation/sequence-exhausted",
            Self::StalePreparation => "generation/stale-preparation",
            Self::PreparationOwner => "generation/preparation-owner",
            Self::PreparationInputMismatch => "generation/preparation-input",
            Self::ResidencyPoisoned => "generation/residency-poisoned",
            Self::Json(_) => "generation/json",
            Self::DependenciesDecode(_) => "generation/dependencies-decode",
            Self::ProductDecode(_) => "generation/product-decode",
            Self::DependenciesEncode(_) => "generation/dependencies-encode",
            Self::ProductEncode(_) => "generation/product-encode",
            Self::SceneInput(_) => "generation/scene-input",
            Self::Building(_) => "generation/building",
            Self::Geometry(_) => "generation/geometry",
            Self::Interior(_) => "generation/interior",
            Self::GraphicsConfiguration { .. } => "generation/graphics-configuration",
            Self::NotPrepared { .. } => "generation/not-prepared",
            Self::MissingDependencies { .. } => "generation/missing-dependencies",
            Self::ProductMismatch => "generation/product-mismatch",
            Self::SceneBindingsMismatch => "generation/scene-bindings-mismatch",
            Self::MissingDistantPlacement { .. } => "generation/missing-distant-placement",
            Self::PromotedPlacementMismatch { .. } => "generation/promoted-placement-mismatch",
        }
    }
}

impl From<SceneInputError> for PreparationError {
    fn from(error: SceneInputError) -> Self {
        Self::SceneInput(Box::new(error))
    }
}

impl From<GenerationError> for PreparationError {
    fn from(error: GenerationError) -> Self {
        Self::Building(Box::new(error))
    }
}

impl From<InteriorLayoutError> for PreparationError {
    fn from(error: InteriorLayoutError) -> Self {
        Self::Interior(Box::new(error))
    }
}
