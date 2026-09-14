use super::*;
use adventuresim_character_creator::export::ShellTextures;

pub(super) struct PlateExport {
    parts: Vec<armor_preview::RiggedArmorPart>,
    textures: ShellTextures,
    morphs: Vec<Vec<MorphDelta>>,
    metal: fabelgeist_armor::material::Metal,
}
impl PlateExport {
    pub(super) fn new(
        armor: &fabelgeist_armor::Armor,
        model: &BodyModel,
        states: &[[f32; 8]],
        body_targets: &[RiggedMorphTarget<'_>],
    ) -> Result<Self> {
        let textures = ShellTextures::armor(&armor.metal)?;
        let parts =
            armor_preview::rigged_parts(armor, model, states).map_err(anyhow::Error::msg)?;
        let morphs = parts
            .iter()
            .map(|part| {
                body_targets
                    .iter()
                    .map(|target| {
                        MorphDelta::fixed_geometry(target.name, part.part.mesh.positions.len())
                    })
                    .collect()
            })
            .collect();
        Ok(Self {
            parts,
            textures,
            morphs,
            metal: armor.metal.clone(),
        })
    }
    pub(super) fn targets(&self) -> Vec<Vec<RiggedMorphTarget<'_>>> {
        self.morphs
            .iter()
            .map(|targets| targets.iter().map(MorphDelta::rigged).collect())
            .collect()
    }
    pub(super) fn shells<'a>(
        &'a self,
        targets: &'a [Vec<RiggedMorphTarget<'a>>],
    ) -> Vec<RiggedShell<'a>> {
        self.parts
            .iter()
            .zip(targets)
            .map(|(part, targets)| RiggedShell {
                surface: Some((&part.part.mesh.uvs, &self.textures)),
                textures: None,
                texcoords: None,
                hinge: None,
                name: &part.part.name,
                positions: &part.part.mesh.positions,
                normals: &part.part.mesh.normals,
                faces: &part.part.mesh.faces,
                joint_indices: Some(&part.indices),
                joint_weights: Some(&part.weights),
                morph_targets: targets,
                base_color: [
                    self.metal.color[0],
                    self.metal.color[1],
                    self.metal.color[2],
                    1.0,
                ],
                metallic: 1.0,
                roughness: self.metal.roughness,
            })
            .collect()
    }
}
