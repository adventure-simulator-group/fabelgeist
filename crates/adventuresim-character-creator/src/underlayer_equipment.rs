//! The studio's underlayers: cut from the wearer on the device and corrected
//! for skeletal fit.
use super::*;
mod proportions;
use adventuresim_character_creator::{
    armor_frames::Wearer,
    device_underlayer::{BodyShape, SurfaceDomain},
    underlayer::UnderlayerDesign,
    underlayer_armor::{self, UnderlayerBody},
};
use fabelgeist_armor::TrunkHoseDesign;

/// Cut an underlayer from the wearer and fit it to every morph sample.
pub(super) fn fitted(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &UnderlayerDesign,
    placement: &str,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let armor = with_body(model, generated, morphs, |body, shapes| {
        underlayer_armor::fit_underlayer(
            adventuresim_character_creator::armor_gpu()?,
            design,
            placement,
            body,
            shapes,
        )
    })?;
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
}

/// Fit trunk hose over the hips in its two alternating fabrics.
pub(super) fn fitted_trunk_hose(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &TrunkHoseDesign,
    placement: &str,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let armor = with_body(model, generated, morphs, |body, shapes| {
        underlayer_armor::fit_trunk_hose(
            adventuresim_character_creator::armor_gpu()?,
            design,
            placement,
            body,
            shapes,
        )
    })?;
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
}

fn with_body(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    morphs: &[ForearmMorphSample],
    fit: impl FnOnce(&UnderlayerBody<'_>, &[(&str, BodyShape<'_>)]) -> Result<GeneratedArmor>,
) -> Result<GeneratedArmor> {
    let character = &model.mhr.character;
    let wearer = Wearer {
        faces: &character.mesh.faces,
        positions: &generated.positions,
        normals: &generated.normals,
        joints: &generated.global_joint_states,
        joint_indices: &character.skin_weights.index,
        joint_weights: &character.skin_weights.weight,
        joint_names: &character.skeleton.names,
    };
    let static_samples = proportions::samples(model, generated);
    let proportions = static_samples.iter().map(shape).collect::<Vec<_>>();
    let shapes = morphs
        .iter()
        .map(|sample| (sample.name.as_str(), shape(sample)))
        .collect::<Vec<_>>();
    fit(
        &UnderlayerBody {
            wearer: &wearer,
            domain: SurfaceDomain {
                uv_faces: &character.mesh.texcoord_faces,
                texcoords: &character.mesh.texcoords,
            },
            surface_domain: MHR_ANATOMICAL_UV_DOMAIN,
            proportions: &proportions,
        },
        &shapes,
    )
}

fn shape(sample: &ForearmMorphSample) -> BodyShape<'_> {
    BodyShape {
        positions: &sample.positions,
        normals: &sample.normals,
    }
}
