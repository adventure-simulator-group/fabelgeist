//! Export fitted skirt variants against an existing body review JSON.
//! This exercises the runtime fitter without requiring the offline body model.

use adventuresim_character_creator::{
    armor_recipes::ParametricDesign,
    device_body::DeviceBody,
    device_fit::{self, FitBody, Fitted, Realization},
};
use anyhow::Result;
use clap::Parser;
use fabelgeist_armor::{ArmorGpu, Permille, SkirtDesign, SkirtProfile};
use fabelgeist_rig::RigJointName;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    /// Body review JSON containing native mesh and rig attributes.
    body: PathBuf,
    /// Output consumed by scripts/render_armor_review.py.
    output: PathBuf,
    /// Additional direct-fit body realizations sharing the reference topology.
    #[arg(long = "morph-body")]
    morph_bodies: Vec<PathBuf>,
}

/// Native attributes at the review-file boundary, in canonical metres.
#[derive(Deserialize)]
struct Body {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    texcoords: Vec<[f32; 2]>,
    joint_indices: Vec<[u32; 8]>,
    joint_weights: Vec<[f32; 8]>,
    joint_names: Vec<RigJointName>,
    joints: Vec<[f32; 8]>,
}

struct Variant {
    name: &'static str,
    profile: SkirtProfile,
}

impl Body {
    fn fit_body(&self) -> FitBody<'_> {
        FitBody {
            faces: &self.faces,
            texcoords: &self.texcoords,
            joint_indices: &self.joint_indices,
            joint_weights: &self.joint_weights,
            joint_names: &self.joint_names,
        }
    }

    fn realization<'a>(&'a self, device: &'a DeviceBody) -> Realization<'a> {
        Realization {
            positions: &self.positions,
            normals: &self.normals,
            joints: &self.joints,
            device,
        }
    }
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
    let samples = args
        .morph_bodies
        .iter()
        .map(|path| Ok(serde_json::from_slice::<Body>(&std::fs::read(path)?)?))
        .collect::<Result<Vec<_>>>()?;
    let devices = samples
        .iter()
        .map(|_| DeviceBody::default())
        .collect::<Vec<_>>();
    let names = args
        .morph_bodies
        .iter()
        .map(|path| {
            path.file_stem()
                .and_then(|v| v.to_str())
                .ok_or_else(|| anyhow::anyhow!("invalid morph filename"))
        })
        .collect::<Result<Vec<_>>>()?;
    let morphs = samples
        .iter()
        .zip(&devices)
        .zip(&names)
        .map(|((body, device), name)| (*name, body.realization(device)))
        .collect::<Vec<_>>();
    let fit = body.fit_body();
    let wearer = body.realization(&device);
    for Variant { name, profile } in variants() {
        let design = ParametricDesign::Skirt(SkirtDesign {
            profile,
            ..Default::default()
        });
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
        std::fs::write(
            args.output.join(format!("{name}--worn.json")),
            serde_json::to_vec(&serde_json::json!({
                "id": name, "placement": "worn", "design": design,
                "positions": generated.positions, "normals": generated.normals,
                "indices": generated.indices, "components": generated.components,
                "joint_indices": generated.joint_indices, "joint_weights": generated.joint_weights,
                "morphs": generated.morphs.iter().map(|morph| serde_json::json!({
                    "name": morph.name, "positions": morph.direct_positions,
                    "normal_deltas": morph.normal_deltas,
                })).collect::<Vec<_>>(),
                "generator_version": fabelgeist_armor::GENERATOR_VERSION,
            }))?,
        )?;
    }
    Ok(())
}
