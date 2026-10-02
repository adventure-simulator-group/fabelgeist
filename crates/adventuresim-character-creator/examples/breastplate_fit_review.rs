//! Reproduce the fitted torso plate from a saved body and typed design.
//! Usage: breastplate_fit_review BODY.json DESIGN.json OUTPUT.json
use adventuresim_armor_model::{BreastplateDesign, generate_breastplate};
use adventuresim_character_creator::breastplate::{TorsoSurfaceInput, build_front_torso_surface};
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
struct Body {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    texcoords: Vec<[f32; 2]>,
    texcoord_faces: Vec<[u32; 3]>,
    joint_indices: Vec<[u32; 8]>,
    joint_weights: Vec<[f32; 8]>,
    joint_names: Vec<String>,
    joints: Vec<[f32; 8]>,
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    anyhow::ensure!(args.len() == 3, "expected BODY DESIGN OUTPUT");
    let body: Body = serde_json::from_slice(&std::fs::read(&args[0])?)?;
    let design: BreastplateDesign = serde_json::from_slice(&std::fs::read(&args[1])?)?;
    let surface = build_front_torso_surface(TorsoSurfaceInput {
        domain: "mhr_body_v1",
        positions: &body.positions,
        normals: &body.normals,
        faces: &body.faces,
        texcoords: &body.texcoords,
        texcoord_faces: &body.texcoord_faces,
        joint_indices: &body.joint_indices,
        joint_weights: &body.joint_weights,
        joint_names: &body.joint_names,
        global_joint_states: &body.joints,
        morphs: &[],
    })
    .map_err(anyhow::Error::msg)?;
    let mesh = generate_breastplate(&design, &surface)?;
    std::fs::write(
        &args[2],
        serde_json::to_vec(&serde_json::json!({
            "id": "breastplate", "placement": "worn", "design": design,
            "positions": mesh.positions, "normals": mesh.normals, "indices": mesh.indices,
            "joint_indices": mesh.joint_indices, "joint_weights": mesh.joint_weights,
            "joint_names": body.joint_names, "joints": body.joints,
            "components": mesh.components, "plate_edges": mesh.plate_edges,
        }))?,
    )?;
    Ok(())
}
