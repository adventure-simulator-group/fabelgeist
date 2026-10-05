//! Generator inputs for the worn outfit.
use super::*;
use adventuresim_character_creator::{
    garment::{DrapeInput, PlateLining, UnderPlate},
    inventory::{FittedPiece, InventoryItemId, Loadout},
};
use std::sync::Arc;

/// Room between worn plate and the outermost cloth pressed under it, metres.
const LINING_CLEARANCE_M: f32 = 0.001;

/// The worn outfit, or why it cannot be worn together.
pub(super) fn loadout<'a>(
    recipe: &'a CharacterRecipe,
    catalog: &'a EquipmentCatalog,
) -> Result<Loadout<'a>> {
    recipe
        .inventory
        .loadout(catalog)
        .map_err(|error| anyhow::anyhow!(recipe.inventory.explain(catalog, &error)))
}

/// Worn catalog clothing as shells offset from the body, and the body faces left visible.
pub(super) fn clothing(
    model: &BodyModel,
    loadout: &Loadout<'_>,
    generated: &GeneratedCharacter,
) -> Result<adventuresim_character_creator::clothing::ClothedMesh> {
    let character = &model.mhr.character;
    generate_clothing_shells(
        &clothing_specifications(loadout)?,
        &generated.positions,
        &generated.normals,
        &character.mesh.faces,
        &character.skin_weights.index,
        &character.skin_weights.weight,
        &character.skeleton.names,
        &generated.global_joint_states,
    )
    .map_err(anyhow::Error::from)
}

fn clothing_specifications(loadout: &Loadout<'_>) -> Result<Vec<GarmentSpecification>> {
    loadout
        .clothing
        .iter()
        .map(|piece| {
            let material = piece
                .item
                .equipment
                .as_ref()
                .and_then(|equipment| equipment.material)
                .with_context(|| format!("item {} has no procedural material", piece.item.id))?;
            Ok(GarmentSpecification::from_catalog(
                format!("{} · {}", piece.item.display_name, piece.placement.id),
                piece.placement,
                material,
            ))
        })
        .collect()
}

/// Drape inputs for worn cloth, innermost first. Cloth worn under plate
/// lies under `lining`, each garment far enough in for those pressed over it.
pub(super) fn drape_inputs(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    loadout: &Loadout<'_>,
    lining: Option<&Arc<PlateLining>>,
) -> Vec<(InventoryItemId, DrapeInput)> {
    let mut over = LINING_CLEARANCE_M;
    let mut inputs = loadout
        .draped
        .iter()
        .rev()
        .map(|piece| {
            let mut input = drape_preview::input(model, generated, piece.selection.clone());
            input.settled = piece.drape.cloned().map(Arc::new);
            if let Some(lining) = lining.filter(|_| UnderPlate::covers(piece.selection.channel())) {
                let half = piece.selection.fabric.fabric().particle_radius();
                input.under_plate = Some(UnderPlate {
                    lining: lining.clone(),
                    standoff: over + half,
                });
                over += 2.0 * half;
            }
            (piece.id, input)
        })
        .collect::<Vec<_>>();
    inputs.reverse();
    inputs
}

/// The lining of the worn rigid plate among `pieces`.
pub(super) fn lining<'a>(
    pieces: impl IntoIterator<Item = (&'a FittedPiece<'a>, &'a GeneratedArmor)>,
) -> Option<Arc<PlateLining>> {
    PlateLining::new(
        pieces
            .into_iter()
            .filter(|(piece, _)| parametric_equipment::is_rigid(piece))
            .map(|(_, armor)| armor),
    )
    .map(Arc::new)
}
