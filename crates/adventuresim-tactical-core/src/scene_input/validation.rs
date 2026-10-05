//! Validate geographic descriptors independently of producer support selection.
use super::*;

impl TacticalSceneInput {
    pub(super) fn validate_source(&self) -> Result<(), SceneInputError> {
        if self.schema_version != TACTICAL_SCENE_SCHEMA_VERSION {
            return invalid("incompatible schema version");
        }
        if self.generation_version != TACTICAL_SCENE_GENERATION_VERSION {
            return invalid("incompatible generation version");
        }
        if self.scene_key.is_empty() || self.scene_key.len() > MAX_TEMPLATE_BYTES {
            return invalid("scene key is empty or oversized");
        }
        let source_id = match &self.source {
            SceneSource::ImportedPackage(value) | SceneSource::SyntheticFixture(value) => value,
        };
        if source_id.is_empty() || source_id.len() > MAX_SOURCE_ID_BYTES {
            return invalid("source identity is empty or oversized");
        }
        if !(-90_000_000..=90_000_000).contains(&self.latitude_microdegrees)
            || !(-180_000_000..=180_000_000).contains(&self.longitude_microdegrees)
        {
            return invalid("geographic origin is out of bounds");
        }
        validate_grid(&self.playable, MAX_PLAYABLE_SIDE, "playable")?;
        crate::scene_fault::validate(self.landform, &self.playable)?;
        urban::validate(self)?;
        establishments::validate(self)?;
        properties::validate(self)?;
        if self.vista.lods.len() > MAX_VISTA_LEVELS {
            return invalid("vista has too many LOD levels");
        }
        let mut previous_level = None;
        let mut previous_spacing = self.playable.spacing_metres;
        let mut vista_samples = 0usize;
        for lod in &self.vista.lods {
            if previous_level.is_some_and(|level| lod.level <= level) {
                return invalid("vista LOD levels are not strictly increasing");
            }
            if !lod.origin_east_metres.is_finite() || !lod.origin_north_metres.is_finite() {
                return invalid("vista LOD origin is not finite");
            }
            let grid = TerrainSampleGrid {
                width: lod.width,
                depth: lod.depth,
                spacing_metres: lod.spacing_metres,
                heights_metres: lod.heights_metres.clone(),
                environment: lod.environment.clone(),
            };
            validate_grid(&grid, u16::MAX as usize, "vista")?;
            if lod.spacing_metres <= previous_spacing {
                return invalid("vista LOD spacing must progressively increase");
            }
            vista_samples = vista_samples
                .checked_add(lod.heights_metres.len())
                .ok_or_else(|| SceneInputError::Validation("vista sample count overflow".into()))?;
            if vista_samples > MAX_VISTA_SAMPLES {
                return invalid("vista sample count exceeds its bound");
            }
            previous_level = Some(lod.level);
            previous_spacing = lod.spacing_metres;
        }
        validate_weather(self.weather)?;
        Ok(())
    }
}
