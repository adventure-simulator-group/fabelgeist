//! Three grounded instances of one source-owned Full asset set in the tactical host.

use std::{fs, path::PathBuf, sync::Arc};

use adventuresim_environment::{
    prepared::{
        FullAuthoringContext, FullExpansionBudget, FullPartVertexLimit, GrownFullGeometry,
        TreeBank, TreeBounds,
    },
    render::full::{GrownFullAssets, GrownFullMaterialPlugin, GrownVertexEncoding},
};
use anyhow::{Context, Result, ensure};
use bevy::prelude::*;
use fabelgeist_botany::identity::PlantId;
use serde::Serialize;

use super::{
    SceneSetupData, view_selection,
    view_specs::{CapturePose, CaptureViewSpec},
};

pub(super) use cameras::PlateCamera;
pub(super) use evidence::Evidence;
pub(super) use runtime::{install, ready};

mod background;
mod cameras;
mod controls;
mod evidence;
pub(super) mod inventory;
mod layout;
mod observation;
mod ordinary_batches;
mod runtime;

pub(super) const PROFILE: &str = "grown-full-review";
const PART_VERTEX_LIMIT: u32 = 100_000;
const FINAL_MESH_BYTE_LIMIT: u64 = 64 * 1024 * 1024;

#[derive(Debug, clap::Args)]
#[group(multiple=true, requires_all=["library","source","role","control","scene_input"], conflicts_with_all=["fixture","profile","prepare_tree_bank","canopy_bps","absolute_minute","leaf_benchmark_frames","tree_lighting_benchmark_frames","scene_performance_benchmark_frames","terrain_wireframe","triangle_census","city_cameras","tree_review_azimuth_degrees","cache_action"])]
pub(crate) struct GrownFullCaptureOptions {
    /// Exact admitted grown library from the independent producer.
    #[arg(long = "grown-review-library")]
    library: Option<PathBuf>,
    /// Actual botanical PlantId, independent of bank ordinal.
    #[arg(long = "grown-review-source")]
    source: Option<u64>,
    /// Live address expansion or native real-attribute evaluation.
    #[arg(long = "grown-review-role", value_enum)]
    role: Option<Role>,
    /// Actual batching, or an explicit expected-rejection control.
    #[arg(long = "grown-review-batch-control", value_enum)]
    control: Option<controls::BatchControl>,
}

#[derive(Resource)]
pub(super) struct Review {
    role: Role,
    control: controls::BatchControl,
    source: PlantId,
    geometry: Option<GrownFullGeometry>,
    cameras: cameras::Cameras,
    placements: layout::RootPlacements,
    metadata: serde_json::Value,
}

pub(super) struct PreparedCapture {
    pub setup: SceneSetupData,
    pub review: Option<Review>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Role {
    Reference,
    Candidate,
}

impl Role {
    fn encoding(self) -> GrownVertexEncoding {
        match self {
            Self::Reference => GrownVertexEncoding::Address,
            Self::Candidate => GrownVertexEncoding::Expanded,
        }
    }
}

impl GrownFullCaptureOptions {
    fn admit(self, setup: &SceneSetupData) -> Result<Review> {
        let role = self.role.context("grown role required")?;
        let control = self.control.context("grown batch control required")?;
        let source = PlantId::from_word(self.source.context("grown source required")?);
        let path = self.library.context("grown library required")?;
        let bytes = fs::read(&path)?;
        let bank = Arc::new(TreeBank::from_bytes(&bytes)?);
        let library = bank.library();
        let mut matches = library.models.iter().filter(|model| model.source == source);
        let model = matches.next().context("grown source missing")?;
        ensure!(matches.next().is_none(), "grown source ambiguous");
        ensure!(
            model.branches.count > 0 && model.leaves.count > 0,
            "leafy grown witness requires wood and foliage"
        );
        let bounds = model.bounds;
        let source_branches = model.branches;
        let source_leaves = model.leaves;
        let species = library
            .species
            .get(model.species.index())
            .context("grown species missing")?;
        let placements = layout::RootPlacements::on_terrain(&setup.generated.terrain)?;
        let cameras = cameras::Cameras::from_source(bounds, &placements, &setup.generated.terrain)?;
        let mut author = pollster::block_on(FullAuthoringContext::new())?;
        let adapter = author.adapter_info();
        let geometry = pollster::block_on(bank.expand_full(
            &mut author,
            source,
            FullExpansionBudget {
                part_vertices: FullPartVertexLimit::from_vertices(PART_VERTEX_LIMIT)?,
                total_mesh_bytes: FINAL_MESH_BYTE_LIMIT.into(),
            },
        ))?;
        // Authoring has finished; this device never borrows the active renderer.
        drop(author);
        let metadata = serde_json::json!({
            "role":role,"source_id":source.word(),"variant":geometry.variant().index(),"bank_id":bank.id(),
            "library_bytes":bytes.len(),"species":species.name,"foliage":format!("{:?}",species.foliage),
            "bark_rgb":species.bark,"leaf_rgb":species.leaf,"bounds":bounds,
            "source_branches":source_branches,"source_leaves":source_leaves,
            "detail":library.detail,"mesh_payload_bytes":u64::from(geometry.mesh_bytes()),
            "part_count":geometry.parts().len(),"part_vertex_limit":PART_VERTEX_LIMIT,"final_mesh_byte_limit":FINAL_MESH_BYTE_LIMIT,
            "authoring_adapter":{"name":adapter.name,"backend":format!("{:?}",adapter.backend),"device_type":format!("{:?}",adapter.device_type),"driver":adapter.driver,"driver_info":adapter.driver_info},
            "input_digest":setup.input.digest()?,"scene_digest":setup.generated.digest,"environment":setup.environment,
            "root_placements":placements,"cameras":cameras,"suppressed_generated_obstacles":setup.generated.obstacles.len(),
            "time_phase":super::CAPTURE_CLOCK_PHASE_SECONDS,
            "draw_policy":"ordinary_gpu_membership","batch_control":control,"shadow_cutout":"analytic source alpha < 0.02 discarded; binary depth/shadow, fractional forward coverage",
            "source_terrain":"saved flat source growth; destination placement by grounded root translations only",
            "scope":"three grounded placements of one Full source asset set; actual main-pass instance membership diagnostic, no complete cache, species calibration, adaptive LOD, motion, memory, performance or arrival acceptance"
        });
        Ok(Review {
            role,
            control,
            source,
            geometry: Some(geometry),
            cameras,
            placements,
            metadata,
        })
    }
}

impl Review {
    fn views(&self, requested: &[String]) -> Result<Vec<CaptureViewSpec>, String> {
        view_selection::select(
            &self.cameras.views(),
            requested,
            view_selection::ViewSequence::Independent,
        )
    }
}

pub(super) fn prepare(
    mut setup: SceneSetupData,
    options: Option<GrownFullCaptureOptions>,
    requested: &[String],
) -> Result<PreparedCapture> {
    let review = options.map(|options| options.admit(&setup)).transpose()?;
    if let Some(review) = &review {
        setup.views = review.views(requested).map_err(anyhow::Error::msg)?;
        setup.requested_views = setup
            .views
            .iter()
            .filter(|view| !view.warmup)
            .map(|view| view.slug.to_owned())
            .collect();
    }
    Ok(PreparedCapture { setup, review })
}
