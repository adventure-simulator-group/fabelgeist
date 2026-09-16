//! Authored equipment boundary, including the no-placeholder armor invariant.

use crate::{
    armor_recipes,
    item_catalog_schema::{
        EquipmentMaterial, EquipmentPlacement, ItemCatalogDocument, ItemDefinition, ItemKind,
    },
    item_design::{CatalogDesigns, ItemDesign},
};
use anyhow::{Context, Result};

/// The item definitions a character can wear, with each parametric item's default design.
#[derive(Clone, Debug)]
pub struct ItemCatalog {
    pub items: Vec<ItemDefinition>,
    pub designs: CatalogDesigns,
}

impl ItemCatalog {
    pub fn new(items: Vec<ItemDefinition>, designs: CatalogDesigns) -> Result<Self> {
        validate_armor_recipes(&items)?;
        Ok(Self { items, designs })
    }

    /// Every `.yaml` item document in `directory`, in file name order.
    pub fn load(directory: &std::path::Path, designs: CatalogDesigns) -> Result<Self> {
        let mut files = std::fs::read_dir(directory)
            .with_context(|| format!("reading item catalog directory {}", directory.display()))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        files.retain(|path| {
            path.extension()
                .is_some_and(|extension| extension == "yaml")
        });
        files.sort();
        let mut items = Vec::new();
        for path in files {
            let document: ItemCatalogDocument = serde_json::from_slice(&std::fs::read(&path)?)
                .with_context(|| format!("parsing item catalog {}", path.display()))?;
            items.extend(document.items);
        }
        anyhow::ensure!(!items.is_empty(), "item catalog contains no definitions");
        Self::new(items, designs)
    }

    pub fn item(&self, id: &str) -> Option<&ItemDefinition> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn placement(
        &self,
        item_id: &str,
        placement_id: &str,
    ) -> Option<(&ItemDefinition, &EquipmentPlacement)> {
        let item = self.item(item_id)?;
        let placement = item
            .equipment
            .as_ref()?
            .placements
            .iter()
            .find(|placement| placement.id == placement_id)?;
        Some((item, placement))
    }

    pub fn material(&self, id: &str) -> Result<EquipmentMaterial> {
        self.item(id)
            .and_then(|item| item.equipment.as_ref())
            .and_then(|equipment| equipment.material)
            .with_context(|| format!("equipment {id} has no material"))
    }

    /// The catalog default design for a parametric item.
    pub fn design(&self, id: &str) -> Option<ItemDesign> {
        self.designs.default_for(id)
    }

    /// Items with a procedural material and body surface, which the creator can generate.
    pub fn wearable(&self) -> impl Iterator<Item = &ItemDefinition> {
        self.items.iter().filter(|item| is_wearable(item))
    }
}

pub fn is_wearable(item: &ItemDefinition) -> bool {
    item.equipment.as_ref().is_some_and(|equipment| {
        equipment.material.is_some()
            && equipment
                .placements
                .iter()
                .any(|placement| !placement.surface.is_empty())
    })
}

fn validate_armor_recipes(items: &[ItemDefinition]) -> Result<()> {
    for item in items {
        anyhow::ensure!(
            !matches!(item.kind, ItemKind::Armor { .. }) || armor_recipes::is_parametric(&item.id),
            "armor {} has no authored parametric recipe",
            item.id
        );
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn authored() -> ItemCatalog {
    let document: ItemCatalogDocument =
        serde_json::from_str(include_str!("../../../content/items/catalog.yaml")).unwrap();
    ItemCatalog::new(document.items, CatalogDesigns::authored()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_cannot_route_new_armor_through_clothing_topology() {
        let mut items = authored().items;
        validate_armor_recipes(&items).unwrap();
        let armor = items
            .iter_mut()
            .find(|item| matches!(item.kind, ItemKind::Armor { .. }))
            .unwrap();
        armor.id = "unimplemented_armor".into();
        assert!(ItemCatalog::new(items, CatalogDesigns::authored()).is_err());
    }
}
