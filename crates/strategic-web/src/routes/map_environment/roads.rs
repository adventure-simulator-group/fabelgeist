//! Lazy immutable source admission; no road I/O on strategic startup.
use adventuresim_terrain::{
    TerrainPack,
    road_pack::{ROAD_MANIFEST_NAME, ROAD_PACK_NAME, RoadPack, RoadPackError},
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub(crate) type Result<T> = std::result::Result<T, RoadSourceError>;

pub(crate) struct RegionalRoadSource {
    directory: PathBuf,
    resident: Mutex<Option<Arc<RoadPack>>>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RoadSourceError {
    #[error(transparent)]
    Package(#[from] RoadPackError),
    #[error("regional road source ownership is unavailable")]
    Ownership,
}

impl RegionalRoadSource {
    pub(crate) fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            resident: Mutex::new(None),
        }
    }

    /// Called only inside bounded blocking capture. Failed admission leaves the
    /// slot empty so an explicit retry can admit repaired development artifacts.
    pub(super) fn load(&self, terrain: &TerrainPack) -> Result<Arc<RoadPack>> {
        let mut resident = self
            .resident
            .lock()
            .map_err(|_| RoadSourceError::Ownership)?;
        if let Some(roads) = &*resident {
            if roads.source().as_str() != terrain.digest() {
                return Err(RoadPackError::SourceMismatch.into());
            }
            return Ok(roads.clone());
        }
        let roads = Arc::new(RoadPack::load(
            &self.directory.join(ROAD_MANIFEST_NAME),
            &self.directory.join(ROAD_PACK_NAME),
            terrain,
        )?);
        *resident = Some(roads.clone());
        Ok(roads)
    }
}
