use super::*;
use adventuresim_character_creator::garment::DrapedGarment;
use std::borrow::Cow;

pub(super) fn prepare<'a>(
    model: &BodyModel,
    recipe: &CharacterRecipe,
    generated: &GeneratedCharacter,
    fitted: Option<&'a [DrapedGarment]>,
) -> Result<Cow<'a, [DrapedGarment]>> {
    if let Some(fitted) = fitted {
        return Ok(Cow::Borrowed(fitted));
    }
    if recipe.garments.is_empty() {
        return Ok(Cow::Borrowed(&[]));
    }
    Ok(Cow::Owned(
        adventuresim_character_creator::garment::drape_outfit(
            recipe
                .garments
                .iter()
                .cloned()
                .map(|selection| {
                    let mut input = drape_preview::input(model, generated, selection);
                    input.armor = recipe.armor.clone();
                    input
                })
                .collect(),
            Vec::new(),
            &std::sync::atomic::AtomicBool::new(false),
            |_| {},
        )
        .garments?,
    ))
}

pub(super) fn validate(
    model: &BodyModel,
    recipe: &CharacterRecipe,
    generated: &GeneratedCharacter,
    fitted: &[DrapedGarment],
) -> Result<()> {
    let mut collision_positions = generated
        .positions
        .iter()
        .copied()
        .map(|p| fabelgeist_math::Vec3::new(p[0], p[1], p[2]))
        .collect::<Vec<_>>();
    let mut collision_faces = model.mhr.character.mesh.faces.clone();
    let wearer =
        fabelgeist_bvh::TriangleBvh::new(collision_positions.clone(), collision_faces.clone());
    for garment in fitted {
        garment.validate_body_clearance(&wearer, recipe.armor.as_ref())?;
        let collision =
            fabelgeist_bvh::TriangleBvh::new(collision_positions.clone(), collision_faces.clone());
        garment.validate_contacts(&collision)?;
        let offset = collision_positions.len() as u32;
        collision_positions.extend(
            garment
                .positions
                .iter()
                .copied()
                .map(|p| fabelgeist_math::Vec3::new(p[0], p[1], p[2])),
        );
        collision_faces.extend(garment.faces.iter().map(|face| face.map(|i| i + offset)));
    }
    if let Some(armor) = &recipe.armor {
        for garment in fitted {
            garment.validate_armor(armor)?;
        }
    }
    Ok(())
}
