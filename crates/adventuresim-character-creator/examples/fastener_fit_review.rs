//! Fit closure hardware to saved generator output without loading MHR.
use adventuresim_armor_model::{ArmorComponentRole, PartMesh};
use adventuresim_character_creator::{armor_frames::Wearer, fasteners};
use anyhow::Result;
use serde::Deserialize;

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
struct Plate {
    id: String,
    placement: String,
    design: serde_json::Value,
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
    components: Vec<adventuresim_armor_model::ArmorComponent>,
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    anyhow::ensure!(
        (3..=4).contains(&args.len()),
        "expected BODY.json PLATE.json OUTPUT.json [FASTENERS.json]"
    );
    let body: Body = serde_json::from_slice(&std::fs::read(&args[0])?)?;
    let plate: Plate = serde_json::from_slice(&std::fs::read(&args[1])?)?;
    let wearer = Wearer {
        positions: &body.positions,
        normals: &body.normals,
        faces: &body.faces,
        joint_indices: &body.joint_indices,
        joint_weights: &body.joint_weights,
        joint_names: &body.joint_names,
        joints: &body.joints,
    };
    let catalog = fasteners::catalog::load(args.get(3).map(std::path::Path::new))?;
    let recipe = catalog.get(&plate.id).expect("item has authored fasteners");
    let mut mesh = PartMesh::new();
    mesh.positions = plate.positions;
    mesh.indices = plate.indices;
    mesh.components = plate.components;
    if mesh.components.is_empty() {
        mesh = mesh.with_component(ArmorComponentRole::Plate, None);
    }
    let support = recipe
        .support_item()
        .map(|id| {
            let path = std::path::Path::new(&args[1])
                .parent()
                .unwrap()
                .join(format!("{id}--{}.json", plate.placement));
            let row: Plate = serde_json::from_slice(&std::fs::read(path)?)?;
            let mut mesh = PartMesh::new();
            mesh.positions = row.positions;
            mesh.indices = row.indices;
            Ok::<_, anyhow::Error>(mesh)
        })
        .transpose()?;
    let hardware = recipe.generate(&mesh, &wearer, &plate.placement, support.as_ref())?;
    mesh.append(hardware);
    std::fs::write(
        &args[2],
        serde_json::to_vec(&serde_json::json!({
            "id": plate.id, "placement": plate.placement, "design": plate.design,
            "positions": mesh.positions, "normals": mesh.normals()?, "indices": mesh.indices,
            "components": mesh.components,
        }))?,
    )?;
    Ok(())
}
