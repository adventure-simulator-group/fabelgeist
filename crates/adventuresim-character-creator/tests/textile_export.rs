//! Native preview and portable garment files retain the same finish contract.
#![cfg(feature = "offline-creator")]
use adventuresim_character_creator::{export::*, textile_surface};
use anyhow::Result;
use bevy::prelude::*;
use fabelgeist_armor::{TextileAppearance, TextileColor, TextileFinish};
use fabelgeist_rig::RigJointName;

#[test]
fn portable_garment_finish_matches_its_native_material() -> Result<()> {
    let directory = std::env::temp_dir().join(format!("textile-optics-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    let positions = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let normals = [[0., 0., 1.]; 3];
    let joints = [[0; 8]; 3];
    let weights = [[1., 0., 0., 0., 0., 0., 0., 0.]; 3];
    for (index, finish) in [TextileFinish::Velvet, TextileFinish::Silk]
        .into_iter()
        .enumerate()
    {
        let fabric = TextileAppearance::solid(TextileColor([65, 9, 16]), finish);
        let native = textile_surface::material(&fabric, &mut Assets::default())?;
        let path = directory.join(format!("finish-{index}.glb"));
        let shell = RiggedShell {
            textile_finish: Some(finish),
            plate_edges: &[],
            surface: None,
            textures: None,
            texcoords: Some(&[[0., 0.], [1., 0.], [0., 1.]]),
            hinge: None,
            name: "fabric",
            positions: &positions,
            normals: &normals,
            faces: &[[0, 1, 2]],
            joint_indices: Some(&joints),
            joint_weights: Some(&weights),
            morph_targets: &[],
            base_color: fabric.material().base_color,
            metallic: 0.,
            roughness: native.perceptual_roughness,
        };
        export_rigged_glb(
            GlbOutput::Standalone(&path),
            "fabric",
            1,
            4,
            &RiggedMesh {
                positions: &positions,
                normals: &normals,
                faces: &[],
                export_body: false,
                joint_indices: &joints,
                joint_weights: &weights,
                joint_names: &[RigJointName::ROOT],
                joint_parents: &[-1],
                global_joint_states: &[[0., 0., 0., 0., 0., 0., 1., 1.]],
                joint_proportions: &[],
                morph_targets: &[],
            },
            &[shell],
            &[],
        )?;
        let bytes = std::fs::read(&path)?;
        let parsed = gltf::Gltf::from_slice(&bytes)?;
        let document: serde_json::Value = serde_json::from_slice(
            &bytes[20..20 + u32::from_le_bytes(bytes[12..16].try_into()?) as usize],
        )?;
        let factor =
            document["materials"][0]["extensions"]["KHR_materials_specular"]["specularFactor"]
                .as_f64()
                .unwrap() as f32;
        assert_eq!(factor * 0.5, native.reflectance);
        assert_eq!(
            parsed
                .materials()
                .next()
                .unwrap()
                .pbr_metallic_roughness()
                .roughness_factor(),
            native.perceptual_roughness
        );
        assert!(
            document["extensionsUsed"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("KHR_materials_specular"))
        );
        std::fs::remove_file(&path)?;
    }
    std::fs::remove_dir(directory)?;
    Ok(())
}
