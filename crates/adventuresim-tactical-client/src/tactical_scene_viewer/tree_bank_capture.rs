//! Process-isolated source/bank content plates under production presentation.

use adventuresim_environment::prepared::{SharedForest, TreeBank};
use anyhow::{Context, Result, ensure};
use bevy::prelude::*;
use fabelgeist_botany::identity::PlantId;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Arc};

use super::{
    SceneSetupData, view_selection,
    view_specs::{CapturePose, CaptureViewSpec},
};
use crate::presentation::{TREE_BANK_SOURCE_FORMAT, TreePresentationSpecies, TreeSourceGeometry};

mod admission;
mod cameras;
mod evidence;
mod runtime;
mod source_draws;

pub(super) use evidence::Evidence;
pub(super) use runtime::{install, ready};

pub(super) const PROFILE: &str = "tree-bank-review";

#[derive(Debug, clap::Args)]
#[group(multiple = true, requires_all=["directory", "source", "role", "scene_input"], conflicts_with_all=["fixture", "profile", "prepare_tree_bank", "canopy_bps", "absolute_minute", "leaf_benchmark_frames", "tree_lighting_benchmark_frames", "scene_performance_benchmark_frames", "terrain_wireframe", "triangle_census", "city_cameras"])]
pub(crate) struct Options {
    /// Freshly authored checked bank and source manifest directory.
    #[arg(long = "bank-review-dir")]
    directory: Option<PathBuf>,
    /// Actual source PlantId from this bank's source.json.
    #[arg(long = "bank-review-source")]
    source: Option<u64>,
    /// Render one whole source or its admitted converted model per process.
    #[arg(long = "bank-review-role", value_enum)]
    role: Option<Role>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Role {
    Source,
    Candidate,
}

struct Request {
    directory: PathBuf,
    source: PlantId,
    role: Role,
}

#[derive(Resource)]
pub(super) struct Review {
    role: Role,
    source: Option<TreeSourceGeometry>,
    forest: Arc<SharedForest>,
    cameras: cameras::Cameras,
    root: Vec3,
    metadata: serde_json::Value,
}

impl Options {
    pub(super) fn admit(self, setup: &SceneSetupData) -> Result<Review> {
        let request = Request {
            directory: self.directory.context("bank review directory required")?,
            source: PlantId::from_word(self.source.context("bank source identity required")?),
            role: self.role.context("bank review role required")?,
        };
        admission::load(request, setup)
    }
}

impl Review {
    pub(super) fn views(&self, requested: &[String]) -> Result<Vec<CaptureViewSpec>, String> {
        view_selection::select(
            &self.cameras.views(),
            requested,
            view_selection::ViewSequence::Independent,
        )
    }
}

pub(super) use cameras::PlateCamera;

pub(super) struct PreparedCapture {
    pub setup: SceneSetupData,
    pub review: Option<Review>,
}

pub(super) fn prepare(
    mut setup: SceneSetupData,
    options: Option<Options>,
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
