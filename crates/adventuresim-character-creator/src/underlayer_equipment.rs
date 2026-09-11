//! Body-triangle skin, UV and morph transfer for the frozen garment cut plan.
use super::*;
mod proportions;
use adventuresim_armor_model::ArmorMorph;
use adventuresim_character_creator::{
    armor_frames::Wearer,
    surface_cut::interpolate,
    underlayer::{UnderlayerDesign, UnderlayerPattern},
};

pub(super) fn fitted(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &UnderlayerDesign,
    placement: &str,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let character = &model.mhr.character;
    let wearer = |positions, normals, joints| Wearer {
        faces: &character.mesh.faces,
        positions,
        normals,
        joints,
        joint_indices: &character.skin_weights.index,
        joint_weights: &character.skin_weights.weight,
        joint_names: &character.skeleton.names,
    };
    let body = wearer(
        &generated.positions,
        &generated.normals,
        &generated.global_joint_states,
    );
    let mut pattern =
        UnderlayerPattern::new(design, placement, &body, &character.mesh.texcoord_faces)?;
    let static_samples = proportions::samples(model, generated);
    for sample in &static_samples {
        pattern.constrain_directions_for(&wearer(
            &sample.positions,
            &sample.normals,
            &sample.global_joint_states,
        ));
    }
    pattern.constrain_for(&body);
    for sample in &static_samples {
        pattern.constrain_translation_for(
            &body,
            &wearer(
                &sample.positions,
                &sample.normals,
                &sample.global_joint_states,
            ),
        );
    }
    for sample in morphs {
        pattern.constrain_for(&wearer(
            &sample.positions,
            &sample.normals,
            &sample.global_joint_states,
        ));
    }
    let mesh = pattern.evaluate(design, &body);
    let normals = mesh.normals()?;
    let mut targets = Vec::new();
    for sample in morphs {
        let endpoint = pattern.evaluate(
            design,
            &wearer(
                &sample.positions,
                &sample.normals,
                &sample.global_joint_states,
            ),
        );
        anyhow::ensure!(
            endpoint.indices == mesh.indices,
            "underlayer morph changed cut topology"
        );
        let endpoint_normals = endpoint.normals()?;
        targets.push(ArmorMorph {
            name: sample.name.clone(),
            position_deltas: deltas(&mesh.positions, &endpoint.positions),
            normal_deltas: deltas(&normals, &endpoint_normals),
            direct_positions: endpoint.positions,
        });
    }
    let attributes = SurfaceAttributes::from_pattern(model, &pattern)?;
    let armor = GeneratedArmor {
        plate_edges: Vec::new(),
        design_hash: adventuresim_armor_model::parametric_design_hash(&serde_json::to_vec(design)?),
        surface_domain: MHR_ANATOMICAL_UV_DOMAIN.into(),
        positions: mesh.positions,
        normals,
        texcoords: attributes.texcoords,
        joint_indices: attributes.joint_indices,
        joint_weights: attributes.joint_weights,
        indices: mesh.indices,
        morphs: targets,
        components: Vec::new(),
    };
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
}

struct SurfaceAttributes {
    texcoords: Vec<[f32; 2]>,
    joint_indices: Vec<[u32; 8]>,
    joint_weights: Vec<[f32; 8]>,
}

impl SurfaceAttributes {
    fn from_pattern(model: &BodyModel, pattern: &UnderlayerPattern) -> Result<Self> {
        let character = &model.mhr.character;
        let mut uv = Vec::new();
        let mut joints = Vec::new();
        let mut weights = Vec::new();
        for source in &pattern.cut.points {
            let face = character.mesh.faces[source.triangle];
            let uv_face = character.mesh.texcoord_faces[source.triangle];
            uv.push(interpolate(
                uv_face.map(|i| character.mesh.texcoords[i as usize]),
                source.weights,
            ));
            let mut influences = std::collections::BTreeMap::<u32, f32>::new();
            for (corner, vertex) in face.iter().enumerate() {
                for (joint, weight) in character.skin_weights.index[*vertex as usize]
                    .iter()
                    .zip(character.skin_weights.weight[*vertex as usize])
                {
                    *influences.entry(*joint).or_default() += source.weights[corner] * weight;
                }
            }
            let mut influences = influences.into_iter().collect::<Vec<_>>();
            influences.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            influences.truncate(8);
            let sum = influences.iter().map(|(_, w)| w).sum::<f32>();
            anyhow::ensure!(sum > 0.0, "cut vertex has no skin influences");
            let mut j = [0; 8];
            let mut w = [0.0; 8];
            for (i, (joint, weight)) in influences.into_iter().enumerate() {
                j[i] = joint;
                w[i] = weight / sum;
            }
            joints.push(j);
            weights.push(w);
        }
        let uv = pattern.shell_attributes(&uv);
        let joints = pattern.shell_attributes(&joints);
        let weights = pattern.shell_attributes(&weights);
        Ok(Self {
            texcoords: uv,
            joint_indices: joints,
            joint_weights: weights,
        })
    }
}

fn deltas(base: &[[f32; 3]], sample: &[[f32; 3]]) -> Vec<[f32; 3]> {
    base.iter()
        .zip(sample)
        .map(|(a, b)| std::array::from_fn(|i| b[i] - a[i]))
        .collect()
}
