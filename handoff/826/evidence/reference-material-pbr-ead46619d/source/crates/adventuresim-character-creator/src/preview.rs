//! Spawn generated clothing and armor into the studio preview.

use super::*;
use adventuresim_character_creator::clothing::ClothingShell;

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

pub(super) fn spawn_clothing(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    walk: &WalkPreview,
    model: &BodyModel,
    shells: Vec<ClothingShell>,
) -> Result<()> {
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
        )?;
        commands.spawn((
            CharacterMesh,
            animation_preview::skin(walk).context("animation preview has no rebuilt rig")?,
            Name::new(specification.name.as_ref().to_owned()),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(adventuresim_character_creator::surface_projection::standard_material(specification.appearance))),
        ));
    }
    Ok(())
}

/// The cord lacing a piece's small plates, as the preview shades it.
pub(super) fn lacing_material(cord: &fabelgeist_armor::Lacing) -> StandardMaterial {
    adventuresim_character_creator::surface_projection::standard_material(cord.appearance())
}

/// Spawn each surface of a piece, and its trim band when it has one.
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy asset stores and spawn context are borrowed independently"
)]
pub(super) fn spawn_armor(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
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
        // A component with its own material, such as leather straps, is not
        // shaded as the piece's plate.
        let own = match surface.component.map(|index| &armor.components[index].appearance) {
            Some(fabelgeist_armor::ComponentAppearance::Surface(surface)) => {
                Some(adventuresim_character_creator::surface_projection::standard_material(*surface))
            }
            Some(fabelgeist_armor::ComponentAppearance::Textile(fabric)) => {
                Some(adventuresim_character_creator::textile_surface::material(fabric, images)?)
            }
            Some(fabelgeist_armor::ComponentAppearance::Inherit) | None => None,
        };
        let mut parts = vec![(
            part_name.clone(),
            surface.plate,
            armor.texcoords.as_slice(),
            own.as_ref().unwrap_or(&material),
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
) -> Result<()> {
    animation_preview::skin_mesh(
        &mut mesh,
        &model.mhr.character.skin_weights.index,
        &model.mhr.character.skin_weights.weight,
    )?;
    commands.spawn((
        CharacterMesh,
        animation_preview::BodySkin {
            positions: generated.positions.clone(),
            faces: model.mhr.character.mesh.faces.clone(),
            indices: model.mhr.character.skin_weights.index.clone(),
            weights: model.mhr.character.skin_weights.weight.clone(),
        },
        animation_preview::skin(walk).context("animation preview has no rebuilt rig")?,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(skin_material())),
    ));
    Ok(())
}

/// The body skin visible beneath the separate hair-painted scalp cap.
pub(super) fn skin_material() -> StandardMaterial {
    StandardMaterial {
        // Skin is a rough dielectric with a small amount of diffuse
        // transmission. This keeps thin features such as the nose, ears,
        // and fingers warm instead of crushing them to black.
        reflectance: 0.46,
        specular_tint: Color::srgb(1.0, 0.93, 0.89),
        // A small back-diffuse lobe is Bevy's inexpensive approximation
        // of the short scattering distance seen in skin. Kept subtle so
        // the body remains opaque and shadowed rather than wax-like.
        diffuse_transmission: 0.045,
        ..adventuresim_character_creator::surface_projection::standard_material(adventuresim_character_creator::body_appearance::SKIN_APPEARANCE)
    }
}
