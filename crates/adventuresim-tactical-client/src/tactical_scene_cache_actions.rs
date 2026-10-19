//! Exclusive native CPU cache authoring, source inspection and pinned admission.
use crate::presentation::prepared_cache::{
    TreeCacheContentId, admit_tree_cache, inspect_tree_cache, write_tree_cache,
};
use anyhow::{Context, Result, bail};
use fabelgeist_botany::identity::PlantId;
use std::path::{Path, PathBuf};

#[derive(Debug, clap::Args)]
#[group(multiple = true, conflicts_with_all = ["profile", "canopy_bps", "absolute_minute", "output", "views", "settle_frames", "leaf_benchmark_frames", "tree_lighting_benchmark_frames", "scene_performance_benchmark_frames", "scene_performance_render_diagnostics", "terrain_wireframe", "triangle_census", "tree_review_azimuth_degrees", "city_cameras", "prepare_tree_bank", "fixture"])]
#[command(group(clap::ArgGroup::new("cache_action").args(["prepare_tree_cache", "check_tree_cache", "admit_tree_cache"]).multiple(false)))]
pub(crate) struct TreeCacheActions {
    /// Write one complete lossless production cache without a renderer.
    #[arg(long, requires_all = ["scene_input", "cache_source"])]
    prepare_tree_cache: Option<PathBuf>,
    /// Inspect structure and regenerate the exact source as an offline control.
    #[arg(long, requires_all = ["scene_input", "cache_source"])]
    check_tree_cache: Option<PathBuf>,
    /// Admit independently pinned offline-verified content without a renderer.
    #[arg(long, requires_all = ["scene_input", "cache_source", "cache_content"])]
    admit_tree_cache: Option<PathBuf>,
    /// Production source PlantId selected for this CPU action.
    #[arg(long, requires = "cache_action")]
    cache_source: Option<u64>,
    /// Previously verified producer identity, selected independently of its file.
    #[arg(long, requires = "admit_tree_cache", conflicts_with_all = ["prepare_tree_cache", "check_tree_cache"])]
    cache_content: Option<TreeCacheContentId>,
}

impl TreeCacheActions {
    pub(crate) fn run(self, input_path: &Path) -> Result<()> {
        let source = PlantId::from_word(self.cache_source.context("cache source required")?);
        match (
            self.prepare_tree_cache,
            self.check_tree_cache,
            self.admit_tree_cache,
        ) {
            (Some(output), None, None) => write_tree_cache(input_path, &output, source),
            (None, Some(cache), None) => inspect_tree_cache(input_path, &cache, source),
            (None, None, Some(cache)) => admit_tree_cache(
                input_path,
                &cache,
                source,
                self.cache_content
                    .context("pinned producer content required")?,
            ),
            _ => bail!("one exclusive CPU cache action required"),
        }
    }
}
