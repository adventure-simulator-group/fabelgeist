//! Static fold and opposing-limb envelopes, evaluated before freezing offsets.
use super::*;
use adventuresim_core::character_proportions::BodyProportion;
use fabelgeist_mhr::math::rotate_vector;

pub(super) fn samples(model: &BodyModel, body: &GeneratedCharacter) -> Vec<ForearmMorphSample> {
    use BodyProportion::{HipWidth, ShoulderWidth, UpperArmLength};
    // The full negative hip editor endpoint already self-intersects the naked
    // body. It cannot define an outward offset envelope; use the generated
    // population's narrow endpoint for this inter-thigh clearance constraint.
    [
        (HipWidth, -HipWidth.generated_limit()),
        (ShoulderWidth, -ShoulderWidth.limit()),
        (UpperArmLength, -UpperArmLength.limit()),
    ]
    .into_iter()
    .map(|(proportion, value)| {
        let character = &model.mhr.character;
        let mut shifts = vec![[0.; 3]; body.global_joint_states.len()];
        for (joint, &parent) in character.skeleton.parents.iter().enumerate() {
            let basis = &body.joint_proportions[joint];
            let delta = value - basis.reference.get(proportion);
            let local = basis.translation_metres[proportion.index()].map(|v| f64::from(v * delta));
            let shift = if parent < 0 {
                local
            } else {
                let state = body.global_joint_states[parent as usize];
                rotate_vector(
                    std::array::from_fn(|i| f64::from(state[i + 3])),
                    local.map(|v| v * f64::from(state[7])),
                )
            };
            shifts[joint] = std::array::from_fn(|i| {
                shift[i] as f32
                    + if parent < 0 {
                        0.
                    } else {
                        shifts[parent as usize][i]
                    }
            });
        }
        let positions = body
            .positions
            .iter()
            .enumerate()
            .map(|(v, p)| {
                let (joints, weights) =
                    adventuresim_character_creator::export::skinning::strongest_four(
                        character.skin_weights.index[v],
                        character.skin_weights.weight[v],
                    );
                std::array::from_fn(|i| {
                    p[i] + joints
                        .iter()
                        .zip(weights)
                        .map(|(j, w)| shifts[*j as usize][i] * w)
                        .sum::<f32>()
                })
            })
            .collect();
        let global_joint_states = body
            .global_joint_states
            .iter()
            .zip(&shifts)
            .map(|(state, shift)| {
                let mut result = *state;
                for i in 0..3 {
                    result[i] += shift[i];
                }
                result
            })
            .collect();
        ForearmMorphSample {
            name: format!("unposed-{}", proportion.mhr_parameter()),
            positions,
            normals: body.normals.clone(),
            global_joint_states,
        }
    })
    .collect()
}
