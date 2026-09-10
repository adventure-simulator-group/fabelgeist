//! Resolve every authored equipment node before replacing its loading placeholder.

use super::*;
use bevy::gltf::GltfExtras;

struct LoadedPart {
    name: String,
    mesh: Handle<Mesh>,
    material_index: usize,
    inverse_bindposes: Handle<SkinnedMeshInverseBindposes>,
    joint_names: Vec<String>,
    extras: Option<GltfExtras>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::gltf::GltfPrimitive;

    #[test]
    fn loads_every_named_node_and_primitive_before_resolving() {
        let mut meshes = Assets::<GltfMesh>::default();
        let mut nodes = Assets::<GltfNode>::default();
        let mut skins = Assets::<GltfSkin>::default();
        let head = nodes.add(GltfNode {
            index: 0,
            name: "c_head".into(),
            children: vec![],
            mesh: None,
            skin: None,
            transform: Transform::IDENTITY,
            is_animation_root: false,
            extras: None,
        });
        let skin = skins.add(GltfSkin {
            index: 0,
            name: "head".into(),
            joints: vec![head.clone()],
            inverse_bind_matrices: default(),
            extras: None,
        });
        let material = Handle::default();
        let mut mesh_handles = Vec::new();
        let mut node_handles = vec![head];
        for (index, name) in ["skull", "bevor", "visor"].into_iter().enumerate() {
            let mesh = meshes.add(GltfMesh {
                index,
                name: name.into(),
                extras: None,
                primitives: (0..if index == 1 { 2 } else { 1 })
                    .map(|primitive| GltfPrimitive {
                        index: primitive,
                        parent_mesh_index: index,
                        name: name.into(),
                        mesh: default(),
                        material: Some(material.clone()),
                        extras: None,
                        material_extras: None,
                    })
                    .collect(),
            });
            mesh_handles.push(mesh.clone());
            node_handles.push(nodes.add(GltfNode {
                index: index + 1,
                name: name.into(),
                children: vec![],
                mesh: Some(mesh),
                skin: Some(skin.clone()),
                transform: Transform::IDENTITY,
                is_animation_root: false,
                extras: Some(GltfExtras {
                    value: format!("{{\"part\":\"{name}\"}}"),
                }),
            }));
        }
        let gltf = Gltf {
            scenes: vec![],
            named_scenes: default(),
            meshes: mesh_handles,
            named_meshes: default(),
            materials: vec![material],
            named_materials: default(),
            nodes: node_handles,
            named_nodes: default(),
            skins: vec![skin],
            named_skins: default(),
            default_scene: None,
            animations: vec![],
            named_animations: default(),
            source: None,
        };
        let assets = EquipmentAssets {
            meshes: &meshes,
            skins: &skins,
            nodes: &nodes,
        };
        let parts = assets.parts(&gltf).unwrap().unwrap();
        assert_eq!(
            parts
                .iter()
                .map(|part| part.name.as_str())
                .collect::<Vec<_>>(),
            ["skull", "bevor", "bevor", "visor"]
        );
        assert!(parts.iter().all(|part| part.joint_names == ["c_head"]));
        assert_eq!(
            parts[3].extras.as_ref().unwrap().value,
            "{\"part\":\"visor\"}"
        );
        meshes.get_mut(&gltf.meshes[2]).unwrap().primitives[0].material = None;
        let assets = EquipmentAssets {
            meshes: &meshes,
            skins: &skins,
            nodes: &nodes,
        };
        assert!(matches!(
            assets.parts(&gltf),
            Some(Err(InvalidEquipment::NoMaterial))
        ));
        meshes.remove(&gltf.meshes[2]);
        let assets = EquipmentAssets {
            meshes: &meshes,
            skins: &skins,
            nodes: &nodes,
        };
        assert!(assets.parts(&gltf).is_none());
    }
}

#[derive(Debug)]
enum InvalidEquipment {
    NoMesh,
    NoPrimitive,
    NoSkin,
    NoMaterial,
    TransformedMesh,
}

struct EquipmentAssets<'a> {
    meshes: &'a Assets<GltfMesh>,
    skins: &'a Assets<GltfSkin>,
    nodes: &'a Assets<GltfNode>,
}

impl EquipmentAssets<'_> {
    fn parts(&self, gltf: &Gltf) -> Option<Result<Vec<LoadedPart>, InvalidEquipment>> {
        let mut parts = Vec::new();
        for handle in &gltf.nodes {
            let node = self.nodes.get(handle)?;
            let Some(mesh) = &node.mesh else { continue };
            if node.transform != Transform::IDENTITY {
                return Some(Err(InvalidEquipment::TransformedMesh));
            }
            let mesh = self.meshes.get(mesh)?;
            if mesh.primitives.is_empty() {
                return Some(Err(InvalidEquipment::NoPrimitive));
            }
            let Some(skin) = node.skin.as_ref() else {
                return Some(Err(InvalidEquipment::NoSkin));
            };
            let skin = self.skins.get(skin)?;
            let joint_names = skin
                .joints
                .iter()
                .map(|joint| self.nodes.get(joint).map(|node| node.name.clone()))
                .collect::<Option<Vec<_>>>()?;
            for primitive in &mesh.primitives {
                let Some(material_index) = primitive.material.as_ref().and_then(|material| {
                    gltf.materials
                        .iter()
                        .position(|candidate| candidate.id() == material.id())
                }) else {
                    return Some(Err(InvalidEquipment::NoMaterial));
                };
                parts.push(LoadedPart {
                    name: node.name.clone(),
                    mesh: primitive.mesh.clone(),
                    material_index,
                    inverse_bindposes: skin.inverse_bind_matrices.clone(),
                    joint_names: joint_names.clone(),
                    extras: node.extras.clone(),
                });
            }
        }
        Some(if parts.is_empty() {
            Err(InvalidEquipment::NoMesh)
        } else {
            Ok(parts)
        })
    }

    fn sockets(&self, gltf: &Gltf) -> Option<BTreeMap<String, Transform>> {
        gltf.named_nodes
            .iter()
            .filter_map(|(name, node)| {
                name.strip_prefix(EQUIPMENT_SOCKET_NODE_PREFIX)
                    .map(|id| (id, node))
            })
            .map(|(id, node)| {
                self.nodes
                    .get(node)
                    .map(|node| (id.to_owned(), node.transform))
            })
            .collect()
    }
}

impl LoadedPart {
    fn spawn(
        self,
        commands: &mut Commands,
        root: Entity,
        item: Entity,
        path: &str,
        server: &AssetServer,
    ) {
        let label = format!(
            "{}/std",
            GltfAssetLabel::Material {
                index: self.material_index,
                is_scale_inverted: false,
            }
        );
        let material = server.load(format!("{path}#{label}"));
        let mut entity = commands.spawn((
            ProceduralEquipmentPart::new(item, self.inverse_bindposes, self.joint_names)
                .render_bundle(self.name, self.mesh, material),
            ChildOf(root),
        ));
        if let Some(extras) = self.extras {
            entity.insert(extras);
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "Bevy injects the equipment asset stores and resolution queries independently"
)]
pub(super) fn resolve_procedural_equipment_models(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltf_skins: Res<Assets<GltfSkin>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    pending: Query<
        (
            Entity,
            &ItemPlaceholder,
            &ProceduralEquipmentPresentation,
            &ProceduralEquipmentRequest,
        ),
        (
            Without<ProceduralEquipmentResolved>,
            Without<ProceduralEquipmentFailed>,
        ),
    >,
    fallbacks: Query<(Entity, &ItemFallback)>,
) {
    let assets = EquipmentAssets {
        meshes: &gltf_meshes,
        skins: &gltf_skins,
        nodes: &gltf_nodes,
    };
    for (root, placeholder, presentation, request) in &pending {
        if matches!(
            asset_server.load_state(request.0.id()),
            LoadState::Failed(_)
        ) {
            warn!(
                asset = presentation.asset_path,
                "Procedural equipment glTF failed to load"
            );
            commands.entity(root).insert(ProceduralEquipmentFailed);
            continue;
        }
        let Some(gltf) = gltfs.get(&request.0) else {
            continue;
        };
        let Some(parts) = assets.parts(gltf) else {
            continue;
        };
        let parts = match parts {
            Ok(parts) => parts,
            Err(reason) => {
                warn!(
                    asset = presentation.asset_path,
                    ?reason,
                    "Invalid procedural equipment asset"
                );
                commands.entity(root).insert(ProceduralEquipmentFailed);
                continue;
            }
        };
        let Some(sockets) = assets.sockets(gltf) else {
            continue;
        };
        for part in parts {
            part.spawn(
                &mut commands,
                root,
                placeholder.0,
                &presentation.asset_path,
                &asset_server,
            );
        }
        commands
            .entity(placeholder.0)
            .insert(EquipmentAttachmentSockets(sockets));
        for (fallback, item) in &fallbacks {
            if item.0 == placeholder.0 {
                commands.entity(fallback).despawn();
            }
        }
        commands.entity(root).insert(ProceduralEquipmentResolved);
    }
}
