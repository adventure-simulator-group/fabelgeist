//! Static fold and opposing-limb envelopes, evaluated before freezing offsets.
use super::*;
use adventuresim_core::character_proportions::BodyProportion;
use fabelgeist_mhr::math::rotate_vector;
mod probe_bounds;

pub(super) fn samples(model: &BodyModel, body: &GeneratedCharacter) -> Vec<ForearmMorphSample> {
    use BodyProportion::{HipWidth, ShoulderWidth, UpperArmLength};
    // Population endpoints can fold an individual wearer's armpits or cross
    // their thighs. Bound the hypothetical probes before freezing offsets;
    // this does not alter the selected body or certify editor endpoints.
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
        let mut positions: Vec<[f32; 3]> = body
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
        let mut fraction = probe_bounds::oriented_area_fraction(
            &body.positions,
            &positions,
            &character.mesh.faces,
        );
        if proportion == HipWidth {
            fraction = fraction.min(sagittal_clearance_fraction(&body.positions, &positions));
        }
        if fraction < 1.0 {
            for (point, original) in positions.iter_mut().zip(&body.positions) {
                *point = std::array::from_fn(|i| original[i] + (point[i] - original[i]) * fraction);
            }
            for shift in &mut shifts {
                *shift = shift.map(|v| v * fraction);
            }
        }
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

/// Keep a quarter of each side's existing medial separation. This limits only
/// the hypothetical clearance probe; it does not modify the selected body or
/// claim support for an editor setting whose naked thighs cross one another.
fn sagittal_clearance_fraction(base: &[[f32; 3]], sample: &[[f32; 3]]) -> f32 {
    const RETAINED_MEDIAL_SEPARATION: f32 = 0.25;
    const SAGITTAL_ROUNDOFF_M: f32 = 1e-6;
    base.iter().zip(sample).fold(1.0, |fraction, (a, b)| {
        let movement = b[0] - a[0];
        if a[0].abs() > SAGITTAL_ROUNDOFF_M && a[0] * movement < 0.0 {
            fraction.min((a[0] * (1.0 - RETAINED_MEDIAL_SEPARATION) / -movement).clamp(0.0, 1.0))
        } else {
            fraction
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hypothetical_narrowing_preserves_the_actual_wearers_medial_gap() {
        let base = [[0.008, 0.8, 0.0], [-0.008, 0.8, 0.0], [0.0, 1.0, 0.0]];
        let crossed = [[-0.002, 0.8, 0.0], [0.002, 0.8, 0.0], [0.0, 1.0, 0.0]];
        let f = sagittal_clearance_fraction(&base, &crossed);
        assert!(f > 0.0 && f < 1.0);
        let left = base[0][0] + (crossed[0][0] - base[0][0]) * f;
        let right = base[1][0] + (crossed[1][0] - base[1][0]) * f;
        assert!(left > 0.0 && right < 0.0 && left - right >= 0.0039);
        assert_eq!(sagittal_clearance_fraction(&base, &base), 1.0);
    }
}
