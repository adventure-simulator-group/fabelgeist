//! Diagnostic source sampling; vista observations never become tactical support.
use super::{Inspection, TerrainStages};
use adventuresim_tactical_core::{
    city_layout::grounding::SupportElevation,
    scene::SceneTerrain,
    scene_coordinates::ScenePlanPoint,
    scene_input::{TacticalSceneInput, VistaSample},
    vista_surface::vista_triangle_height,
};

pub(super) struct InspectionSource<'a> {
    terrain: &'a SceneTerrain,
    vista: &'a VistaSample,
}

impl<'a> InspectionSource<'a> {
    pub(super) fn new(terrain: &'a SceneTerrain, vista: &'a VistaSample) -> Self {
        Self { terrain, vista }
    }

    pub(super) fn height_at(&self, point: ScenePlanPoint) -> Option<SupportElevation> {
        self.terrain.height_at(point).or_else(|| {
            self.vista.lods.iter().enumerate().find_map(|(index, lod)| {
                // The complete triangle/LOD stitch kernel consumes scene metres.
                SupportElevation::from_metres(vista_triangle_height(
                    lod,
                    self.vista.lods.get(index + 1),
                    self.terrain,
                    point.metres(),
                )?)
            })
        })
    }
}

impl TerrainStages {
    pub(super) fn load_verified(
        request: &Inspection,
        input: &TacticalSceneInput,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let stages: TerrainStages = serde_json::from_slice(&std::fs::read(
            request.terrain_stages
                .as_ref()
                .ok_or("missing terrain stages")?,
        )?)?;
        if Some(stages.source_digest.as_str())
            != match &input.source {
                adventuresim_tactical_core::scene_input::SceneSource::ImportedPackage(digest) => {
                    Some(digest.as_str())
                }
                _ => None,
            }
        {
            return Err("terrain stages do not identify the scene source".into());
        }
        if stages.input_digest != input.digest()? {
            return Err("terrain stages do not identify this exact scene input".into());
        }
        Ok(stages)
    }
}
