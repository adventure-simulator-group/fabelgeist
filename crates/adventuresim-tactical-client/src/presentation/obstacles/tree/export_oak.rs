//! Untracked scratch module: dumps the procedural English oak LOD submeshes as
//! JSON so an external script can assemble a static glTF binary.
//!
//! Reachable only through a temporary `#[cfg(test)] mod export_oak;` line in
//! `tree/mod.rs`. Run from the repository root with:
//!
//! ```text
//! cargo test -p adventuresim-tactical-client --bin adventuresim-tactical-client \
//!     export_english_oak_lod_meshes -- --ignored --nocapture
//! ```
//!
//! Output goes to `target/oak-export/` (override with `OAK_EXPORT_DIR`).

use std::path::{Path, PathBuf};

use adventuresim_tactical_core::prelude::TREE_TRUNK_HEIGHT_METRES;
use bevy::{mesh::VertexAttributeValues, prelude::Mesh};

use super::{
    WoodyBranchMeshQuality, procedural_oak_bud_mesh, procedural_oak_leaf_card_mesh,
    procedural_oak_leaves, procedural_oak_textured_leaf_mesh, procedural_tree_branch_mesh,
    procedural_tree_skeleton, procedural_woody_crown_mesh,
    procedural_woody_sparse_leaf_card_mesh,
};
use super::geometry::{ENGLISH_OAK_PARAMETERS, NATURAL_OAK_GNARLING};

const SEED: u64 = 42;
const CANOPY_COMPETITION: f32 = 0.0;

fn dump(name: &str, mesh: &Mesh, dir: &Path) -> serde_json::Value {
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(VertexAttributeValues::as_float3)
        .expect("positions")
        .to_vec();
    let normals = mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .and_then(VertexAttributeValues::as_float3)
        .expect("normals")
        .to_vec();
    let uv0 = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(values)) => Some(values.clone()),
        _ => None,
    };
    let colors = match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(values)) => Some(values.clone()),
        _ => None,
    };
    let indices = mesh
        .indices()
        .expect("indexed mesh")
        .iter()
        .map(|index| index as u32)
        .collect::<Vec<_>>();
    let summary = serde_json::json!({
        "name": name,
        "vertices": positions.len(),
        "triangles": indices.len() / 3,
        "has_uv0": uv0.is_some(),
        "has_colors": colors.is_some(),
    });
    let value = serde_json::json!({
        "name": name,
        "positions": positions,
        "normals": normals,
        "uv0": uv0,
        "colors": colors,
        "indices": indices,
    });
    std::fs::write(
        dir.join(format!("{name}.json")),
        serde_json::to_vec(&value).expect("serialize"),
    )
    .expect("write mesh json");
    println!("{summary}");
    summary
}

#[test]
#[ignore = "scratch export for the static oak glTF; run explicitly"]
fn export_english_oak_lod_meshes() {
    let dir = std::env::var_os("OAK_EXPORT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/oak-export")
        });
    std::fs::create_dir_all(&dir).expect("create export dir");

    let branches = procedural_tree_skeleton(SEED, CANOPY_COMPETITION);
    let leaves = procedural_oak_leaves(SEED, &branches, CANOPY_COMPETITION);

    let mut meshes = Vec::new();
    // LOD0: full-detail trunk + roots + every branch to depth 3, terminal buds,
    // cambered textured leaves (TreeLeafRepresentation::TexturedMesh).
    meshes.push(dump("lod0_wood", &procedural_tree_branch_mesh(&branches, 3), &dir));
    meshes.push(dump("lod0_buds", &procedural_oak_bud_mesh(&branches), &dir));
    meshes.push(dump("lod0_leaves", &procedural_oak_textured_leaf_mesh(&leaves), &dir));
    // Trunk + roots alone (depth 0, full detail); stays resident through LOD2
    // in the streamed presentation and is merged with the aggregate crowns.
    meshes.push(dump("trunk", &procedural_tree_branch_mesh(&branches, 0), &dir));
    // Source LOD1: aggregate crown wood to depth 2, flat alpha leaf cards.
    meshes.push(dump(
        "lod1_crown",
        &procedural_woody_crown_mesh(&branches, 2, WoodyBranchMeshQuality::AggregateLod1),
        &dir,
    ));
    meshes.push(dump("lod1_leaves", &procedural_oak_leaf_card_mesh(&leaves), &dir));
    // Source LOD2: aggregate crown wood to depth 1, sparse enlarged leaf cards.
    meshes.push(dump(
        "lod2_crown",
        &procedural_woody_crown_mesh(&branches, 1, WoodyBranchMeshQuality::AggregateLod2),
        &dir,
    ));
    meshes.push(dump("lod2_leaves", &procedural_woody_sparse_leaf_card_mesh(&leaves), &dir));

    let meta = serde_json::json!({
        "seed": SEED,
        "canopy_competition": CANOPY_COMPETITION,
        "species": "ENGLISH_OAK_PARAMETERS",
        "gnarling": format!("NATURAL_OAK_GNARLING {NATURAL_OAK_GNARLING:?}"),
        "parameters": format!("{ENGLISH_OAK_PARAMETERS:?}"),
        "ground_y": -TREE_TRUNK_HEIGHT_METRES * 0.5,
        "branch_segments": branches.len(),
        "leaves": leaves.len(),
        "bark_uv": "u = 1 m circumference tiles, v = 2 m axial tiles (shader multiplies both by relief.x = 2 for the 0.5 m oak-bark tile)",
        "meshes": meshes,
    });
    std::fs::write(
        dir.join("meta.json"),
        serde_json::to_vec_pretty(&meta).expect("serialize meta"),
    )
    .expect("write meta");
    println!("exported {} branch segments, {} leaves to {}", branches.len(), leaves.len(), dir.display());
}
