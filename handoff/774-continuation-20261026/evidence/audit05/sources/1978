//! Ignored diagnostic capture; not a production scene or grounding format.
use adventuresim_tactical_core::{city_layout::grounding::GeographicSurface, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum TerrainPreparationPurpose {
    CompleteSceneContext,
    PropertyAssetIsolation,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PreparedTerrainCapture {
    pub purpose: TerrainPreparationPurpose,
    pub input_digest: String,
    pub source_digest: adventuresim_world_schema::source_package::SourcePackageDigest,
    pub terrain: SceneTerrain,
}

impl PreparedTerrainCapture {
    pub fn geographic_surface(
        &self,
        input: &TacticalSceneInput,
        ungraded_vista: &VistaSample,
    ) -> Result<GeographicSurface, Box<dyn std::error::Error>> {
        if self.input_digest != input.digest()?
            || input.source != SceneSource::ImportedPackage(self.source_digest.clone())
        {
            return Err("prepared terrain capture must identify the exact original input and geographic source".into());
        }
        GeographicSurface::from_presented_scene(&self.terrain, ungraded_vista).ok_or_else(|| {
            "prepared fine terrain and vista rings cannot form a valid geographic source".into()
        })
    }
}
