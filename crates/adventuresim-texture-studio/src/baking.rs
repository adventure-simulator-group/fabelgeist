//! Coalesced draft/refine jobs. Rendering never waits for texture generation.
use crate::{app::Studio, document::Document};
use adventuresim_procedural_textures::{BakeResolution, BakedRecipe};
use bevy::prelude::*;

#[cfg(target_family = "wasm")]
#[path = "baking/browser.rs"]
mod backend;
#[cfg(not(target_family = "wasm"))]
#[path = "baking/native.rs"]
mod backend;

pub(crate) struct Baker {
    backend: backend::Backend,
    busy: Option<(u64, BakeResolution)>,
}
impl Baker {
    pub fn new() -> Self {
        Self {
            backend: backend::Backend::new(),
            busy: None,
        }
    }
    fn start(&mut self, document: &Document, revision: u64, resolution: BakeResolution) {
        let mut document = document.clone();
        document.texture.resolution = resolution;
        self.backend.start(document);
        self.busy = Some((revision, resolution));
    }
}

pub(crate) fn update(mut studio: ResMut<Studio>, mut baker: NonSendMut<Baker>, time: Res<Time>) {
    if let Some(result) = baker.backend.poll()
        && let Some((revision, quality)) = baker.busy.take()
    {
        match result {
            Ok(bake) if revision == studio.revision => {
                let size = bake.maps[0].size;
                studio.current = Some(bake);
                studio.applied = Some((revision, quality));
                studio.scene_dirty = true;
                studio.status = format!(
                    "{size} × {size} · {}",
                    if quality == studio.document.texture.resolution {
                        "Final quality"
                    } else {
                        "Draft · refining after edits"
                    }
                );
            }
            Err(error) => {
                // A trapped WASM instance is discarded before another edit or retry.
                baker.backend.cancel();
                if revision == studio.revision {
                    studio.status = format!("Bake failed: {error}");
                    studio.failed_revision = Some(revision);
                }
            }
            _ => {}
        }
    }
    if let Some((revision, quality)) = baker.busy
        && quality != BakeResolution::Draft
        && revision != studio.revision
        && baker.backend.cancel()
    {
        baker.busy = None;
    }
    if baker.busy.is_some() || studio.failed_revision == Some(studio.revision) {
        return;
    }
    let age = time.elapsed_secs_f64() - studio.last_edit;
    let target = studio.document.texture.resolution;
    let quality = match studio.applied {
        Some((revision, quality)) if revision == studio.revision && quality == target => return,
        Some((revision, _)) if revision == studio.revision && age > 0.45 => target,
        Some((revision, _)) if revision == studio.revision => return,
        _ => BakeResolution::Draft,
    };
    if let Err(error) = studio.document.texture.validate() {
        studio.status = error.to_string();
        return;
    }
    baker.start(&studio.document, studio.revision, quality);
    studio.status = if matches!(quality, BakeResolution::Full) {
        "Refining full mip chain…"
    } else {
        "Baking draft…"
    }
    .into();
}
