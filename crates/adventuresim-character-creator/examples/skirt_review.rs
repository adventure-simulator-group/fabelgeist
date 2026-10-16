//! Export fitted skirt variants against an existing body review JSON.
//! This exercises the runtime fitter without requiring the offline body model.

use adventuresim_character_creator::{
    armor_recipes::ParametricDesign,
    device_body::DeviceBody,
    device_fit::{self, Fitted},
};
use anyhow::Result;
use clap::Parser;
use fabelgeist_armor::{ArmorGpu, Permille, SkirtDesign, SkirtProfile};
use review::{Body, MorphBodies};
use std::path::PathBuf;

#[path = "textile_review/mod.rs"]
mod review;

#[derive(Parser)]
struct Args {
    /// Body review JSON containing native mesh and rig attributes.
    body: PathBuf,
    /// Output consumed by scripts/render_armor_review.py.
    output: PathBuf,
    /// Additional direct-fit body realizations sharing the reference topology.
    #[arg(long = "morph-body")]
    morph_bodies: Vec<PathBuf>,
    /// Join a minimal body-conforming upper section.
    #[arg(long)]
    dress: bool,
    /// Use one serialized SkirtDesign instead of the default three profiles.
    #[arg(long)]
    design: Option<PathBuf>,
}

struct Variant {
    name: &'static str,
    profile: SkirtProfile,
}

fn variants() -> [Variant; 3] {
    [
        Variant {
            name: "straight_skirt",
            profile: SkirtProfile::Straight,
        },
        Variant {
            name: "flared_skirt",
            profile: SkirtProfile::Flared {
                width: Permille(1600),
                depth: Permille(1600),
            },
        },
        Variant {
            name: "bell_skirt",
            profile: SkirtProfile::Bell {
                width: Permille(2200),
                depth: Permille(1800),
                roundness: Permille(1000),
            },
        },
    ]
}

fn main() -> Result<()> {
    let args = Args::parse();
    let bytes = std::fs::read(&args.body)?;
    let body: Body = serde_json::from_slice(&bytes)?;
    std::fs::create_dir_all(&args.output)?;
    std::fs::write(args.output.join("body.json"), &bytes)?;
    let gpu = ArmorGpu::open()?;
    let device = DeviceBody::default();
    let samples = MorphBodies::load(&args.morph_bodies)?;
    let names = samples.names();
    let morphs = samples.realizations();
    let fit = body.fit_body();
    let wearer = body.realization(&device);
    let custom = args
        .design
        .as_ref()
        .map(|path| -> Result<SkirtDesign> { Ok(serde_json::from_slice(&std::fs::read(path)?)?) })
        .transpose()?;
    for Variant { name, profile } in
        variants()
            .into_iter()
            .take(if custom.is_some() { 1 } else { 3 })
    {
        let mut skirt = custom.clone().unwrap_or(SkirtDesign {
            profile,
            ..Default::default()
        });
        if args.dress {
            skirt.bodice = Some(Default::default());
        }
        let design = ParametricDesign::Skirt(skirt);
        eprintln!("Fitting {name}");
        let piece = device_fit::fit_recipe(&gpu, &fit, &wearer, &morphs, &design, "worn", &[])?;
        let generated = device_fit::assemble_recipe(
            &design,
            piece,
            &Fitted {
                placement: "worn",
                morphs: &names,
                domain: "review_body",
                joint_names: &body.joint_names,
                joints: &body.joints,
            },
        )?;
        review::write_generated(
            &args.output,
            &format!("{name}--worn"),
            "worn",
            &design,
            &generated,
        )?;
    }
    Ok(())
}
