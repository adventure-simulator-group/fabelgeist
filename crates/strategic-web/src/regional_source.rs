//! Startup admission of the terrain shared by routing and map presentation.
use crate::routes::travel::TerrainPlanner;
use adventuresim_terrain::{TerrainPack, TerrainPurpose};
use std::{path::Path, sync::Arc};

pub(crate) fn load_terrain(bundle: &Path) -> Option<Arc<TerrainPlanner>> {
    match admit_terrain(bundle) {
        Ok(planner) => {
            tracing::info!(bundle=%bundle.display(),digest=%planner.digest(),"loaded final terrain for strategic routing and map presentation");
            Some(Arc::new(planner))
        }
        Err(error) => {
            tracing::warn!(bundle=%bundle.display(),%error,"strategic terrain unavailable; disabling terrain routing and map presentation");
            None
        }
    }
}

fn admit_terrain(bundle: &Path) -> anyhow::Result<TerrainPlanner> {
    let pack = TerrainPack::load(
        &bundle.join("terrain-routing-v3.json"),
        &bundle.join("terrain-routing-v3.pack"),
    )?;
    anyhow::ensure!(
        pack.purpose() == TerrainPurpose::Final,
        "strategic routing and map presentation require the final terrain source"
    );
    Ok(TerrainPlanner::new(Arc::new(pack)))
}
