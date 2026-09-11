//! Refit a recipe on a saved body review without loading the MHR model.
//!
//! cargo run --manifest-path crates/adventuresim-character-creator/Cargo.toml
//! --example armor_fit_review -- BODY.json DESIGN.json PLACEMENT OUTPUT.json
//! [SUPPORT.json ...]
use adventuresim_character_creator::{
    armor_frames::Wearer, armor_layer::ArmorLayerSurface, armor_recipes,
};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct Body {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    joint_indices: Vec<[u32; 8]>,
    joint_weights: Vec<[f32; 8]>,
    joint_names: Vec<String>,
    joints: Vec<[f32; 8]>,
}

#[derive(Deserialize)]
struct Support {
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

fn read<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    anyhow::ensure!(
        args.len() >= 4,
        "expected BODY DESIGN PLACEMENT OUTPUT [SUPPORT ...]"
    );
    let body: Body = read(&args[0])?;
    let design: armor_recipes::ParametricDesign = read(&args[1])?;
    let support = args[4..]
        .iter()
        .map(read)
        .collect::<Result<Vec<Support>>>()?;
    let layers = support
        .iter()
        .map(|s| ArmorLayerSurface {
            positions: &s.positions,
            faces: s.indices.as_chunks::<3>().0,
            relief: adventuresim_armor_model::Millimeters(0),
        })
        .collect::<Vec<_>>();
    let wearer = Wearer {
        positions: &body.positions,
        normals: &body.normals,
        faces: &body.faces,
        joint_indices: &body.joint_indices,
        joint_weights: &body.joint_weights,
        joint_names: &body.joint_names,
        joints: &body.joints,
    };
    let mesh = armor_recipes::fitted_mesh(&design, &args[2], &wearer, &layers)?;
    let output = Path::new(&args[3]);
    std::fs::create_dir_all(
        output
            .parent()
            .context("output requires a parent directory")?,
    )?;
    std::fs::write(
        output,
        serde_json::to_vec(&serde_json::json!({
            "id": "review", "placement": args[2], "design": design,
            "positions": mesh.positions, "normals": mesh.normals()?, "indices": mesh.indices,
        }))?,
    )?;
    Ok(())
}
