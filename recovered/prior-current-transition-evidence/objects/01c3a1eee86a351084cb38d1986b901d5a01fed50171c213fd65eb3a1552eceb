use super::*;
pub(super) fn fitted_bracer(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &BracerDesign,
    side: ForearmSide,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let character = &model.mhr.character;
    let armor = adventuresim_character_creator::device_bracer::generate_bracer_on_device(
        adventuresim_character_creator::armor_gpu()?,
        design,
        ForearmSurfaceInput {
            domain: MHR_ANATOMICAL_UV_DOMAIN,
            side,
            positions: &generated.positions,
            normals: &generated.normals,
            faces: &character.mesh.faces,
            texcoords: &character.mesh.texcoords,
            texcoord_faces: &character.mesh.texcoord_faces,
            joint_indices: &character.skin_weights.index,
            joint_weights: &character.skin_weights.weight,
            joint_names: &character.skeleton.names,
            global_joint_states: &generated.global_joint_states,
            morphs,
        },
    )?;
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
}

pub(super) fn fitted_breastplate(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &BreastplateDesign,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let character = &model.mhr.character;
    let armor = adventuresim_character_creator::device_torso::generate_breastplate_on_device(
        adventuresim_character_creator::armor_gpu()?,
        design,
        TorsoSurfaceInput {
            domain: MHR_ANATOMICAL_UV_DOMAIN,
            positions: &generated.positions,
            normals: &generated.normals,
            faces: &character.mesh.faces,
            texcoords: &character.mesh.texcoords,
            texcoord_faces: &character.mesh.texcoord_faces,
            joint_indices: &character.skin_weights.index,
            joint_weights: &character.skin_weights.weight,
            joint_names: &character.skeleton.names,
            global_joint_states: &generated.global_joint_states,
            morphs,
        },
    )?;
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
}
