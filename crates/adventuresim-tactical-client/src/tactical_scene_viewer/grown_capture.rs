//! Three grounded instances of one source-owned grown asset set in the tactical host.

use std::{fs, path::PathBuf, sync::Arc};

use adventuresim_environment::{
    prepared::{
        GrownGeometry, GrownMeshBudget, GrownMeshBytes, GrownPartVertexLimit, GrownVertexEncoding,
        TreeBank, TreeBounds,
    },
    render::{Level, LevelSelection, grown::GrownAssets},
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
pub(super) use prepare::prepare;
pub(super) use runtime::ready;
use subject::Subject;

mod background;
mod cameras;
mod construction;
mod controls;
mod evidence;
pub(super) mod generated;
pub(super) mod inventory;
mod layout;
mod observation;
mod ordinary_batches;
mod prepare;
pub(super) mod runtime;
mod subject;

pub(super) const PROFILE: &str = "grown-review";
pub(super) const GENERATED_PROFILE: &str = "grown-generated-review";
const PART_VERTEX_LIMIT: u32 = 100_000;
const FINAL_MESH_BYTE_LIMIT: u64 = 64 * 1024 * 1024;

#[derive(Debug, clap::Args)]
#[group(multiple=true, requires_all=["mode","library","source","level","control","scene_input"], conflicts_with_all=["fixture","profile","prepare_tree_bank","canopy_bps","absolute_minute","leaf_benchmark_frames","tree_lighting_benchmark_frames","scene_performance_benchmark_frames","terrain_wireframe","triangle_census","city_cameras","tree_review_azimuth_degrees","cache_action"])]
pub(crate) struct GrownCaptureOptions {
    /// Private specimens or the actual generated tactical roots.
    #[arg(long = "grown-review-mode", value_enum)]
    mode: Option<CaptureMode>,
    /// Second actual source, required only by the generated-scene trial.
    #[arg(long = "grown-review-second-source")]
    second_source: Option<u64>,
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
    level: Option<LevelSelection>,
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
    pub review: Option<PreparedReview>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Role {
    Reference,
    Candidate,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub(super) enum CaptureMode {
    Specimens,
    Generated,
}

pub(super) enum PreparedReview {
    Specimens(Review),
    Generated(generated::Review),
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
    pub(super) fn profile(&self) -> Result<&'static str> {
        match self.mode.context("grown capture mode required")? {
            CaptureMode::Specimens => Ok(PROFILE),
            CaptureMode::Generated => Ok(GENERATED_PROFILE),
        }
    }
}

pub(super) fn install(app: &mut App, review: PreparedReview) {
    match review {
        PreparedReview::Specimens(review) => runtime::install(app, review),
        PreparedReview::Generated(review) => generated::install(app, review),
    }
}

// Closed CLI token boundary; the renderer owns the shared Level enum.
fn parse_level(value: &str) -> Result<LevelSelection, String> {
    match value {
        "full" => Ok(LevelSelection::Fixed(Level::Full)),
        "middle" => Ok(LevelSelection::Fixed(Level::Middle)),
        "coarse" => Ok(LevelSelection::Fixed(Level::Coarse)),
        "adaptive" => Ok(LevelSelection::Adaptive),
        _ => Err("expected adaptive, full, middle or coarse".to_owned()),
    }
}
