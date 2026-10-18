//! Admitted runtime clothing requests and body-mesh generation.

use super::{RuntimeBody, catalog_item};
use crate::{
    clothing::{GarmentSpecification, generate_clothing_shells},
    item_catalog_schema::{EquipmentMaterial, EquipmentPlacement, ItemDefinition},
};
use anyhow::{Context, Result};
use fabelgeist_armor::{GeneratedArmor, parametric_design_hash};

pub(super) struct RuntimeClothing<'a> {
    body: &'a RuntimeBody,
    item: &'a ItemDefinition,
    placement: &'a EquipmentPlacement,
    material: EquipmentMaterial,
}

impl<'a> RuntimeClothing<'a> {
    pub(super) fn admit(body: &'a RuntimeBody, item_id: &str, placement_id: &str) -> Result<Self> {
        body.validate()?;
        let item =
            catalog_item(item_id).with_context(|| format!("unknown clothing item {item_id}"))?;
        let equipment = item
            .equipment
            .as_ref()
            .with_context(|| format!("clothing item {item_id} has no equipment definition"))?;
        let material = equipment
            .material
            .with_context(|| format!("clothing item {item_id} has no material"))?;
        let placement = equipment
            .placements
            .iter()
            .find(|placement| placement.id == placement_id)
            .with_context(|| format!("clothing item {item_id} has no placement {placement_id}"))?;
        Ok(Self {
            body,
            item,
            placement,
            material,
        })
    }

    pub(super) fn generate(self) -> Result<GeneratedArmor> {
        let Self {
            body,
            item,
            placement,
            material,
        } = self;
        let item_id = item.id.as_str();
        let placement_id = placement.id.as_str();
        let specification = GarmentSpecification::from_catalog(
            format!("{item_id} · {placement_id}").into(),
            placement,
            material,
        );
        let generated = generate_clothing_shells(
            &[specification],
            &body.positions,
            &body.normals,
            &body.faces,
            &body.joint_indices,
            &body.joint_weights,
            &body.joint_names,
            &body.global_joint_states,
        )
        .map_err(anyhow::Error::from)?;
        let shell = generated
            .shells
            .into_iter()
            .next()
            .context("clothing generator returned no shell")?;
        let base_positions = shell.positions.clone();
        let base_normals = shell.normals.clone();
        let indices = shell
            .faces
            .iter()
            .flat_map(|face| face.iter().copied())
            .collect::<Vec<_>>();
        let design_hash = parametric_design_hash(
            &serde_json::to_vec(&(item_id, placement_id)).expect("clothing IDs serialize"),
        );
        Ok(GeneratedArmor {
            design_hash,
            surface_domain: body.domain.clone(),
            positions: base_positions,
            normals: base_normals,
            texcoords: body.texcoords.clone(),
            joint_indices: body.joint_indices.clone(),
            joint_weights: body.joint_weights.clone(),
            indices,
            // Offset from the body: a cloth layer, not a thickened plate.
            faces: Vec::new(),
            trim: None,
            grids: Vec::new(),
            morphs: Vec::new(),
            components: Vec::new(),
        })
    }
}
