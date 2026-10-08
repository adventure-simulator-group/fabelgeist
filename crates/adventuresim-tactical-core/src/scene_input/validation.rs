//! Validate geographic descriptors independently of producer support selection.
use super::*;

impl TacticalSceneInput {
    pub(super) fn validate_source(&self) -> Result<(), SceneInputError> {
        if self.schema_version != TACTICAL_SCENE_SCHEMA_VERSION {
            return invalid(SceneValidationError::SchemaVersion);
        }
        if self.generation_version != TACTICAL_SCENE_GENERATION_VERSION {
            return invalid(SceneValidationError::GenerationVersion);
        }
        if self.scene_key.is_empty() || self.scene_key.len() > MAX_TEMPLATE_BYTES {
            return invalid(SceneValidationError::SceneKey);
        }
        let source_id = match &self.source {
            SceneSource::ImportedPackage(value) => value.as_str(),
            SceneSource::SyntheticFixture(value) => value.as_str(),
        };
        if source_id.is_empty() || source_id.len() > MAX_SOURCE_ID_BYTES {
            return invalid(SceneValidationError::SourceIdentity);
        }

        validate_grid(&self.playable, MAX_PLAYABLE_SIDE, SampleGridKind::Playable)?;
        crate::scene_fault::validate(self.landform, &self.playable)?;
        urban::validate(self)?;
        establishments::validate(self)?;
        properties::validate(self)?;
        if self.vista.lods.len() > MAX_VISTA_LEVELS {
            return invalid(SceneValidationError::VistaLevelCount);
        }
        let mut previous_level = None;
        let mut previous_spacing = self.playable.spacing_metres;
        let mut vista_samples = 0usize;
        for lod in &self.vista.lods {
            if previous_level.is_some_and(|level| lod.level <= level) {
                return invalid(SceneValidationError::VistaLevelOrder);
            }
            if !lod.origin_east_metres.is_finite() || !lod.origin_north_metres.is_finite() {
                return invalid(SceneValidationError::VistaOrigin);
            }
            let grid = TerrainSampleGrid {
                width: lod.width,
                depth: lod.depth,
                spacing_metres: lod.spacing_metres,
                heights_metres: lod.heights_metres.clone(),
                environment: lod.environment.clone(),
            };
            validate_grid(&grid, u16::MAX as usize, SampleGridKind::Vista)?;
            if lod.spacing_metres <= previous_spacing {
                return invalid(SceneValidationError::VistaSpacingOrder);
            }
            vista_samples = vista_samples.checked_add(lod.heights_metres.len()).ok_or({
                SceneInputError::Validation(SceneValidationError::VistaSampleOverflow)
            })?;
            if vista_samples > MAX_VISTA_SAMPLES {
                return invalid(SceneValidationError::VistaSampleCount);
            }
            previous_level = Some(lod.level);
            previous_spacing = lod.spacing_metres;
        }
        validate_weather(self.weather)?;
        Ok(())
    }
}
