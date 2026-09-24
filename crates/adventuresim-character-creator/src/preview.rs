//! Spawn generated clothing and armor into the studio preview.

use super::*;
use adventuresim_character_creator::clothing::ClothingShell;

pub(super) fn spawn_clothing(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    walk: &WalkPreview,
    model: &BodyModel,
    shells: Vec<ClothingShell>,
) {
    for shell in shells {
        let specification = shell.specification;
        let indices = shell
            .faces
            .iter()
            .flat_map(|face| face.iter().copied())
            .collect::<Vec<_>>();
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, shell.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, shell.normals)
        .with_inserted_indices(Indices::U32(indices));
        animation_preview::skin_mesh(
            &mut mesh,
            &model.mhr.character.skin_weights.index,
            &model.mhr.character.skin_weights.weight,
        );
        let [red, green, blue, alpha] = specification.base_color;
        commands.spawn((
            CharacterMesh,
            animation_preview::skin(walk).expect("animation rig was just rebuilt"),
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

/// How a piece is shaded: its plate, and the band along its edges.
pub(super) struct ArmorShading<'a> {
    pub plate: StandardMaterial,
    pub trim: Option<TrimPreview<'a>>,
}

/// The band along a piece's edges, as the preview shades it.
pub(super) struct TrimPreview<'a> {
    pub texcoords: &'a [[f32; 2]],
    pub material: StandardMaterial,
}

/// The cord lacing a piece's small plates, as the preview shades it.
pub(super) fn lacing_material(cord: &fabelgeist_armor::Lacing) -> StandardMaterial {
    let [red, green, blue] = cord.color;
    StandardMaterial {
        base_color: Color::srgb(red, green, blue),
        metallic: 0.0,
        perceptual_roughness: cord.roughness,
        ..default()
    }
}

/// Spawn each surface of a piece, and its trim band when it has one.
pub(super) fn spawn_armor(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    armor: &GeneratedArmor,
    name: String,
    shading: ArmorShading<'_>,
    marker: impl Bundle + Clone,
) -> Result<()> {
    let ArmorShading {
        plate: material,
        trim,
    } = shading;
    for surface in armor.surfaces() {
        let part_name = match surface.component {
            Some(index) => format!("{name}.{}", armor.components[index].role.name()),
            None => name.clone(),
        };
        let mut parts = vec![(
            part_name.clone(),
            surface.plate,
            armor.texcoords.as_slice(),
            &material,
        )];
        if let Some(trim) = trim.as_ref().filter(|_| !surface.trim.is_empty()) {
            parts.push((
                format!("{part_name}.trim"),
                surface.trim,
                trim.texcoords,
                &trim.material,
            ));
        }
        for (part_name, indices, texcoords, material) in parts {
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, armor.positions.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, armor.normals.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, texcoords.to_vec())
            .with_inserted_indices(Indices::U32(armor.indices[indices].to_vec()));
            if material.normal_map_texture.is_some() {
                mesh.generate_tangents()
                    .context("generating tangents for textured armor preview")?;
            }
            commands.spawn((
                marker.clone(),
                Name::new(part_name),
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(material.clone())),
            ));
        }
    }
    Ok(())
}

pub(super) fn spawn_body(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    walk: &WalkPreview,
    model: &BodyModel,
    generated: &GeneratedCharacter,
    mut mesh: Mesh,
) {
    animation_preview::skin_mesh(
        &mut mesh,
        &model.mhr.character.skin_weights.index,
        &model.mhr.character.skin_weights.weight,
    );
    commands.spawn((
        CharacterMesh,
        animation_preview::BodySkin {
            positions: generated.positions.clone(),
            faces: model.mhr.character.mesh.faces.clone(),
            indices: model.mhr.character.skin_weights.index.clone(),
            weights: model.mhr.character.skin_weights.weight.clone(),
        },
        animation_preview::skin(walk).expect("animation rig was just rebuilt"),
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
