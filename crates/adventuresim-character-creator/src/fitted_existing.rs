use super::*;
pub(super) fn fitted_bracer(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &BracerDesign,
    side: ForearmSide,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let character = &model.mhr.character;
    let surface = build_forearm_surface(ForearmSurfaceInput {
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
    })
    .map_err(anyhow::Error::msg)?;
    let armor = generate_bracer(design, &surface).map_err(anyhow::Error::new)?;
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
    let surface = build_front_torso_surface(TorsoSurfaceInput {
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
    })
    .map_err(anyhow::Error::msg)?;
    let mut armor = generate_breastplate(design, &surface).map_err(anyhow::Error::new)?;
    breastplate_skeletal_fit::refit(&mut armor, morphs, |sample| {
        let surface = build_front_torso_surface(TorsoSurfaceInput {
            domain: MHR_ANATOMICAL_UV_DOMAIN,
            positions: &sample.positions,
            normals: &sample.normals,
            faces: &character.mesh.faces,
            texcoords: &character.mesh.texcoords,
            texcoord_faces: &character.mesh.texcoord_faces,
            joint_indices: &character.skin_weights.index,
            joint_weights: &character.skin_weights.weight,
            joint_names: &character.skeleton.names,
            global_joint_states: &sample.global_joint_states,
            morphs: &[],
        })
        .map_err(anyhow::Error::msg)?;
        Ok(generate_breastplate(design, &surface)?)
    })?;
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
}
