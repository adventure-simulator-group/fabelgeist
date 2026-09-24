use super::*;
use adventuresim_character_creator::{garment::DrapedGarment, inventory::Loadout};
use std::borrow::Cow;

/// The draped garments to export, and why any worn garment is missing or flawed.
/// Garments that could not be draped are left out rather than failing the export.
pub(super) fn prepare<'a>(
    model: &BodyModel,
    loadout: &Loadout<'_>,
    generated: &GeneratedCharacter,
    fitted: Option<&'a [DrapedGarment]>,
) -> (Cow<'a, [DrapedGarment]>, Vec<String>) {
    let (garments, mut warnings) = match fitted {
        Some(fitted) => (Cow::Borrowed(fitted), Vec::new()),
        None if loadout.draped.is_empty() => (Cow::Borrowed(&[][..]), Vec::new()),
        None => {
            let outcome = adventuresim_character_creator::garment::drape_outfit(
                outfit::drape_inputs(model, generated, loadout)
                    .into_iter()
                    .map(|(_, input)| input)
                    .collect(),
                Vec::new(),
                &std::sync::atomic::AtomicBool::new(false),
                |_| {},
            );
            let warnings = outcome.problems().into_iter().collect();
            (Cow::Owned(outcome.garments), warnings)
        }
    };
    let missing = loadout.draped.len().saturating_sub(garments.len());
    if missing > 0 {
        warnings.push(format!(
            "{missing} of {} draped garments could not be draped and are left out",
            loadout.draped.len()
        ));
    }
    (garments, warnings)
}

pub(super) fn validate(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    fitted: &[DrapedGarment],
) -> Vec<String> {
    let mut warnings = Vec::new();
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
        if let Err(error) = garment.validate_body_clearance(&wearer) {
            warnings.push(format!("{}: {error:#}", garment.name));
        }
        let collision =
            fabelgeist_bvh::TriangleBvh::new(collision_positions.clone(), collision_faces.clone());
        warnings.extend(
            garment
                .contact_issues(&collision)
                .into_iter()
                .map(|issue| format!("{}: {issue}", garment.name)),
        );
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
    warnings
}
