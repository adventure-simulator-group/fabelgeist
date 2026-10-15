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

fn main() -> Result<()> {
    let args = Args::parse();
    let bytes = std::fs::read(&args.body)?;
    let body: Body = serde_json::from_slice(&bytes)?;
    std::fs::create_dir_all(&args.output)?;
    std::fs::write(args.output.join("body.json"), &bytes)?;
    let gpu = ArmorGpu::open()?;
    let device = DeviceBody::default();
    let fit = FitBody {
        faces: &body.faces,
        texcoords: &body.texcoords,
        joint_indices: &body.joint_indices,
        joint_weights: &body.joint_weights,
        joint_names: &body.joint_names,
    };
    let wearer = Realization {
        positions: &body.positions,
        normals: &body.normals,
        joints: &body.joints,
        device: &device,
    };
    for Variant { name, profile } in [
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
    ] {
        let design = ParametricDesign::Skirt(SkirtDesign {
            profile,
            ..Default::default()
        });
        eprintln!("Fitting {name}");
        let piece = device_fit::fit_recipe(&gpu, &fit, &wearer, &[], &design, "worn", &[])?;
        let generated = device_fit::assemble_recipe(
            &design,
            piece,
            &Fitted {
                placement: "worn",
                morphs: &[],
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
                "generator_version": fabelgeist_armor::GENERATOR_VERSION,
            }))?,
        )?;
    }
    Ok(())
}
