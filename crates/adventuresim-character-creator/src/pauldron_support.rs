//! Resolve the selected torso recipes for shoulder fitting in preview/export.
use super::*;
use adventuresim_character_creator::{
    armor_frames::Wearer,
    armor_layer::ArmorLayerSurface,
    armor_recipes::{self, ParametricDesign},
};

pub(super) struct PauldronSupport {
    torso: GeneratedArmor,
    relief: adventuresim_armor_model::Millimeters,
    torso_faces: Vec<[u32; 3]>,
    gorget: ParametricDesign,
}

impl PauldronSupport {
    pub(super) fn new(
        model: &BodyModel,
        body: &GeneratedCharacter,
        catalog: &EquipmentCatalog,
        breastplate: &BreastplateDesign,
        morphs: &[ForearmMorphSample],
    ) -> Result<Self> {
        let mut envelope = breastplate.clone();
        let relief = envelope
            .fluting
            .take()
            .map_or(adventuresim_armor_model::Millimeters(0), |pattern| {
                pattern.depth
            });
        let torso = fitted_breastplate(model, body, &envelope, morphs)?;
        let torso_faces = torso.indices.as_chunks::<3>().0.to_vec();
        Ok(Self {
            torso,
            relief,
            torso_faces,
            gorget: catalog
                .design("gorget")
                .context("pauldron support requires a gorget recipe")?,
        })
    }

    pub(super) fn mesh(
        &self,
        design: &ParametricDesign,
        placement: &str,
        wearer: &Wearer<'_>,
        target: Option<&str>,
    ) -> Result<adventuresim_armor_model::PartMesh> {
        let positions = if let Some(name) = target {
            &self
                .torso
                .morphs
                .iter()
                .find(|sample| sample.name == name)
                .context("missing torso support morph")?
                .direct_positions
        } else {
            &self.torso.positions
        };
        let gorget = armor_recipes::fitted_mesh(&self.gorget, "worn", wearer, &[])?;
        let layers = [
            ArmorLayerSurface {
                relief: self.relief,
                positions,
                faces: &self.torso_faces,
            },
            ArmorLayerSurface {
                relief: adventuresim_armor_model::Millimeters(0),
                positions: &gorget.positions,
                faces: gorget.indices.as_chunks::<3>().0,
            },
        ];
        armor_recipes::fitted_mesh(design, placement, wearer, &layers)
    }
}
