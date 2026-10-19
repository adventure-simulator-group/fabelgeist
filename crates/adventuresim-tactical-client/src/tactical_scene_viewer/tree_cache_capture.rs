//! Process-isolated source/cache content plates under production presentation.

use anyhow::{Context, Result, ensure};
use bevy::prelude::*;
use fabelgeist_botany::identity::PlantId;
use serde::Serialize;
use std::{fs, path::PathBuf, sync::Arc};

use super::{
    SceneSetupData, view_selection,
    view_specs::{CapturePose, CaptureViewSpec},
};
use crate::presentation::{PreparedTreeCache, TreeCacheRepresentation, TreeSourceGeometry};

mod admission;
mod background;
mod cameras;
mod evidence;
pub(super) mod inventory;
mod material_draws;
mod runtime;

pub(super) use evidence::Evidence;
pub(super) use runtime::{install, ready};

pub(super) const PROFILE: &str = "tree-cache-review";

#[derive(Debug, clap::Args)]
#[group(multiple = true, requires_all=["directory", "source", "role", "representation", "scene_input"], conflicts_with_all=["fixture", "profile", "prepare_tree_bank", "canopy_bps", "absolute_minute", "leaf_benchmark_frames", "tree_lighting_benchmark_frames", "scene_performance_benchmark_frames", "terrain_wireframe", "triangle_census", "city_cameras", "tree_review_azimuth_degrees"])]
pub(crate) struct Options {
    /// Directory containing the checked complete variant.treecache.
    #[arg(long = "cache-review-dir")]
    directory: Option<PathBuf>,
    /// Actual production source PlantId bound by the cache.
    #[arg(long = "cache-review-source")]
    source: Option<u64>,
    /// Render generated production assets or their admitted cache per process.
    #[arg(long = "cache-review-role", value_enum)]
    role: Option<Role>,
    /// Fixed production representation, held for the entire process.
    #[arg(long = "cache-review-representation", value_enum)]
    representation: Option<TreeCacheRepresentation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Role {
    Reference,
    Candidate,
}

struct Request {
    directory: PathBuf,
    source: PlantId,
    role: Role,
    representation: TreeCacheRepresentation,
}

#[derive(Resource)]
pub(super) struct Review {
    role: Role,
    representation: TreeCacheRepresentation,
    source: PlantId,
    cache: Option<PreparedTreeCache>,
    cameras: cameras::Cameras,
    root: Vec3,
    metadata: serde_json::Value,
}

impl Options {
    pub(super) fn admit(self, setup: &SceneSetupData) -> Result<Review> {
        let request = Request {
            directory: self.directory.context("cache review directory required")?,
            source: PlantId::from_word(self.source.context("cache source identity required")?),
            role: self.role.context("cache review role required")?,
            representation: self
                .representation
                .context("cache representation required")?,
        };
        admission::load(request, setup)
    }
}

impl Review {
    pub(super) fn views(&self, requested: &[String]) -> Result<Vec<CaptureViewSpec>, String> {
        view_selection::select(
            &self.cameras.views(self.representation),
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
