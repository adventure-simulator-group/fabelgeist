use super::character_morphs::{
    CharacterMorphs, MorphDelta, armor_targets, rigged_armor, rigged_clothing,
};
use super::*;
mod draped;

pub(super) fn export_character(
    path: &std::path::Path,
    model: &BodyModel,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
    fitted: Option<&[adventuresim_character_creator::garment::DrapedGarment]>,
) -> Result<Vec<String>> {
    let loadout = outfit::loadout(recipe, catalog)?;
    let generated = generate_character(model, recipe)?;
    let morphs = CharacterMorphs::generate(model, recipe, &generated)?;
    let armor = parametric_equipment::selected(model, &generated, &loadout, &morphs.samples)?;
    let lining = outfit::lining(armor.iter().map(|piece| (&piece.piece, &piece.generated)));
    let (fitted, mut warnings) =
        draped::prepare(model, &loadout, &generated, fitted, lining.as_ref());
    warnings.extend(draped::validate(model, &generated, &fitted));
    let body_targets = morphs
        .body
        .iter()
        .map(MorphDelta::rigged)
        .collect::<Vec<_>>();
    let character = &model.mhr.character;
    let clothed = outfit::clothing(model, &loadout, &generated)?;
    let clothing_morphs = morphs.clothing(&clothed.shells)?;
    let clothing_targets = clothing_morphs
        .iter()
        .map(|targets| targets.iter().map(MorphDelta::rigged).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let (plate_faces, plate_targets) = piece_geometry(&armor, |piece| Some(&piece.generated));
    let (lacing_faces, lacing_targets) =
        piece_geometry(&armor, |piece| piece.lacing.as_ref().map(|l| &l.generated));
    let mut shells = clothed
        .shells
        .iter()
        .zip(&clothing_targets)
        .map(|(shell, targets)| rigged_clothing(shell, targets))
        .collect::<Vec<_>>();
    let metals = catalog_metals(&armor, catalog)?;
    shells.extend(catalog_shells(
        &armor,
        CatalogGeometry {
            faces: &plate_faces,
            targets: &plate_targets,
        },
        CatalogGeometry {
            faces: &lacing_faces,
            targets: &lacing_targets,
        },
        &metals,
        catalog,
    )?);
    let draped_morphs = morphs.draped(&generated, &character.mesh.faces, &fitted)?;
    let draped_targets: Vec<_> = draped_morphs
        .iter()
        .map(|targets| targets.iter().map(MorphDelta::rigged).collect::<Vec<_>>())
        .collect();
    let mail_surfaces = mail_surfaces(&loadout, &fitted)?;
    for ((garment, targets), mail) in fitted.iter().zip(&draped_targets).zip(&mail_surfaces) {
        shells.push(garment.rigged(targets, mail.as_ref()));
    }
    export_rigged_glb(
        GlbOutput::Standalone(path),
        &recipe.name,
        recipe.version,
        model.config.lod,
        &RiggedMesh {
            joint_proportions: &generated.joint_proportions,
            morph_targets: &body_targets,
            positions: &generated.positions,
            normals: &generated.normals,
            faces: &clothed.visible_body_faces,
            export_body: true,
            joint_indices: &character.skin_weights.index,
            joint_weights: &character.skin_weights.weight,
            joint_names: &character.skeleton.names,
            joint_parents: &character.skeleton.parents,
            global_joint_states: &generated.global_joint_states,
        },
        &shells,
        &[],
    )?;
    Ok(warnings)
}

type CatalogMetals = Vec<(
    fabelgeist_armor::material::Metal,
    adventuresim_character_creator::export::ShellTextures,
)>;

/// Maps for each distinct catalog steel and trim worn, engraving included,
/// baked once.
fn catalog_metals(
    armor: &[parametric_equipment::SelectedArmor<'_>],
    catalog: &EquipmentCatalog,
) -> Result<CatalogMetals> {
    let mut metals: CatalogMetals = Vec::new();
    for piece in armor {
        let material = catalog.material(&piece.piece.piece.item.id)?;
        let plate = adventuresim_character_creator::armor_metal::metal(
            material,
            piece.piece.decoration.engraving.as_ref(),
        );
        let trim = piece.trim.as_ref().map(|trim| trim.metal.clone());
        for metal in plate.into_iter().chain(trim) {
            if metals.iter().any(|(known, _)| *known == metal) {
                continue;
            }
            let textures = adventuresim_character_creator::export::ShellTextures::armor(&metal)?;
            metals.push((metal, textures));
        }
    }
    Ok(metals)
}

/// Each worn piece's triangles and morph targets for the mesh `mesh` picks
/// of it; empty where it has none.
fn piece_geometry<'a>(
    armor: &'a [parametric_equipment::SelectedArmor<'_>],
    mesh: impl Fn(&'a parametric_equipment::SelectedArmor<'_>) -> Option<&'a GeneratedArmor>,
) -> (Vec<Vec<[u32; 3]>>, Vec<Vec<RiggedMorphTarget<'a>>>) {
    armor
        .iter()
        .map(|piece| {
            mesh(piece).map_or_else(Default::default, |mesh| {
                (
                    mesh.indices.as_chunks::<3>().0.to_vec(),
                    armor_targets(mesh),
                )
            })
        })
        .unzip()
}

/// Each worn piece's triangles and morph targets, for one of its meshes.
struct CatalogGeometry<'a> {
    faces: &'a [Vec<[u32; 3]>],
    targets: &'a [Vec<RiggedMorphTarget<'a>>],
}

fn catalog_shells<'a>(
    armor: &'a [parametric_equipment::SelectedArmor<'_>],
    plates: CatalogGeometry<'a>,
    lacing: CatalogGeometry<'a>,
    metals: &'a CatalogMetals,
    catalog: &EquipmentCatalog,
) -> Result<Vec<RiggedShell<'a>>> {
    let mut shells = Vec::new();
    for (i, piece) in armor.iter().enumerate() {
        let trim_name = piece.trim.as_ref().map_or(&piece.name, |trim| &trim.name);
        let rigged = rigged_armor(
            &piece.name,
            trim_name,
            &piece.generated,
            &plates.faces[i],
            &plates.targets[i],
        );
        let mut parts = rigged.plate;
        let material = catalog.material(&piece.piece.piece.item.id)?;
        let (color, metallic, roughness) = adventuresim_character_creator::equipment_pbr(material);
        let metal = adventuresim_character_creator::armor_metal::metal(
            material,
            piece.piece.decoration.engraving.as_ref(),
        )
        .and_then(|metal| metals.iter().find(|(known, _)| *known == metal))
        .map(|(_, textures)| (piece.generated.texcoords.as_slice(), textures));
        for shell in &mut parts {
            shell.base_color = color;
            shell.metallic = metallic;
            shell.roughness = roughness;
            shell.textures = adventuresim_character_creator::underlayer_material::textures(
                piece.piece.design.recipe(),
            );
            shell.surface = metal;
        }
        crate::character_morphs::component_materials(&piece.generated, &mut parts);
        shells.extend(parts);
        if let Some(trim) = &piece.trim {
            let textures = metals
                .iter()
                .find(|(known, _)| *known == trim.metal)
                .map(|(_, textures)| textures)
                .context("trim metal was not baked")?;
            let [red, green, blue] = trim.metal.color;
            for mut shell in rigged.trim {
                shell.base_color = [red, green, blue, 1.0];
                shell.metallic = 1.0;
                shell.roughness = trim.metal.roughness;
                // The band's own coordinates run along each edge.
                shell.texcoords = Some(&trim.texcoords);
                shell.surface = Some((&trim.texcoords, textures));
                shells.push(shell);
            }
        }
        if let Some(laced) = &piece.lacing {
            let [red, green, blue] = laced.cord.color;
            let rigged = rigged_armor(
                &laced.name,
                &laced.name,
                &laced.generated,
                &lacing.faces[i],
                &lacing.targets[i],
            );
            for mut shell in rigged.plate {
                shell.base_color = [red, green, blue, 1.0];
                shell.metallic = 0.0;
                shell.roughness = laced.cord.roughness;
                shells.push(shell);
            }
        }
    }
    Ok(shells)
}

/// Chainmail appearance comes from the current recipe, not the drape.
fn mail_surfaces(
    loadout: &adventuresim_character_creator::inventory::Loadout<'_>,
    fitted: &[adventuresim_character_creator::garment::DrapedGarment],
) -> Result<Vec<Option<adventuresim_character_creator::garment_material::MailSurface>>> {
    fitted
        .iter()
        .enumerate()
        .map(|(index, garment)| {
            let selection = loadout
                .draped
                .get(index)
                .context("draped garment has no worn inventory article")?
                .selection;
            (garment.fabric == FabricPreset::Chainmail)
                .then(|| {
                    adventuresim_character_creator::garment_material::MailSurface::new(
                        &selection.mail,
                        &garment.texcoords,
                    )
                })
                .transpose()
        })
        .collect()
}
