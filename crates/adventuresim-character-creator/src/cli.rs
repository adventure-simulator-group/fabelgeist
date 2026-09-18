use adventuresim_character_creator::lod::{MAX_CHARACTER_LOD, MIN_CHARACTER_LOD};
use bevy::prelude::Resource;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Resource, Clone)]
#[command(about = "Fabelgeist's MHR character design studio")]
pub(super) struct Args {
    #[arg(
        long,
        env = "MHR_ASSETS",
        default_value = "target/mhr-assets/v1.0.1/assets"
    )]
    pub(super) assets: PathBuf,
    #[arg(long, default_value_t = MIN_CHARACTER_LOD,
        value_parser = clap::value_parser!(u8).range(i64::from(MIN_CHARACTER_LOD)..=i64::from(MAX_CHARACTER_LOD)))]
    pub(super) lod: u8,
    #[arg(long, default_value = "assets_src/characters/mhr_base.json")]
    pub(super) recipe: PathBuf,
    #[arg(long, default_value = "assets_src/biped/unarmed/base.glb")]
    pub(super) glb: PathBuf,
    #[arg(long, default_value = "content/items")]
    pub(super) catalog: PathBuf,
    #[arg(long, default_value = "assets/equipment/procedural")]
    pub(super) equipment_output: PathBuf,
    /// Vambrace design for studio, review, character and equipment exports.
    #[arg(long)]
    pub(super) bracer_design: Option<PathBuf>,
    /// BreastplateDesign JSON for studio, character, and equipment exports.
    #[arg(long)]
    pub(super) breastplate_design: Option<PathBuf>,
    /// Generate dense armor only as input to the offline texture baker.
    #[arg(long, requires = "armor_review_dir")]
    pub(super) armor_bake_source: bool,
    /// Export the selected recipe without opening the studio window.
    #[arg(long)]
    pub(super) export_only: bool,
    /// Generate one procedural MHR asset for every armor/clothing placement.
    #[arg(long)]
    pub(super) generate_equipment: bool,
    /// Restrict generation to these comma-separated catalog items.
    #[arg(long, requires = "generate_equipment", value_delimiter = ',')]
    pub(super) equipment_item: Vec<String>,
    /// Export actual body and recipe triangles for reproducible artistic review.
    #[arg(long)]
    pub(super) armor_review_dir: Option<PathBuf>,
    /// Review the complete catalog, the recipe's outfit, or only its body.
    #[arg(
        long,
        value_enum,
        default_value = "catalog",
        requires = "armor_review_dir"
    )]
    pub(super) armor_review_selection: super::review_export::ReviewSelection,
    /// Saved item defaults and placement-specific armor recipes for preview and exports.
    #[arg(long)]
    pub(super) armor_designs: Option<PathBuf>,
    /// Leather closure dimensions and fastening layouts for preview and exports.
    #[arg(long)]
    pub(super) fastener_designs: Option<PathBuf>,
    /// Write editable item defaults and any loaded placement-specific recipes.
    #[arg(long)]
    pub(super) write_armor_designs: Option<PathBuf>,
    /// Print JSON wall-clock timings for generation and export stages.
    #[arg(long)]
    pub(super) profile: bool,
}
