use super::*;

use adventuresim_armor_model::GeneratedArmor;
use adventuresim_character_creator::{
    design_input::{load_bracer_design, load_breastplate_design},
    runtime_equipment::RuntimeBody,
};
use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode, GltfSkin},
    mesh::{VertexAttributeValues, morph::MorphAttributes, skinning::SkinnedMeshInverseBindposes},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

mod body;
use body::try_build_runtime_body;

#[derive(Component)]
pub(super) struct RuntimeEquipmentPresentation {
    pub(super) item: Entity,
    pub(super) item_id: String,
    pub(super) placement_id: String,
}

#[derive(Resource, Default)]
pub(super) struct RuntimeEquipmentBodyCache {
    pub(super) body: Option<RuntimeBody>,
    pub(super) inverse_bindposes: Option<Handle<SkinnedMeshInverseBindposes>>,
    pub(super) bracer_design: Option<adventuresim_armor_model::BracerDesign>,
    pub(super) breastplate_design: Option<adventuresim_armor_model::BreastplateDesign>,
    pub(super) failed: bool,
}

struct PreparedRuntimeEquipment<'a> {
    body: &'a RuntimeBody,
    inverse_bindposes: &'a Handle<SkinnedMeshInverseBindposes>,
    bracer_design: &'a adventuresim_armor_model::BracerDesign,
    breastplate_design: &'a adventuresim_armor_model::BreastplateDesign,
}

struct RuntimeEquipmentAssets<'a> {
    meshes: &'a mut Assets<Mesh>,
    images: &'a mut Assets<Image>,
    materials: &'a mut Assets<StandardMaterial>,
}

#[expect(clippy::too_many_arguments)]
pub(super) fn generate_runtime_equipment_models(
    mut commands: Commands,
    animation: Res<crate::animation::AnimationRuntime>,
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_skins: Res<Assets<GltfSkin>>,
    mut meshes: ResMut<Assets<Mesh>>,
    inverse_bindposes: Res<Assets<SkinnedMeshInverseBindposes>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cache: ResMut<RuntimeEquipmentBodyCache>,
    pending: Query<(Entity, &RuntimeEquipmentPresentation), Without<ProceduralEquipmentResolved>>,
) {
    if cache.failed {
        return;
    }

    if cache.body.is_none() {
        let Some(base_handle) = animation.requested_base() else {
            return;
        };
        let Some(body) = try_build_runtime_body(
            base_handle,
            &gltfs,
            &gltf_meshes,
            &gltf_nodes,
            &gltf_skins,
            &meshes,
            &inverse_bindposes,
        ) else {
            return;
        };

        match body {
            Ok((body, inverse_bindposes_handle)) => {
                cache.body = Some(body);
                cache.inverse_bindposes = Some(inverse_bindposes_handle);
            }
            Err(error) => {
                error!("failed to prepare the runtime armor body: {error:#}");
                cache.failed = true;
                return;
            }
        }
    }

    if cache.bracer_design.is_none() {
        match load_bracer_design(None) {
            Ok(design) => cache.bracer_design = Some(design),
            Err(error) => {
                error!("failed to load the runtime bracer design: {error:#}");
                cache.failed = true;
                return;
            }
        }
    }
    if cache.breastplate_design.is_none() {
        match load_breastplate_design(None) {
            Ok(design) => cache.breastplate_design = Some(design),
            Err(error) => {
                error!("failed to load the runtime breastplate design: {error:#}");
                cache.failed = true;
                return;
            }
        }
    }

    let Some(body) = cache.body.as_ref() else {
        return;
    };
    let Some(inverse_bindposes_handle) = cache.inverse_bindposes.as_ref() else {
        return;
    };
    let Some(bracer_design) = cache.bracer_design.as_ref() else {
        return;
    };
    let Some(breastplate_design) = cache.breastplate_design.as_ref() else {
        return;
    };

    let prepared = PreparedRuntimeEquipment {
        body,
        inverse_bindposes: inverse_bindposes_handle,
        bracer_design,
        breastplate_design,
    };
    let mut assets = RuntimeEquipmentAssets {
        meshes: &mut meshes,
        images: &mut images,
        materials: &mut materials,
    };
    for (entity, presentation) in &pending {
        spawn_runtime_equipment(&mut commands, entity, presentation, &prepared, &mut assets);
    }
}

fn spawn_runtime_equipment(
    commands: &mut Commands,
    entity: Entity,
    presentation: &RuntimeEquipmentPresentation,
    prepared: &PreparedRuntimeEquipment,
    assets: &mut RuntimeEquipmentAssets,
) {
    let generated = match generate_runtime_equipment(presentation, prepared) {
        Ok(generated) => generated,
        Err(error) => {
            error!(
                item = %presentation.item_id,
                placement = %presentation.placement_id,
                "failed to generate runtime equipment: {error:#}"
            );
            commands.entity(entity).insert(ProceduralEquipmentResolved);
            return;
        }
    };
    let mesh = runtime_equipment_mesh(&generated, assets.meshes);
    let material = runtime_equipment_material(
        &presentation.item_id,
        &generated,
        assets.images,
        assets.materials,
    );
    let part = ProceduralEquipmentPart::new(
        presentation.item,
        prepared.inverse_bindposes.clone(),
        prepared.body.joint_names.clone(),
    );
    commands.entity(entity).with_children(|commands| {
        commands.spawn(part.render_bundle(
            format!("Runtime armor {}", presentation.item_id),
            mesh,
            material,
        ));
    });
    commands
        .entity(entity)
        .insert((ProceduralEquipmentResolved, Visibility::Inherited));
}

fn generate_runtime_equipment(
    presentation: &RuntimeEquipmentPresentation,
    prepared: &PreparedRuntimeEquipment,
) -> anyhow::Result<GeneratedArmor> {
    if adventuresim_character_creator::runtime_equipment::is_runtime_armor(&presentation.item_id) {
        adventuresim_character_creator::runtime_equipment::generate_runtime_armor(
            prepared.body,
            &presentation.item_id,
            &presentation.placement_id,
            prepared.bracer_design,
            prepared.breastplate_design,
        )
    } else {
        adventuresim_character_creator::runtime_equipment::generate_runtime_clothing(
            prepared.body,
            &presentation.item_id,
            &presentation.placement_id,
        )
    }
}

fn runtime_equipment_mesh(armor: &GeneratedArmor, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, armor.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, armor.normals.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, armor.texcoords.clone())
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(
            armor
                .joint_indices
                .iter()
                .zip(armor.joint_weights.iter())
                .map(|(joint, weight)| strongest_four(*joint, *weight).0)
                .collect::<Vec<[u16; 4]>>(),
        ),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_WEIGHT,
        armor
            .joint_weights
            .iter()
            .zip(armor.joint_indices.iter())
            .map(|(weights, joints)| strongest_four(*joints, *weights).1)
            .collect::<Vec<[f32; 4]>>(),
    )
    .with_inserted_indices(Indices::U32(armor.indices.clone()));

    let morphs = armor
        .morphs
        .iter()
        .flat_map(|morph| {
            morph
                .position_deltas
                .iter()
                .zip(&morph.normal_deltas)
                .map(|(position, normal)| {
                    MorphAttributes::new(
                        Vec3::from_array(*position),
                        Vec3::from_array(*normal),
                        Vec3::ZERO,
                    )
                })
        })
        .collect::<Vec<_>>();
    let names = armor
        .morphs
        .iter()
        .map(|morph| morph.name.clone())
        .collect::<Vec<_>>();
    if !morphs.is_empty() {
        mesh.set_morph_targets(morphs);
        mesh.set_morph_target_names(names);
    }
    meshes.add(mesh)
}

fn runtime_equipment_material(
    item_id: &str,
    armor: &GeneratedArmor,
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    let material =
        adventuresim_character_creator::runtime_equipment::runtime_equipment_material(item_id)
            .unwrap_or_else(|| panic!("runtime armor material missing for {item_id}"));
    let (base_color, metallic, roughness) = adventuresim_character_creator::equipment_pbr(material);
    let normal_map = armor.normal_map.as_ref().map(|normal_map| {
        images.add(Image::new(
            Extent3d {
                width: normal_map.width,
                height: normal_map.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            normal_map.rgba8.clone(),
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        ))
    });
    materials.add(StandardMaterial {
        base_color: Color::srgba(base_color[0], base_color[1], base_color[2], base_color[3]),
        metallic,
        perceptual_roughness: roughness,
        normal_map_texture: normal_map,
        ..default()
    })
}

fn strongest_four(joints: [u32; 8], weights: [f32; 8]) -> ([u16; 4], [f32; 4]) {
    let mut combined = std::collections::BTreeMap::<u32, f32>::new();
    for (joint, weight) in joints.into_iter().zip(weights) {
        *combined.entry(joint).or_default() += weight;
    }
    let mut influences = combined.into_iter().collect::<Vec<_>>();
    influences.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    influences.truncate(4);
    let sum = influences.iter().map(|(_, weight)| *weight).sum::<f32>();
    let mut result_joints = [0; 4];
    let mut result_weights = [0.0; 4];
    for (index, (joint, weight)) in influences.into_iter().enumerate() {
        result_joints[index] = joint as u16;
        result_weights[index] = weight / sum;
    }
    (result_joints, result_weights)
}
