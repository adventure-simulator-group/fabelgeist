//! The same fitted closure geometry is used by preview and both GLB exporters.
use super::*;
use adventuresim_armor_model::{ArmorComponentRole, ArmorMorph, PartMesh};
use adventuresim_character_creator::armor_frames::Wearer;

pub(super) fn attach(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    samples: &[ForearmMorphSample],
    catalog: &EquipmentCatalog,
    item_id: &str,
    placement: &str,
    mut armor: GeneratedArmor,
) -> Result<GeneratedArmor> {
    let Some(recipe) = catalog.2.get(item_id) else {
        return Ok(armor);
    };
    let fitter = ClosureFitter {
        model,
        catalog,
        armor: &armor,
        recipe,
        placement,
    };
    let mesh = fitter.evaluate(
        &armor.positions,
        &generated.positions,
        &generated.normals,
        &generated.global_joint_states,
    )?;
    let normals = mesh.normals().context("reference fastening normals")?;
    let nearest = mesh
        .positions
        .iter()
        .map(|p| {
            armor
                .positions
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    squared_distance(*p, **a).total_cmp(&squared_distance(*p, **b))
                })
                .map(|(i, _)| i)
                .expect("fitted armor has vertices")
        })
        .collect::<Vec<_>>();
    let mut morphs = Vec::new();
    for (target, sample) in armor.morphs.iter().zip(samples) {
        anyhow::ensure!(
            target.name == sample.name,
            "fastener morph ordering differs from plate"
        );
        let endpoint = fitter
            .evaluate(
                &target.direct_positions,
                &sample.positions,
                &sample.normals,
                &sample.global_joint_states,
            )
            .with_context(|| {
                format!("fit {item_id} {placement} fastening target {}", target.name)
            })?;
        anyhow::ensure!(
            endpoint.indices == mesh.indices && endpoint.positions.len() == mesh.positions.len(),
            "fastener fitting changed morph correspondence"
        );
        morphs.push(ArmorMorph {
            name: target.name.clone(),
            position_deltas: deltas(&mesh.positions, &endpoint.positions),
            normal_deltas: deltas(
                &normals,
                &endpoint.normals().with_context(|| {
                    format!("fastening normals at morph {} ({placement})", target.name)
                })?,
            ),
            direct_positions: endpoint.positions,
        });
    }
    let mut hardware = GeneratedArmor {
        design_hash: adventuresim_armor_model::parametric_design_hash(&serde_json::to_vec(recipe)?),
        surface_domain: armor.surface_domain.clone(),
        plate_edges: Vec::new(),
        positions: mesh.positions,
        normals,
        indices: mesh.indices,
        components: mesh.components,
        texcoords: nearest.iter().map(|i| armor.texcoords[*i]).collect(),
        joint_indices: nearest.iter().map(|i| armor.joint_indices[*i]).collect(),
        joint_weights: nearest.iter().map(|i| armor.joint_weights[*i]).collect(),
        morphs,
    };
    if matches!(
        recipe,
        adventuresim_character_creator::fasteners::catalog::FastenerRecipe::TassetSuspension(_)
    ) {
        crate::fastener_skin::attach_suspenders(&armor, &mut hardware)?;
    } else {
        crate::fastener_skin::attach(&armor, &mut hardware);
    }
    let hardware = character_morphs::correct_armor_fit(hardware, generated, samples);
    append(&mut armor, hardware)?;
    Ok(armor)
}

fn append(armor: &mut GeneratedArmor, hardware: GeneratedArmor) -> Result<()> {
    anyhow::ensure!(
        armor.morphs.len() == hardware.morphs.len(),
        "missing fastener morph samples"
    );
    if armor.components.is_empty() {
        armor
            .components
            .push(adventuresim_armor_model::ArmorComponent {
                role: ArmorComponentRole::Plate,
                vertices: 0..armor.positions.len(),
                indices: 0..armor.indices.len(),
                hinge: None,
                material: None,
            });
    }
    let vertices = armor.positions.len();
    let indices = armor.indices.len();
    armor
        .components
        .extend(hardware.components.into_iter().map(|mut c| {
            c.vertices = c.vertices.start + vertices..c.vertices.end + vertices;
            c.indices = c.indices.start + indices..c.indices.end + indices;
            c
        }));
    armor.positions.extend(hardware.positions);
    armor.normals.extend(hardware.normals);
    armor.texcoords.extend(hardware.texcoords);
    armor.joint_indices.extend(hardware.joint_indices);
    armor.joint_weights.extend(hardware.joint_weights);
    armor
        .indices
        .extend(hardware.indices.into_iter().map(|i| i + vertices as u32));
    for (a, b) in armor.morphs.iter_mut().zip(hardware.morphs) {
        anyhow::ensure!(a.name == b.name, "fastener morph name mismatch");
        a.position_deltas.extend(b.position_deltas);
        a.normal_deltas.extend(b.normal_deltas);
        a.direct_positions.extend(b.direct_positions);
    }
    armor.design_hash = adventuresim_armor_model::parametric_design_hash(
        &[armor.design_hash, hardware.design_hash].concat(),
    );
    Ok(())
}

fn deltas(a: &[[f32; 3]], b: &[[f32; 3]]) -> Vec<[f32; 3]> {
    a.iter()
        .zip(b)
        .map(|(a, b)| std::array::from_fn(|i| b[i] - a[i]))
        .collect()
}

fn squared_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum()
}

struct ClosureFitter<'a> {
    model: &'a BodyModel,
    catalog: &'a EquipmentCatalog,
    armor: &'a GeneratedArmor,
    recipe: &'a adventuresim_character_creator::fasteners::catalog::FastenerRecipe,
    placement: &'a str,
}

impl ClosureFitter<'_> {
    fn evaluate(
        &self,
        positions: &[[f32; 3]],
        body: &[[f32; 3]],
        normals: &[[f32; 3]],
        joints: &[[f32; 8]],
    ) -> Result<PartMesh> {
        let wearer = Wearer {
            positions: body,
            normals,
            faces: &self.model.mhr.character.mesh.faces,
            joint_indices: &self.model.mhr.character.skin_weights.index,
            joint_weights: &self.model.mhr.character.skin_weights.weight,
            joint_names: &self.model.mhr.character.skeleton.names,
            joints,
        };
        let mut plate = PartMesh::new();
        plate.positions = positions.to_vec();
        plate.indices = self.armor.indices.clone();
        plate.components = self.armor.components.clone();
        let support =
            self.recipe
                .fitted_support(&self.catalog.2, &wearer, self.placement, &|id| {
                    adventuresim_character_creator::armor_recipes::fitted_mesh(
                        &self
                            .catalog
                            .design(id, self.placement)
                            .context("missing closure support recipe")?,
                        self.placement,
                        &wearer,
                        &[],
                    )
                })?;
        self.recipe
            .generate(&plate, &wearer, self.placement, support.as_ref())
    }
}
