//! Authored equipment boundary, including the no-placeholder armor invariant.

use crate::{
    armor_recipes,
    item_catalog_schema::{
        EquipmentMaterial, ItemCatalogDocument, ItemDefinition, ItemKind,
    },
    item_design::{CatalogDesigns, ItemDesign},
};
use anyhow::{Context, Result};
pub use adventuresim_core::equipment_assets::{CatalogItem, CatalogPlacement};

/// The item definitions a character can wear, with each parametric item's default design.
#[derive(Clone, Debug)]
pub struct ItemCatalog {
    pub items: Vec<ItemDefinition>,
    pub designs: CatalogDesigns,
}

/// A validated export selection. Input spelling is admitted against this catalog
/// once; duplicate catalog records with the same identity remain selected.
pub struct EquipmentItemSelection<'a> {
    ids: Vec<&'a str>,
}

impl<'a> EquipmentItemSelection<'a> {
    pub fn admit(catalog: &'a ItemCatalog, filters: &[String]) -> Result<Self> {
        let mut ids = Vec::new();
        for filter in filters {
            let Some(item) = catalog.wearable().find(|item| item.id == *filter) else {
                anyhow::bail!("unknown procedural equipment item {filter}");
            };
            ids.push(item.definition().id.as_str());
        }
        Ok(Self { ids })
    }

    pub fn is_filtered(&self) -> bool {
        !self.ids.is_empty()
    }

    pub fn includes(&self, item: CatalogItem<'_>) -> bool {
        self.ids.is_empty() || self.ids.contains(&item.id.as_str())
    }
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

    pub fn item(&self, id: &str) -> Option<CatalogItem<'_>> {
        CatalogItem::find(&self.items, id)
    }

    pub fn placement(&self, item_id: &str, placement_id: &str) -> Option<CatalogPlacement<'_>> {
        self.item(item_id)?.placement(placement_id)
    }

    /// The catalog default design for a parametric item.
    pub fn design(&self, item: CatalogItem<'_>) -> Option<ItemDesign> {
        self.designs.default_for(item)
    }

    /// The catalog design for a parametric item worn in `placement`.
    pub fn placed_design(&self, placement: CatalogPlacement<'_>) -> Option<ItemDesign> {
        self.designs.default_at(placement)
    }

    /// Items with a procedural material and body surface, which the creator can generate.
    pub fn wearable(&self) -> impl Iterator<Item = CatalogItem<'_>> {
        CatalogItem::all(&self.items).filter(|item| is_wearable(item.definition()))
    }
}

pub fn material(item: CatalogItem<'_>) -> Result<EquipmentMaterial> {
    item.equipment
        .as_ref()
        .and_then(|equipment| equipment.material)
        .with_context(|| format!("equipment {} has no material", item.id))
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
    for item in CatalogItem::all(items) {
        anyhow::ensure!(
            !matches!(item.kind, ItemKind::Armor { .. }) || armor_recipes::is_parametric(item),
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
