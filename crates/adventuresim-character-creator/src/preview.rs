//! Spawn generated clothing and armor into the studio preview.

use super::*;
use adventuresim_character_creator::clothing::ClothingShell;

pub(super) fn spawn_clothing(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    shells: Vec<ClothingShell>,
) {
    for shell in shells {
        let specification = shell.specification;
        let indices = shell
            .faces
            .iter()
            .flat_map(|face| face.iter().copied())
            .collect::<Vec<_>>();
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, shell.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, shell.normals)
        .with_inserted_indices(Indices::U32(indices));
        let [red, green, blue, alpha] = specification.base_color;
        commands.spawn((
            CharacterMesh,
            Name::new(specification.name.clone()),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(red, green, blue, alpha),
                metallic: specification.metallic,
                perceptual_roughness: specification.roughness,
                ..default()
            })),
        ));
    }
}

pub(super) fn spawn_armor(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    armor: &GeneratedArmor,
    name: String,
    material: adventuresim_character_creator::item_catalog_schema::EquipmentMaterial,
) {
    let (color, metallic, roughness) = adventuresim_character_creator::equipment_pbr(material);
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, armor.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, armor.normals.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, armor.texcoords.clone())
    .with_inserted_indices(Indices::U32(armor.indices.clone()));
    commands.spawn((
        CharacterMesh,
        Name::new(name),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(color[0], color[1], color[2], color[3]),
            metallic,
            perceptual_roughness: roughness,
            ..default()
        })),
    ));
}

pub(super) fn spawn_body(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    mesh: Mesh,
) {
    commands.spawn((
        CharacterMesh,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            // Skin is a rough dielectric with a small amount of diffuse
            // transmission. This keeps thin features such as the nose, ears,
            // and fingers warm instead of crushing them to black.
            base_color: Color::srgb(0.64, 0.39, 0.30),
            metallic: 0.0,
            perceptual_roughness: 0.52,
            reflectance: 0.46,
            specular_tint: Color::srgb(1.0, 0.93, 0.89),
            // A small back-diffuse lobe is Bevy's inexpensive approximation
            // of the short scattering distance seen in skin. Kept subtle so
            // the body remains opaque and shadowed rather than wax-like.
            diffuse_transmission: 0.045,
            ..default()
        })),
    ));
}
