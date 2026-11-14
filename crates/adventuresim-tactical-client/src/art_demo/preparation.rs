//! Exhibit preparation keeps owned causes through the retained loading state.
use adventuresim_tactical_core::{
    city_layout::grounding::SupportColliderError, scene_input::SceneInputError,
};
use std::sync::Arc;

pub(super) type ArtDemoPreparationResult<T> = Result<T, ArtDemoPreparationError>;
pub(super) type SharedArtDemoResult<T> = Result<T, Arc<ArtDemoPreparationError>>;

#[derive(Debug, thiserror::Error)]
pub(super) enum ArtDemoPreparationError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    SceneInput(#[from] Box<SceneInputError>),
    #[error(transparent)]
    Collider(#[from] SupportColliderError),
    #[error(transparent)]
    Weapon(#[from] adventuresim_weapon_model::GenerateError),
    #[error("prepared city uses a different scene format; regenerate the city assets")]
    CityFormatMismatch,
    #[error("prebaked cloud asset belongs to scene {baked}, not {scene}")]
    CloudSceneMismatch { baked: &'static str, scene: String },
    #[error("woodland fixture contains no oak specimen")]
    MissingOak,
    #[error("model exhibit requires an asset")]
    MissingModelAsset,
    #[error("weapon exhibit requires a chassis")]
    MissingWeaponAsset,
    #[error("unknown weapon chassis {chassis}")]
    UnknownWeaponChassis { chassis: String },
    #[error("oak impostor asset was not requested")]
    ImpostorNotRequested,
    #[error("prepared oak impostors failed: {source}")]
    ImpostorLoad {
        #[source]
        source: Arc<bevy::asset::AssetLoadError>,
    },
}

impl From<SceneInputError> for ArtDemoPreparationError {
    fn from(error: SceneInputError) -> Self {
        Self::SceneInput(Box::new(error))
    }
}
