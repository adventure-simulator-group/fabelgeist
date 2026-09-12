//! Direct breastplate fits supply the existing skeletal residual endpoints.
//!
//! Identity channels retain their fixed body correspondence so signed mixtures
//! stay linear. Large skeleton changes instead need the authored carrier fitted
//! to that endpoint's body. Bone translation is removed by the existing caller
//! only after these absolute target positions have replaced the transferred ones.
use adventuresim_armor_model::GeneratedArmor;
use adventuresim_core::skeletal_fit::SkeletalFitMorph;
use anyhow::{Context, Result, ensure};

use super::ForearmMorphSample;

pub(super) fn refit(
    armor: &mut GeneratedArmor,
    samples: &[ForearmMorphSample],
    mut generate: impl FnMut(&ForearmMorphSample) -> Result<GeneratedArmor>,
) -> Result<()> {
    for sample in samples {
        if SkeletalFitMorph::from_name(&sample.name).is_none() {
            continue;
        }
        let fitted = generate(sample)
            .with_context(|| format!("refitting breastplate skeletal endpoint {}", sample.name))?;
        ensure!(
            fitted.positions.len() == armor.positions.len()
                && fitted.normals.len() == armor.normals.len()
                && fitted.indices == armor.indices
                && fitted.components == armor.components
                && fitted.plate_edges == armor.plate_edges,
            "breastplate skeletal endpoint {} changed its reference topology",
            sample.name
        );
        let target = armor
            .morphs
            .iter_mut()
            .find(|target| target.name == sample.name)
            .context("missing generated breastplate skeletal target")?;
        target.position_deltas = armor
            .positions
            .iter()
            .zip(&fitted.positions)
            .map(|(base, endpoint)| std::array::from_fn(|axis| endpoint[axis] - base[axis]))
            .collect();
        target.normal_deltas = armor
            .normals
            .iter()
            .zip(&fitted.normals)
            .map(|(base, endpoint)| std::array::from_fn(|axis| endpoint[axis] - base[axis]))
            .collect();
        target.direct_positions = fitted.positions;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_armor_model::{ArmorComponent, ArmorComponentRole, ArmorMorph};

    fn armor() -> GeneratedArmor {
        GeneratedArmor {
            plate_edges: vec![[0, 1], [1, 2], [2, 0]],
            components: vec![ArmorComponent {
                role: ArmorComponentRole::Plate,
                vertices: 0..3,
                indices: 0..3,
                hinge: None,
                material: None,
            }],
            design_hash: [0; 32],
            surface_domain: "fixture".into(),
            positions: vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.0, 0.1, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            texcoords: vec![[0.0; 2]; 3],
            joint_indices: vec![[0; 8]; 3],
            joint_weights: vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            morphs: vec![],
        }
    }

    fn sample(name: &str, amount: f32) -> ForearmMorphSample {
        ForearmMorphSample {
            name: name.into(),
            positions: armor()
                .positions
                .iter()
                .map(|p| [p[0], p[1] + amount, p[2]])
                .collect(),
            normals: vec![[0.0, 0.0, 1.0]; 3],
            global_joint_states: vec![[0.0, amount * 0.5, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]],
        }
    }

    fn fitted(sample: &ForearmMorphSample) -> GeneratedArmor {
        let mut mesh = armor();
        mesh.positions = sample
            .positions
            .iter()
            .map(|p| [p[0], p[1] * 1.2 + 0.01, p[2]])
            .collect();
        mesh.normals = vec![[0.0, 1.0, 0.0]; 3];
        mesh
    }

    #[test]
    fn direct_skeletal_fits_reconstruct_after_bones_without_changing_identity_targets() {
        let mut base = armor();
        let mut samples = vec![sample("mhr_identity_00", 0.2)];
        samples
            .extend(SkeletalFitMorph::ALL.map(|target| sample(target.name(), target.endpoint())));
        base.morphs = samples
            .iter()
            .map(|sample| ArmorMorph {
                name: sample.name.clone(),
                direct_positions: base.positions.clone(),
                position_deltas: vec![[0.0; 3]; 3],
                normal_deltas: vec![[0.0; 3]; 3],
            })
            .collect();
        let identity = base.morphs[0].clone();
        let mut calls = 0;
        refit(&mut base, &samples, |sample| {
            assert!(SkeletalFitMorph::from_name(&sample.name).is_some());
            calls += 1;
            Ok(fitted(sample))
        })
        .unwrap();
        assert_eq!(calls, SkeletalFitMorph::ALL.len());
        let reference = crate::GeneratedCharacter {
            joint_proportions: vec![],
            positions: armor().positions,
            normals: armor().normals,
            global_joint_states: vec![[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]],
        };
        let corrected = crate::character_morphs::correct_armor_fit(base, &reference, &samples);
        assert_eq!(corrected.morphs[0], identity);
        for (sample, target) in samples[1..].iter().zip(&corrected.morphs[1..]) {
            let direct = fitted(sample);
            assert_eq!(target.direct_positions, direct.positions);
            for index in 0..corrected.positions.len() {
                for axis in 0..3 {
                    let runtime = corrected.positions[index][axis]
                        + target.position_deltas[index][axis]
                        + sample.global_joint_states[0][axis];
                    assert!((runtime - direct.positions[index][axis]).abs() < 1e-6);
                    assert_eq!(
                        corrected.normals[index][axis] + target.normal_deltas[index][axis],
                        direct.normals[index][axis]
                    );
                }
            }
        }
    }

    #[test]
    fn direct_skeletal_refit_rejects_changed_vertex_or_component_connectivity() {
        let samples = [sample(SkeletalFitMorph::ShortSpine.name(), -1.1)];
        for change in 0..3 {
            let mut base = armor();
            base.morphs.push(ArmorMorph {
                name: samples[0].name.clone(),
                direct_positions: base.positions.clone(),
                position_deltas: vec![[0.0; 3]; 3],
                normal_deltas: vec![[0.0; 3]; 3],
            });
            assert!(
                refit(&mut base, &samples, |sample| {
                    let mut result = fitted(sample);
                    match change {
                        0 => result.indices.swap(0, 1),
                        1 => result.positions.pop().map(|_| ()).unwrap(),
                        _ => result.components[0].vertices.end -= 1,
                    }
                    Ok(result)
                })
                .is_err()
            );
        }
    }
}
