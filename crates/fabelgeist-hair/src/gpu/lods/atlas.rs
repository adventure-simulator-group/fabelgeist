//! Reuse an immutable atlas recipe without publishing failed level builds.

use crate::cards::{AtlasSettings, CardAtlas};
use crate::gpu::GpuGroomer;
use crate::gpu::error::Result;
use crate::lod::ATLAS_SEED;
use std::sync::Arc;

impl GpuGroomer {
    /// Return a candidate; the level builder publishes it after submission.
    pub(super) fn atlas_for(&self, settings: AtlasSettings) -> Result<Arc<CardAtlas>> {
        if let Some(atlas) = &self.atlas
            && atlas.settings() == settings
        {
            return Ok(atlas.clone());
        }
        Ok(Arc::new(CardAtlas::paint(settings, ATLAS_SEED)?))
    }
}
