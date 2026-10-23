//! Export the native tailoring source independently of garment assembly.
use adventuresim_character_creator::{
    armor_recipes::ParametricDesign, cloth_panel::PanelPlan, device_body::DeviceBody,
};
use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

#[path = "textile_review/body.rs"]
mod review;

#[derive(Parser)]
struct Args {
    body: PathBuf,
    outfit: PathBuf,
    output: PathBuf,
}
#[derive(Deserialize)]
struct Article {
    design: ParametricDesign,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let bytes = std::fs::read(&args.body)?;
    let body: review::Body = serde_json::from_slice(&bytes)?;
    let articles: Vec<Article> = serde_json::from_slice(&std::fs::read(args.outfit)?)?;
    let article = articles.first().context("empty review outfit")?;
    let (design, sleeves) = match &article.design {
        ParametricDesign::Gown(gown) => (&gown.bodice, gown.sleeves.as_ref()),
        ParametricDesign::Bodice(bodice) => (bodice, None),
        _ => anyhow::bail!("panel review requires a gown or bodice first"),
    };
    let device = DeviceBody::default();
    let realization = body.realization(&device);
    let fit = body.fit_body();
    let wearer = realization.wearer(&fit);
    let head = fabelgeist_armor::PartFrame {
        origin: [0.; 3],
        axes: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        half_extents: [1.; 3],
    };
    let (_, surface) = PanelPlan::new(design, &wearer, &head, sleeves)?;
    let source = surface.wearer(&wearer);
    let indices = surface.faces.iter().flatten().copied().collect::<Vec<_>>();
    let count = surface.positions.len();
    let mesh = serde_json::json!({"positions":surface.positions,"normals":surface.normals,"indices":indices,"texcoords":vec![[0.,0.];count],"joint_indices":source.joint_indices.native_words(),"joint_weights":source.joint_weights,"morphs":[],"textile_maps":[],"components":[{"role":"bodice_fabric","vertices":{"start":0,"end":count},"indices":{"start":0,"end":indices.len()},"material":{"base_color":[0.8,0.8,0.8,1.],"metallic":0.,"roughness":0.8},"textile":null}]});
    std::fs::create_dir_all(&args.output)?;
    std::fs::write(args.output.join("body.json"), bytes)?;
    std::fs::write(
        args.output.join("article-0.json"),
        serde_json::to_vec(&mesh)?,
    )?;
    Ok(())
}
