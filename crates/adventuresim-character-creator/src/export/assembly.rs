//! Preserve independently authored shells as named, skinned glTF mesh nodes.

use super::*;

pub(super) fn append(
    mut geometry: Value,
    mesh: &RiggedMesh<'_>,
    shells: &[RiggedShell<'_>],
    character_name: &str,
    nodes: &mut Vec<Value>,
    scene_nodes: &mut Vec<usize>,
) -> Vec<Value> {
    let parts = mesh
        .export_body
        .then_some((character_name, None, None))
        .into_iter()
        .chain(
            shells
                .iter()
                .map(|shell| (shell.name, shell.hinge, Some(shell))),
        );
    let primitives = geometry["primitives"].take().as_array().unwrap().clone();
    parts
        .zip(primitives)
        .enumerate()
        .map(|(index, ((name, hinge, shell), mut primitive))| {
            if let Some(shell) = shell.filter(|shell| !shell.plate_edges.is_empty()) {
                primitive["extras"] = json!({"adventuresim_plate_edges": {
                    "space": "reference_body", "units": "metres",
                    "segments": shell.plate_edges.iter().map(|edge| edge.map(|i| shell.positions[i as usize])).collect::<Vec<_>>()
                }});
            }
            let mut exported = json!({"name": name, "primitives": [primitive]});
            for key in ["weights", "extras"] {
                if let Some(value) = geometry.get(key) {
                    exported[key] = value.clone();
                }
            }
            let mut node = json!({"name": name, "mesh": index, "skin": 0});
            if let Some(hinge) = hinge {
                node["extras"] = json!({"adventuresim_hinge": {
                    "space": "reference_body", "origin": hinge.origin, "axis": hinge.axis
                }});
            }
            scene_nodes.push(nodes.len());
            nodes.push(node);
            exported
        })
        .collect()
}

pub(super) fn extras(
    character_name: &str,
    recipe_version: u8,
    lod: u8,
    shells: &[RiggedShell<'_>],
    sockets: &[RiggedSocket<'_>],
    attachments: &[Value],
) -> Value {
    let mut extras = json!({
        "adventuresim_character": {
            "name": character_name,
            "recipe_version": recipe_version,
            "mhr_release": "v1.0.1",
            "lod": lod,
            "equipment": shells.iter().map(|shell| shell.name).collect::<Vec<_>>(),
        },
        "adventuresim_rig": {
            "family": "mhr",
            "neutral_pose": "T-pose",
            "units": "metres",
            "up_axis": "+Y",
            "forward_axis": "-Z",
            "attachments": attachments,
        },
    });
    if !sockets.is_empty() {
        extras["adventuresim_equipment"] = json!({
            "attachment_sockets": sockets.iter().map(|socket| json!({
                "attachment_point_id": socket.attachment_point_id,
                "node": format!("{EQUIPMENT_SOCKET_NODE_PREFIX}{}", socket.attachment_point_id),
                "space": "pelvis_local",
                "surface_uv": {
                    "domain": socket.surface_uv_domain,
                    "uv": socket.surface_uv
                },
                "tangent_axis": "+Y",
                "normal_axis": "+Z"
            })).collect::<Vec<_>>(),
        });
    }
    extras
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_components_keep_names_hinges_skin_and_all_morphs() {
        let directory =
            std::env::temp_dir().join(format!("helmet-assembly-export-{}", std::process::id()));
        let path = directory.join("close_helmet.glb");
        let positions = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let normals = [[0.0, 0.0, 1.0]; 4];
        let texcoords = [[0.0, 0.0], [0.2, 0.3], [0.4, 0.6], [0.8, 0.9]];
        let maps = SurfaceTextures {
            base_color_png: include_bytes!(
                "../../../../assets_src/equipment/materials/mail-base-color.png"
            ),
            normal_png: include_bytes!(
                "../../../../assets_src/equipment/materials/mail-normal.png"
            ),
            occlusion_png: Some(include_bytes!(
                "../../../../assets_src/equipment/materials/mail-occlusion.png"
            )),
            cutout: true,
        };
        let joints = [[0; 8]; 4];
        let weights = [[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 4];
        let names = ["c_head".to_owned()];
        let states = [[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]];
        let target_names = (0..47)
            .map(|index| format!("fit_{index}"))
            .collect::<Vec<_>>();
        let deltas = [
            [0.1, 0.0, 0.0],
            [0.2, 0.0, 0.0],
            [0.3, 0.0, 0.0],
            [0.4, 0.0, 0.0],
        ];
        let targets = target_names
            .iter()
            .map(|name| RiggedMorphTarget {
                name,
                position_deltas: &deltas,
                normal_deltas: &[[0.0; 3]; 4],
            })
            .collect::<Vec<_>>();
        let hinge = adventuresim_armor_model::ArmorHinge {
            origin: [0.1, 1.7, 0.0],
            axis: [1.0, 0.0, 0.0],
        };
        let faces = [[1, 2, 3]];
        let shells = ["skull", "bevor", "visor"].map(|name| RiggedShell {
            plate_edges: &[[1, 2], [0, 1]],
            textures: Some(maps),
            texcoords: Some(&texcoords),
            name,
            hinge: (name != "skull").then_some(hinge),
            positions: &positions,
            normals: &normals,
            faces: &faces,
            joint_indices: Some(&joints),
            joint_weights: Some(&weights),
            morph_targets: &targets,
            base_color: [1.0; 4],
            metallic: 1.0,
            roughness: 0.2,
        });
        export_rigged_glb(
            GlbOutput::Standalone(&path),
            "close_helmet",
            1,
            1,
            &RiggedMesh {
                joint_proportions: &[],
                morph_targets: &[],
                positions: &positions,
                normals: &normals,
                faces: &[],
                export_body: false,
                joint_indices: &joints,
                joint_weights: &weights,
                joint_names: &names,
                joint_parents: &[-1],
                global_joint_states: &states,
            },
            &shells,
            &[],
        )
        .unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let parsed = gltf::Gltf::from_slice(&bytes).unwrap();
        assert_eq!(parsed.images().count(), 9);
        for (image, expected) in parsed.images().zip(
            [
                maps.base_color_png,
                maps.normal_png,
                maps.occlusion_png.unwrap(),
            ]
            .repeat(3),
        ) {
            let gltf::image::Source::View { view, mime_type } = image.source() else {
                panic!("surface images must be embedded");
            };
            assert_eq!(mime_type, "image/png");
            assert_eq!(
                &parsed.blob.as_ref().unwrap()[view.offset()..view.offset() + view.length()],
                expected
            );
        }
        assert_eq!(parsed.meshes().count(), 3);
        for (mesh, expected) in parsed.meshes().zip(["skull", "bevor", "visor"]) {
            assert_eq!(mesh.name(), Some(expected));
            assert_eq!(mesh.weights().unwrap(), &[0.0; 47]);
            let primitive = mesh.primitives().next().unwrap();
            let extras: Value =
                serde_json::from_str(primitive.extras().as_ref().unwrap().get()).unwrap();
            assert_eq!(
                extras["adventuresim_plate_edges"]["segments"],
                json!([[positions[1], positions[2]]])
            );
            let reader = primitive.reader(|_| parsed.blob.as_deref());
            assert_eq!(
                reader
                    .read_tex_coords(0)
                    .unwrap()
                    .into_f32()
                    .collect::<Vec<_>>(),
                texcoords[1..]
            );
            assert_eq!(
                primitive.material().alpha_mode(),
                gltf::material::AlphaMode::Mask
            );
            assert!(primitive.material().normal_texture().is_some());
            let material = primitive.material();
            let color = material
                .pbr_metallic_roughness()
                .base_color_texture()
                .unwrap();
            let normal = material.normal_texture().unwrap();
            let occlusion = material.occlusion_texture().unwrap();
            assert_eq!(occlusion.strength(), 1.0);
            assert_ne!(color.texture().index(), normal.texture().index());
            assert_ne!(color.texture().index(), occlusion.texture().index());
            assert_ne!(normal.texture().index(), occlusion.texture().index());
            for (texture, expected) in [
                (color.texture(), maps.base_color_png),
                (normal.texture(), maps.normal_png),
                (occlusion.texture(), maps.occlusion_png.unwrap()),
            ] {
                let gltf::image::Source::View { view, .. } = texture.source().source() else {
                    panic!("material channels must use their embedded images");
                };
                assert_eq!(
                    &parsed.blob.as_ref().unwrap()[view.offset()..view.offset() + view.length()],
                    expected
                );
            }
            assert_eq!(
                reader.read_positions().unwrap().collect::<Vec<_>>(),
                positions[1..]
            );
            assert_eq!(
                reader
                    .read_joints(0)
                    .unwrap()
                    .into_u16()
                    .collect::<Vec<_>>(),
                vec![[0; 4]; 3]
            );
            let morphs = reader.read_morph_targets().collect::<Vec<_>>();
            assert_eq!(morphs.len(), 47);
            for (positions, _, _) in morphs {
                assert_eq!(positions.unwrap().collect::<Vec<_>>(), deltas[1..]);
            }
        }
        let size = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        let document: Value = serde_json::from_slice(&bytes[20..20 + size]).unwrap();
        let nodes = document["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|node| node.get("mesh").is_some())
            .collect::<Vec<_>>();
        assert_eq!(nodes.len(), 3);
        for (index, node) in nodes.iter().enumerate() {
            assert_eq!(node["mesh"], index);
            assert_eq!(node["skin"], 0);
            if index > 0 {
                assert_eq!(
                    node["extras"]["adventuresim_hinge"]["space"],
                    "reference_body"
                );
                assert_eq!(
                    node["extras"]["adventuresim_hinge"]["axis"],
                    json!(hinge.axis)
                );
            }
        }
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
