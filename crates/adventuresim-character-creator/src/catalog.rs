//! Authored equipment boundary, including the no-placeholder armor invariant.
use super::*;
use adventuresim_character_creator::item_catalog_schema::{EquipmentMaterial, ItemKind};
use adventuresim_character_creator::{armor_design_input::ArmorDesigns, armor_recipes};

#[derive(Resource)]
pub(super) struct EquipmentCatalog(pub Vec<ItemDefinition>, pub ArmorDesigns);

impl EquipmentCatalog {
    pub(super) fn material(&self, id: &str) -> Result<EquipmentMaterial> {
        self.0
            .iter()
            .find(|item| item.id == id)
            .and_then(|item| item.equipment.as_ref())
            .and_then(|e| e.material)
            .with_context(|| format!("equipment {id} has no material"))
    }
    pub(super) fn design(&self, id: &str) -> Option<armor_recipes::ParametricDesign> {
        self.1
            .get(id)
            .cloned()
            .or_else(|| armor_recipes::recipe(id))
    }
}

pub(super) fn load_item_catalog(directory: &std::path::Path) -> Result<Vec<ItemDefinition>> {
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
    validate_armor_recipes(&items)?;
    Ok(items)
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

pub(super) fn procedural_items(
    catalog: &EquipmentCatalog,
) -> impl Iterator<Item = &ItemDefinition> {
    catalog.0.iter().filter(|item| {
        item.equipment.as_ref().is_some_and(|equipment| {
            equipment.material.is_some()
                && equipment
                    .placements
                    .iter()
                    .any(|placement| !placement.surface.is_empty())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_cannot_route_new_armor_through_clothing_topology() {
        let mut document: ItemCatalogDocument =
            serde_json::from_str(include_str!("../../../content/items/catalog.yaml")).unwrap();
        validate_armor_recipes(&document.items).unwrap();
        let armor = document
            .items
            .iter_mut()
            .find(|item| matches!(item.kind, ItemKind::Armor { .. }))
            .unwrap();
        armor.id = "unimplemented_armor".into();
        assert!(validate_armor_recipes(&document.items).is_err());
    }
}
