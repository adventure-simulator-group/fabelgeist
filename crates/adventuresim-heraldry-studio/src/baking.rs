//! Coalesced draft and refinement jobs. Only matching results can enter the scene.
#[cfg(target_family = "wasm")]
#[path = "baking/browser.rs"]
mod backend;
#[cfg(not(target_family = "wasm"))]
#[path = "baking/native.rs"]
mod backend;
use crate::app::Studio;
use adventuresim_heraldry::{
    bake::{Baked, Resolution, stamp},
    document::Document,
    export,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
const REFINE_DELAY: f64 = 0.45;
#[derive(Serialize, Deserialize)]
struct Job {
    document: Document,
    resolution: Resolution,
    export: bool,
}
pub(crate) struct Baker {
    backend: backend::Backend,
    busy: Option<Job>,
    failed: Option<String>,
}
impl Baker {
    pub fn new() -> Self {
        Self {
            backend: backend::Backend::new(),
            busy: None,
            failed: None,
        }
    }
}
pub(crate) fn execute(source: &str) -> Result<Vec<u8>, String> {
    let job: Job = serde_json::from_str(source).map_err(|e| e.to_string())?;
    let baked = Baked::generate(&job.document, job.resolution).map_err(|e| e.to_string())?;
    let bytes = baked.to_bytes();
    let mut out = (bytes.len() as u32).to_le_bytes().to_vec();
    out.extend(bytes);
    if job.export {
        out.extend(export::zip(
            &export::bundle(&job.document, &baked, job.resolution).map_err(|e| e.to_string())?,
        ));
    }
    Ok(out)
}
pub(crate) fn update(mut studio: ResMut<Studio>, mut baker: NonSendMut<Baker>, time: Res<Time>) {
    if let Some(result) = baker.backend.poll()
        && let Some(job) = baker.busy.take()
    {
        match result {
            Ok(bytes) => match receive(&bytes, &job, &mut studio) {
                Ok(()) => (),
                Err(error) => studio.status = error,
            },
            Err(error) => {
                baker.failed = Some(stamp(&job.document, job.resolution));
                studio.status = format!("Bake failed: {error}");
            }
        }
    }
    if let Some(job) = &baker.busy
        && stamp(&job.document, job.resolution) != stamp(&studio.document, job.resolution)
        && baker.backend.cancel()
    {
        baker.busy = None;
    }
    if baker.busy.is_some() {
        return;
    }
    let target = if studio.export_requested {
        studio.resolution
    } else if studio.ready() {
        return;
    } else if studio
        .current
        .as_ref()
        .is_some_and(|b| b.matches(&studio.document, Resolution::Draft))
    {
        if time.elapsed_secs_f64() - studio.last_edit < REFINE_DELAY {
            return;
        }
        studio.resolution
    } else {
        Resolution::Draft
    };
    if baker.failed.as_ref() == Some(&stamp(&studio.document, target)) {
        return;
    }
    if let Err(error) = studio.document.validate() {
        studio.status = error.to_string();
        return;
    }
    let job = Job {
        document: studio.document.clone(),
        resolution: target,
        export: studio.export_requested,
    };
    studio.export_requested = false;
    studio.status = format!(
        "{} {} px…",
        if job.export { "Exporting" } else { "Baking" },
        target.pixels()
    );
    baker.backend.start(serde_json::to_string(&job).unwrap());
    baker.busy = Some(job);
}
fn receive(bytes: &[u8], job: &Job, studio: &mut Studio) -> Result<(), String> {
    if bytes.len() < 4 {
        return Err("Incomplete worker response".into());
    }
    let length = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
    let baked = Baked::from_bytes(bytes.get(4..4 + length).ok_or("Incomplete worker maps")?)
        .map_err(|e| e.to_string())?;
    if !baked.matches(&studio.document, job.resolution) {
        return Ok(());
    }
    studio.status = format!(
        "{} × {} · {}",
        baked.size,
        baked.size,
        if job.resolution == studio.resolution {
            "Ready"
        } else {
            "Draft; refining"
        }
    );
    studio.current = Some(std::sync::Arc::new(baked));
    studio.current_document = Some(job.document.clone());
    studio.scene_dirty = true;
    if job.export {
        if job.document != studio.document {
            studio.status = "Document changed during export; export again.".into();
            return Ok(());
        }
        crate::exchange::save_bytes("heraldry.zip", &bytes[4 + length..], "application/zip")?;
        studio.status = "Exported heraldry.zip".into();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_worker_results_never_replace_current_artwork() {
        let job = Job {
            document: Document::default(),
            resolution: Resolution::Draft,
            export: false,
        };
        let bytes = execute(&serde_json::to_string(&job).unwrap()).unwrap();
        let mut studio = Studio::new(job.document.clone());
        studio.document.surface.seed.0 += 1;
        receive(&bytes, &job, &mut studio).unwrap();
        assert!(studio.current.is_none());
        studio.document = job.document.clone();
        studio.document.view.light.0 += 20.0;
        receive(&bytes, &job, &mut studio).unwrap();
        assert!(
            studio
                .current
                .as_ref()
                .unwrap()
                .matches(&studio.document, Resolution::Draft)
        );
        assert!(receive(&bytes[..70], &job, &mut studio).is_err());
    }
}
