//! Body-triangle skin, UV and morph transfer for the frozen garment cut plan.
use super::*;
mod proportions;
use adventuresim_armor_model::ArmorMorph;
use adventuresim_character_creator::{
    armor_frames::Wearer,
    surface_cut::interpolate,
    underlayer::{UnderlayerDesign, UnderlayerEnvelope, UnderlayerPattern},
};

pub(super) fn fit_instance_envelope(
    model: &BodyModel,
    generated: &GeneratedCharacter,
) -> UnderlayerEnvelope {
    let character = &model.mhr.character;
    UnderlayerEnvelope::for_instance(&Wearer {
        detail: model.armor_detail,
        faces: &character.mesh.faces,
        positions: &generated.positions,
        normals: &generated.normals,
        joints: &generated.global_joint_states,
        joint_indices: &character.skin_weights.index,
        joint_weights: &character.skin_weights.weight,
        joint_names: &character.skeleton.names,
    })
}

pub(super) fn fit_envelope(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    morphs: &[ForearmMorphSample],
) -> UnderlayerEnvelope {
    let character = &model.mhr.character;
    let wearer = |positions, normals, joints| Wearer {
        detail: model.armor_detail,
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
    let mut envelope = UnderlayerEnvelope::new(&body);
    let static_samples = proportions::samples(model, generated);
    for sample in &static_samples {
        envelope.constrain_directions_for(&wearer(
            &sample.positions,
            &sample.normals,
            &sample.global_joint_states,
        ));
    }
    envelope.constrain_for(&body);
    for sample in &static_samples {
        envelope.constrain_translation_for(
            &body,
            &wearer(
                &sample.positions,
                &sample.normals,
                &sample.global_joint_states,
            ),
        );
    }
    for sample in morphs {
        envelope.constrain_for(&wearer(
            &sample.positions,
            &sample.normals,
            &sample.global_joint_states,
        ));
    }
    envelope
}

pub(super) fn fitted(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &UnderlayerDesign,
    placement: &str,
    morphs: &[ForearmMorphSample],
    envelope: &UnderlayerEnvelope,
) -> Result<GeneratedArmor> {
    let topology = if design.kind.is_mail() {
        RenderTopology::ClosedShell
    } else {
        RenderTopology::TwoSidedSheet
    };
    fitted_with_topology(
        model, generated, design, placement, morphs, envelope, topology,
    )
}

#[derive(Clone, Copy)]
enum RenderTopology {
    ClosedShell,
    TwoSidedSheet,
}

impl RenderTopology {
    fn construction_faces(self, hidden: std::ops::Range<usize>) -> Vec<std::ops::Range<usize>> {
        match self {
            Self::ClosedShell => Vec::new(),
            Self::TwoSidedSheet => vec![hidden],
        }
    }
}

fn fitted_with_topology(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &UnderlayerDesign,
    placement: &str,
    morphs: &[ForearmMorphSample],
    envelope: &UnderlayerEnvelope,
    topology: RenderTopology,
) -> Result<GeneratedArmor> {
    let character = &model.mhr.character;
    let wearer = |positions, normals, joints| Wearer {
        detail: model.armor_detail,
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
    let pattern = UnderlayerPattern::new(
        design,
        placement,
        &body,
        &character.mesh.texcoord_faces,
        envelope,
    )?;
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
    let components = fabric_component(design, mesh.positions.len(), mesh.indices.len());
    let exterior_indices = pattern.cut.faces.len() * 3;
    let construction_faces = topology.construction_faces(exterior_indices..mesh.indices.len());
    let armor = GeneratedArmor {
        construction_faces,
        plate_edges: Vec::new(),
        design_hash: adventuresim_armor_model::parametric_design_hash(&serde_json::to_vec(design)?),
        surface_domain: MHR_ANATOMICAL_UV_DOMAIN.into(),
        positions: mesh.positions,
        normals,
        texcoords: attributes.texcoords,
        normal_map: None,
        joint_indices: attributes.joint_indices,
        joint_weights: attributes.joint_weights,
        indices: mesh.indices,
        morphs: targets,
        components,
    };
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
}

fn fabric_component(
    design: &UnderlayerDesign,
    vertex_count: usize,
    index_count: usize,
) -> Vec<adventuresim_armor_model::ArmorComponent> {
    if design.kind.is_mail() {
        return Vec::new();
    }
    vec![adventuresim_armor_model::ArmorComponent {
        role: adventuresim_armor_model::ArmorComponentRole::OuterFabric,
        vertices: 0..vertex_count,
        indices: 0..index_count,
        hinge: None,
        material: Some(design.color.material()),
    }]
}

pub(super) fn fitted_trunk_hose(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &adventuresim_armor_model::TrunkHoseDesign,
    placement: &str,
    morphs: &[ForearmMorphSample],
    envelope: &UnderlayerEnvelope,
) -> Result<GeneratedArmor> {
    use adventuresim_character_creator::underlayer::{SurfaceBox, UnderlayerKind};
    design.validate().map_err(anyhow::Error::new)?;
    let carrier = UnderlayerDesign {
        kind: UnderlayerKind::MailBrayette,
        clearance: design.clearance,
        thickness: design.thickness,
        color: design.primary_color,
        length: design.length,
        sleeve_length: adventuresim_armor_model::Permille(1_000),
        patch_width: adventuresim_armor_model::Millimeters(80),
        cuts: Vec::<SurfaceBox>::new(),
    };
    let mut armor = fitted_with_topology(
        model,
        generated,
        &carrier,
        placement,
        morphs,
        envelope,
        RenderTopology::TwoSidedSheet,
    )?;
    armor.design_hash =
        adventuresim_armor_model::parametric_design_hash(&serde_json::to_vec(design)?);
    apply_trunk_hose_panes(&mut armor, design);
    Ok(armor)
}

fn apply_trunk_hose_panes(
    armor: &mut GeneratedArmor,
    design: &adventuresim_armor_model::TrunkHoseDesign,
) {
    let exterior_end = armor
        .construction_faces
        .first()
        .map_or(armor.indices.len(), |range| range.start);
    let construction = armor.indices.split_off(exterior_end);
    let center = [
        armor.positions.iter().map(|point| point[0]).sum::<f32>() / armor.positions.len() as f32,
        armor.positions.iter().map(|point| point[2]).sum::<f32>() / armor.positions.len() as f32,
    ];
    let repeat = std::f32::consts::TAU / f32::from(design.panel_count);
    let mut primary = Vec::new();
    let mut secondary = Vec::new();
    for face in armor.indices.as_chunks::<3>().0 {
        let centroid = face.iter().fold([0.0; 2], |mut centroid, index| {
            let point = armor.positions[*index as usize];
            centroid[0] += point[0] / 3.0;
            centroid[1] += point[2] / 3.0;
            centroid
        });
        let angle = (centroid[0] - center[0])
            .atan2(centroid[1] - center[1])
            .rem_euclid(std::f32::consts::TAU);
        let pane = ((angle / repeat + 0.5).floor() as u8) % design.panel_count;
        if pane.is_multiple_of(2) {
            primary.extend_from_slice(face);
        } else {
            secondary.extend_from_slice(face);
        }
    }
    let split = primary.len();
    primary.append(&mut secondary);
    let exterior_end = primary.len();
    primary.extend(construction);
    armor.indices = primary;
    armor.construction_faces = std::iter::once(exterior_end..armor.indices.len()).collect();
    armor.components = [
        (0..split, design.primary_color),
        (split..exterior_end, design.secondary_color),
    ]
    .into_iter()
    .map(
        |(indices, color)| adventuresim_armor_model::ArmorComponent {
            role: adventuresim_armor_model::ArmorComponentRole::OuterFabric,
            vertices: 0..armor.positions.len(),
            indices,
            hinge: None,
            material: Some(color.material()),
        },
    )
    .collect();
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
