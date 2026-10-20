//! Three grounded instances of one source-owned grown asset set in the tactical host.

use std::{fs, path::PathBuf, sync::Arc};

use adventuresim_environment::{
    prepared::{
        GrownGeometry, GrownMeshBudget, GrownMeshBytes, GrownPartVertexLimit, GrownVertexEncoding,
        TreeBank, TreeBounds,
    },
    render::{
        Level,
        grown::{GrownAssets, GrownMaterialPlugin},
    },
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
mod construction;
mod controls;
mod evidence;
pub(super) mod inventory;
mod layout;
mod observation;
mod ordinary_batches;
mod runtime;

pub(super) const PROFILE: &str = "grown-review";
const PART_VERTEX_LIMIT: u32 = 100_000;
const FINAL_MESH_BYTE_LIMIT: u64 = 64 * 1024 * 1024;

#[derive(Debug, clap::Args)]
#[group(multiple=true, requires_all=["library","source","role","level","control","scene_input"], conflicts_with_all=["fixture","profile","prepare_tree_bank","canopy_bps","absolute_minute","leaf_benchmark_frames","tree_lighting_benchmark_frames","scene_performance_benchmark_frames","terrain_wireframe","triangle_census","city_cameras","tree_review_azimuth_degrees","cache_action"])]
pub(crate) struct GrownCaptureOptions {
    /// Exact admitted grown library from the independent producer.
    #[arg(long = "grown-review-library")]
    library: Option<PathBuf>,
    /// Actual botanical PlantId, independent of bank ordinal.
    #[arg(long = "grown-review-source")]
    source: Option<u64>,
    /// Live address expansion or native real-attribute evaluation.
    #[arg(long = "grown-review-role", value_enum)]
    role: Option<Role>,
    /// Explicit geometry detail, shared by native expansion and source materials.
    #[arg(long = "grown-review-level", value_parser = parse_level)]
    level: Option<Level>,
    /// Actual batching, or an explicit expected-rejection control.
    #[arg(long = "grown-review-batch-control", value_enum)]
    control: Option<controls::BatchControl>,
}

#[derive(Resource)]
pub(super) struct Review {
    role: Role,
    level: Level,
    control: controls::BatchControl,
    source: PlantId,
    geometry: Option<GrownGeometry>,
    requested: Vec<String>,
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

impl GrownCaptureOptions {
    fn admit(self, setup: &SceneSetupData) -> Result<Review> {
        let role = self.role.context("grown role required")?;
        let level = self.level.context("grown level required")?;
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
        let constructed = role.construct(
            &bank,
            source,
            level,
            GrownMeshBudget {
                part_vertices: GrownPartVertexLimit::from_vertices(PART_VERTEX_LIMIT)?,
                total_mesh_bytes: GrownMeshBytes::from_bytes(FINAL_MESH_BYTE_LIMIT),
            },
        )?;
        let geometry = constructed.geometry;
        let metadata = serde_json::json!({
            "role":role,"level":geometry.level(),"source_id":source.word(),"variant":geometry.variant().index(),"bank_id":bank.id(),
            "library_bytes":bytes.len(),"species":species.name,"foliage":format!("{:?}",species.foliage),
            "bark_rgb":species.bark,"leaf_rgb":species.leaf,"bounds":bounds,
            "source_branches":source_branches,"source_leaves":source_leaves,
            "detail":library.detail,"mesh_payload_bytes":geometry.mesh_bytes().bytes(),
            "part_count":geometry.parts().len(),"part_vertex_limit":PART_VERTEX_LIMIT,"final_mesh_byte_limit":FINAL_MESH_BYTE_LIMIT,
            "vertex_encoding":format!("{:?}",geometry.encoding()),"authoring_adapter":constructed.authoring_adapter,
            "input_digest":setup.input.digest()?,"scene_digest":setup.generated.digest,"environment":setup.environment,
            "root_placements":placements,"suppressed_generated_obstacles":setup.generated.obstacles.len(),
            "time_phase":super::CAPTURE_CLOCK_PHASE_SECONDS,
            "draw_policy":"ordinary_gpu_membership","batch_control":control,"shadow_cutout":"analytic source alpha < 0.02 discarded; binary depth/shadow, fractional forward coverage",
            "source_terrain":"saved flat source growth; destination placement by grounded root translations only",
            "scope":"three grounded placements of one grown source asset set; actual main-pass instance membership diagnostic, no complete cache, species calibration, adaptive LOD, motion, memory, performance or arrival acceptance"
        });
        Ok(Review {
            role,
            level,
            control,
            source,
            geometry: Some(geometry),
            requested: Vec::new(),
            placements,
            metadata,
        })
    }
}

pub(super) fn prepare(
    mut setup: SceneSetupData,
    options: Option<GrownCaptureOptions>,
    requested: &[String],
) -> Result<PreparedCapture> {
    if options.is_some() {
        view_selection::validate(&cameras::Cameras::slugs(), requested)
            .map_err(anyhow::Error::msg)?;
    }
    let mut review = options.map(|options| options.admit(&setup)).transpose()?;
    if let Some(review) = &mut review {
        review.requested = requested.to_vec();
        setup.views.clear();
        setup.requested_views.clear();
    }
    Ok(PreparedCapture { setup, review })
}

// Closed CLI token boundary; the renderer owns the shared Level enum.
fn parse_level(value: &str) -> Result<Level, String> {
    match value {
        "full" => Ok(Level::Full),
        "middle" => Ok(Level::Middle),
        "coarse" => Ok(Level::Coarse),
        _ => Err("expected full, middle or coarse".to_owned()),
    }
}
