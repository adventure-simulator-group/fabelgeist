//! Underlayers fitted on the device, assembled into armor with their own
//! surface attributes and morph targets.
use super::*;
mod proportions;
use crate::parametric_equipment::deltas;
use adventuresim_armor_model::ArmorMorph;
use adventuresim_character_creator::{
    armor_frames::Wearer,
    device_underlayer::{self, BodyShape, SurfaceDomain},
    underlayer::UnderlayerDesign,
};

/// Cut an underlayer from the wearer and fit it to every morph sample.
pub(super) fn fitted(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &UnderlayerDesign,
    placement: &str,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let character = &model.mhr.character;
    let body = Wearer {
        faces: &character.mesh.faces,
        positions: &generated.positions,
        normals: &generated.normals,
        joints: &generated.global_joint_states,
        joint_indices: &character.skin_weights.index,
        joint_weights: &character.skin_weights.weight,
        joint_names: &character.skeleton.names,
    };
    let static_samples = proportions::samples(model, generated);
    let fitted = device_underlayer::fit(
        adventuresim_character_creator::armor_gpu()?,
        design,
        placement,
        &body,
        SurfaceDomain {
            uv_faces: &character.mesh.texcoord_faces,
            texcoords: &character.mesh.texcoords,
        },
        &shapes(&static_samples),
        &shapes(morphs),
    )?;
    let base = fitted.base;
    let targets = morphs
        .iter()
        .zip(fitted.endpoints)
        .map(|(sample, endpoint)| ArmorMorph {
            name: sample.name.clone(),
            position_deltas: deltas(&base.positions, &endpoint.positions),
            normal_deltas: deltas(&base.normals, &endpoint.normals),
            direct_positions: endpoint.positions,
        })
        .collect();
    let armor = GeneratedArmor {
        design_hash: adventuresim_armor_model::parametric_design_hash(&serde_json::to_vec(design)?),
        surface_domain: MHR_ANATOMICAL_UV_DOMAIN.into(),
        positions: base.positions,
        normals: base.normals,
        texcoords: fitted.texcoords,
        joint_indices: fitted.joint_indices,
        joint_weights: fitted.joint_weights,
        indices: fitted.indices,
        morphs: targets,
        components: Vec::new(),
    };
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
}

fn shapes(samples: &[ForearmMorphSample]) -> Vec<BodyShape<'_>> {
    samples
        .iter()
        .map(|sample| BodyShape {
            positions: &sample.positions,
            normals: &sample.normals,
        })
        .collect()
}
