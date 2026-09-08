use super::*;

pub(super) fn generate_character(
    model: &BodyModel,
    recipe: &CharacterRecipe,
) -> Result<GeneratedCharacter> {
    recipe.validate().map_err(anyhow::Error::msg)?;
    let device = Device::default();
    let identity = Tensor::from_data(
        TensorData::new(recipe.identity.clone(), [1, IDENTITY_MORPH_COUNT]),
        &device,
    );
    let expression = Tensor::from_data(
        TensorData::new(
            recipe.expression.clone(),
            [1, NUM_FACE_EXPRESSION_BLEND_SHAPES],
        ),
        &device,
    );
    let parameters = adventuresim_character_creator::proportions::model_parameters(
        &model.mhr,
        recipe.proportions,
    )?;
    let pose = Tensor::from_data(
        TensorData::new(parameters, [1, model.mhr.num_model_parameters()]),
        &device,
    );
    let output = model.mhr.forward(identity, pose, Some(expression))?;
    let vertex_values = output
        .vertices
        .into_data()
        .into_vec::<f32>()
        .map_err(|error| anyhow::anyhow!("GPU vertex readback failed: {error:?}"))?;
    let (vertex_chunks, vertex_remainder) = vertex_values.as_chunks::<3>();
    if !vertex_remainder.is_empty() {
        return Err(anyhow::anyhow!(
            "GPU vertex readback did not contain complete three-axis positions"
        ));
    }
    let positions: Vec<[f32; 3]> = vertex_chunks
        .iter()
        .map(|v| [v[0] / 100.0, v[1] / 100.0, v[2] / 100.0])
        .collect();

    let normal_values = output
        .normals
        .into_data()
        .into_vec::<f32>()
        .map_err(|error| anyhow::anyhow!("GPU normal readback failed: {error:?}"))?;
    let (normal_chunks, normal_remainder) = normal_values.as_chunks::<3>();
    if !normal_remainder.is_empty() {
        return Err(anyhow::anyhow!(
            "GPU normal readback did not contain complete three-axis normals"
        ));
    }
    let normals: Vec<[f32; 3]> = normal_chunks.iter().map(|n| [n[0], n[1], n[2]]).collect();

    let skeleton_values = output
        .skeleton_state
        .into_data()
        .into_vec::<f32>()
        .map_err(|error| anyhow::anyhow!("GPU skeleton readback failed: {error:?}"))?;
    let (skeleton_chunks, skeleton_remainder) = skeleton_values.as_chunks::<8>();
    if !skeleton_remainder.is_empty() {
        return Err(anyhow::anyhow!(
            "GPU skeleton readback did not contain complete joint states"
        ));
    }
    let global_joint_states = skeleton_chunks
        .iter()
        .map(|joint| {
            let mut state = *joint;
            state[0] /= 100.0;
            state[1] /= 100.0;
            state[2] /= 100.0;
            state
        })
        .collect();
    Ok(GeneratedCharacter {
        joint_proportions: adventuresim_character_creator::proportions::joint_bases(
            &model.mhr,
            recipe.proportions,
        )?,
        positions,
        normals,
        global_joint_states,
    })
}
