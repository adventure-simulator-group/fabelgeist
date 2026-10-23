//! Export independently inspectable garments through the shared native fitter.
//! Articles are authored in inner-to-outer order; no hidden layer is culled.
use adventuresim_character_creator::{
    armor_design_input::ArmorPlacement,
    armor_frames::{FittingPlacement, Side},
    armor_recipes::ParametricDesign,
    device_body::DeviceBody,
    device_fit::{self, Fitted},
};
use anyhow::Result;
use clap::Parser;
use fabelgeist_armor::ArmorGpu;
use serde::Deserialize;
use std::path::PathBuf;

use review::{Body, MorphBodies};

#[path = "textile_review/mod.rs"]
mod review;

#[derive(Parser)]
struct Args {
    /// Native body-review JSON in canonical unposed metres.
    body: PathBuf,
    /// Ordered array of articles, each with a design and typed placement.
    outfit: PathBuf,
    output: PathBuf,
    #[arg(long = "morph-body")]
    morph_bodies: Vec<PathBuf>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Article {
    design: ParametricDesign,
    placement: ArmorPlacement,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let body_bytes = std::fs::read(&args.body)?;
    let body: Body = serde_json::from_slice(&body_bytes)?;
    let outfit_bytes = std::fs::read(&args.outfit)?;
    let articles: Vec<Article> = serde_json::from_slice(&outfit_bytes)?;
    anyhow::ensure!(!articles.is_empty(), "review outfit has no articles");
    for article in &articles {
        article.design.validate()?;
    }
    std::fs::create_dir_all(&args.output)?;
    std::fs::write(args.output.join("body.json"), body_bytes)?;
    std::fs::write(args.output.join("outfit.json"), outfit_bytes)?;
    let gpu = ArmorGpu::open()?;
    let device = DeviceBody::default();
    let samples = MorphBodies::load(&args.morph_bodies)?;
    let names = samples.names();
    let morphs = samples.realizations();
    let fit = body.fit_body();
    let wearer = body.realization(&device);
    let mut garments = Vec::new();
    for (index, article) in articles.iter().enumerate() {
        let placement = article.placement.as_str();
        let fitting_placement = match article.placement {
            ArmorPlacement::Left => FittingPlacement::Lateral(Side::Left),
            ArmorPlacement::Right => FittingPlacement::Lateral(Side::Right),
            ArmorPlacement::Worn => FittingPlacement::Unpaired,
        };
        let layers = garments.iter().collect::<Vec<_>>();
        let piece = device_fit::fit_recipe(
            &gpu,
            &fit,
            &wearer,
            &morphs,
            &article.design,
            fitting_placement,
            &layers,
        )?;
        let generated = device_fit::assemble_recipe(
            &article.design,
            piece,
            &Fitted {
                placement: fitting_placement,
                morphs: &names,
                domain: "review_body",
                joint_names: &body.joint_names,
                joints: &body.joints,
            },
        )?;
        review::write_generated(
            &args.output,
            &format!("article-{index}"),
            placement,
            &article.design,
            &generated,
        )?;
        garments.push(generated);
    }
    Ok(())
}
