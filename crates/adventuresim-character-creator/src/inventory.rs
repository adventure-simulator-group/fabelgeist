//! A character's belongings: every article carried, and which of them are worn.
//!
//! Catalog items, draped garments and plate armor are all articles. Whether
//! worn articles fit together follows the catalog's body occupancy and
//! attachment rules, through the same equipment graph the game uses.

mod loadout;
mod occupancy;

pub use loadout::{CatalogPiece, DrapedPiece, FittedPiece, Loadout};
pub use occupancy::{Occupancy, WornGraph};

use crate::{
    equipment_catalog::ItemCatalog,
    garment::GarmentSelection,
    item_catalog_schema::{
        EquipmentChannel, EquipmentLocation, EquipmentPlacement, ItemDefinition,
    },
    item_design::ItemDesign,
};
use serde::{Deserialize, Serialize};

/// Stable identity of one article within a character's inventory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InventoryItemId(pub u64);

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Inventory {
    items: Vec<InventoryItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryItem {
    pub id: InventoryItemId,
    /// Worn articles are generated on the body; the rest are only carried.
    pub worn: bool,
    pub article: Article,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Article {
    Catalog(CatalogArticle),
    /// Cloth sewn from a pattern, or fitted as a surface, and draped by simulation.
    Draped(GarmentSelection),
    /// The Fabelgeist breastplate and fauld builder.
    Plate(fabelgeist_armor::Armor),
}

/// One catalog item in one of its authored placements.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogArticle {
    pub item_id: String,
    pub placement_id: String,
    /// This article's own shape; `None` builds the catalog default.
    pub design: Option<ItemDesign>,
}

/// Why an article cannot be worn with the rest of the outfit.
#[derive(Clone, Debug, PartialEq)]
pub enum EquipConflict {
    /// Another worn article already fills this body cell.
    Occupied {
        location: EquipmentLocation,
        channel: EquipmentChannel,
        by: InventoryItemId,
    },
    /// No worn article offers the attachment point this article hangs from.
    Unsupported {
        channel: EquipmentChannel,
        location: Option<EquipmentLocation>,
    },
    Invalid(String),
}

impl CatalogArticle {
    pub fn new(item_id: impl Into<String>, placement_id: impl Into<String>) -> Self {
        Self {
            item_id: item_id.into(),
            placement_id: placement_id.into(),
            design: None,
        }
    }

    pub fn resolve<'a>(
        &self,
        catalog: &'a ItemCatalog,
    ) -> Result<(&'a ItemDefinition, &'a EquipmentPlacement), EquipConflict> {
        catalog
            .placement(&self.item_id, &self.placement_id)
            .ok_or_else(|| {
                EquipConflict::Invalid(format!(
                    "the catalog has no {} placement {}",
                    self.item_id, self.placement_id
                ))
            })
    }

    /// The shape this article is built from, if its item is parametric.
    pub fn design(&self, catalog: &ItemCatalog) -> Result<Option<ItemDesign>, EquipConflict> {
        let default = catalog.design(&self.item_id);
        match (&self.design, default) {
            (None, default) => Ok(default),
            (Some(own), Some(default)) if default.same_family(own) => Ok(Some(own.clone())),
            (Some(_), _) => Err(EquipConflict::Invalid(format!(
                "{} has a design of another construction",
                self.item_id
            ))),
        }
    }
}

impl Article {
    pub fn name(&self, catalog: &ItemCatalog) -> String {
        match self {
            Self::Catalog(article) => match article.resolve(catalog) {
                Ok((item, _)) if placement_count(item) > 1 => {
                    format!("{} ({})", item.display_name, article.placement_id)
                }
                Ok((item, _)) => item.display_name.clone(),
                Err(_) => format!("Unknown {}", article.item_id),
            },
            Self::Draped(selection) => {
                let preset = selection.preset.label().to_lowercase();
                format!("{} {preset}", selection.fabric.label())
            }
            Self::Plate(armor) => {
                use fabelgeist_armor::Construction;
                let construction = match armor.construction {
                    Construction::Solid => "Solid",
                    Construction::Lamellar => "Lamellar",
                    Construction::Scale => "Scale",
                };
                format!("{construction} plate breastplate")
            }
        }
    }

    /// Catalog weight; generated garments and plate have no authored weight.
    pub fn weight_kg(&self, catalog: &ItemCatalog) -> Option<f32> {
        match self {
            Self::Catalog(article) => article
                .resolve(catalog)
                .ok()
                .map(|(item, _)| item.weight_kg),
            Self::Draped(_) | Self::Plate(_) => None,
        }
    }

    fn validate(&self) -> Result<(), String> {
        match self {
            Self::Catalog(article) => {
                if article.item_id.is_empty() || article.placement_id.is_empty() {
                    return Err("catalog articles require item and placement IDs".into());
                }
                article
                    .design
                    .as_ref()
                    .map_or(Ok(()), ItemDesign::validate)
                    .map_err(|error| format!("{}: {error:#}", article.item_id))
            }
            Self::Draped(selection) => selection.validate().map_err(|error| error.to_string()),
            Self::Plate(armor) => armor.validate(),
        }
    }
}

fn placement_count(item: &ItemDefinition) -> usize {
    item.equipment
        .as_ref()
        .map_or(0, |equipment| equipment.placements.len())
}

impl Inventory {
    pub fn items(&self) -> &[InventoryItem] {
        &self.items
    }

    pub fn get(&self, id: InventoryItemId) -> Option<&InventoryItem> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn get_mut(&mut self, id: InventoryItemId) -> Option<&mut InventoryItem> {
        self.items.iter_mut().find(|item| item.id == id)
    }

    pub fn worn(&self) -> impl Iterator<Item = &InventoryItem> {
        self.items.iter().filter(|item| item.worn)
    }

    /// Carry a new article without wearing it.
    pub fn add(&mut self, article: Article) -> InventoryItemId {
        let id = InventoryItemId(
            self.items
                .iter()
                .map(|item| item.id.0 + 1)
                .max()
                .unwrap_or(1),
        );
        self.items.push(InventoryItem {
            id,
            worn: false,
            article,
        });
        id
    }

    /// Discard an article, taking off whatever hangs from it.
    pub fn remove(&mut self, id: InventoryItemId, catalog: &ItemCatalog) -> Vec<InventoryItemId> {
        let taken_off = self.take_off(id, catalog);
        self.items.retain(|item| item.id != id);
        taken_off
    }

    /// Move an article earlier or later; draped garments of one layer drape in this order.
    pub fn shift(&mut self, id: InventoryItemId, later: bool) {
        let Some(index) = self.items.iter().position(|item| item.id == id) else {
            return;
        };
        let target = if later {
            index + 1
        } else {
            index.wrapping_sub(1)
        };
        if target < self.items.len() {
            self.items.swap(index, target);
        }
    }

    /// Wear an article, taking off whatever occupies its place. Returns what was taken off.
    pub fn wear(
        &mut self,
        id: InventoryItemId,
        catalog: &ItemCatalog,
    ) -> Result<Vec<InventoryItemId>, EquipConflict> {
        let before = self.items.clone();
        self.set_worn(id, true);
        let mut displaced = Vec::new();
        loop {
            match self.fit(catalog) {
                Ok(_) => return Ok(displaced),
                Err((culprit, EquipConflict::Occupied { by, .. })) if culprit == id || by == id => {
                    let other = if culprit == id { by } else { culprit };
                    displaced.extend(self.take_off(other, catalog));
                }
                Err((_, conflict)) => {
                    self.items = before;
                    return Err(conflict);
                }
            }
        }
    }

    /// Stop wearing an article and whatever hangs from it. Returns everything taken off.
    pub fn take_off(&mut self, id: InventoryItemId, catalog: &ItemCatalog) -> Vec<InventoryItemId> {
        if !self.get(id).is_some_and(|item| item.worn) {
            return Vec::new();
        }
        self.set_worn(id, false);
        let mut taken_off = vec![id];
        while let Err((culprit, EquipConflict::Unsupported { .. })) = self.fit(catalog) {
            self.set_worn(culprit, false);
            taken_off.push(culprit);
        }
        taken_off
    }

    fn set_worn(&mut self, id: InventoryItemId, worn: bool) {
        if let Some(item) = self.get_mut(id) {
            item.worn = worn;
        }
    }

    /// Join every worn article into one equipment graph.
    pub fn fit(
        &self,
        catalog: &ItemCatalog,
    ) -> Result<WornGraph, (InventoryItemId, EquipConflict)> {
        let candidates = self
            .worn()
            .map(|item| {
                let occupancy = item
                    .article
                    .occupancy(catalog)
                    .map_err(|conflict| (item.id, conflict))?;
                let equipment = match &item.article {
                    Article::Catalog(article) => catalog
                        .item(&article.item_id)
                        .and_then(|definition| definition.equipment.as_ref()),
                    Article::Draped(_) | Article::Plate(_) => None,
                };
                Ok(occupancy::Candidate {
                    id: item.id,
                    occupancy,
                    equipment,
                    attachment_tags: equipment.map_or(&[], |e| e.attachment_tags.as_slice()),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        WornGraph::build(candidates)
    }

    /// Presentation text for why `item` cannot be worn.
    pub fn explain(
        &self,
        catalog: &ItemCatalog,
        (item, conflict): &(InventoryItemId, EquipConflict),
    ) -> String {
        let name = |id: InventoryItemId| {
            self.get(id)
                .map_or_else(|| id.to_string(), |item| item.article.name(catalog))
        };
        format!(
            "{} cannot be worn: {}",
            name(*item),
            conflict.describe(name)
        )
    }

    pub fn validate(&self) -> Result<(), String> {
        for (index, item) in self.items.iter().enumerate() {
            if self.items[..index].iter().any(|other| other.id == item.id) {
                return Err(format!("inventory repeats item {}", item.id.0));
            }
            item.article.validate()?;
        }
        Ok(())
    }
}

impl std::fmt::Display for InventoryItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl EquipConflict {
    /// Presentation text, naming other articles through `name`.
    pub fn describe(&self, name: impl Fn(InventoryItemId) -> String) -> String {
        match self {
            Self::Occupied {
                location,
                channel,
                by,
            } => format!(
                "{} already fills the {} {} layer",
                name(*by),
                location_label(*location),
                channel_label(*channel).to_lowercase()
            ),
            Self::Unsupported { channel, location } => format!(
                "needs a worn {} garment to attach to{}",
                channel_label(*channel).to_lowercase(),
                location.map_or(String::new(), |location| format!(
                    " on the {}",
                    location_label(location)
                ))
            ),
            Self::Invalid(reason) => reason.clone(),
        }
    }
}

pub fn channel_label(channel: EquipmentChannel) -> &'static str {
    match channel {
        EquipmentChannel::Held => "Held",
        EquipmentChannel::BaseClothing => "Clothing",
        EquipmentChannel::Padding => "Padding",
        EquipmentChannel::FlexibleArmor => "Mail",
        EquipmentChannel::RigidArmor => "Plate",
        EquipmentChannel::Outerwear => "Outerwear",
        EquipmentChannel::Accessory => "Accessories",
        EquipmentChannel::Mount => "Mounted",
        EquipmentChannel::Containment => "Containers",
    }
}

pub fn location_label(location: EquipmentLocation) -> &'static str {
    use EquipmentLocation::*;
    match location {
        Head => "head",
        Face => "face",
        Neck => "neck",
        Chest => "chest",
        Stomach => "stomach",
        Back => "back",
        LeftShoulder => "left shoulder",
        RightShoulder => "right shoulder",
        LeftArm => "left arm",
        RightArm => "right arm",
        LeftHand => "left hand",
        RightHand => "right hand",
        LeftLeg => "left leg",
        RightLeg => "right leg",
        LeftFoot => "left foot",
        RightFoot => "right foot",
        LeftBelt => "left belt",
        RightBelt => "right belt",
        FrontBelt => "front belt",
        BackBelt => "back belt",
        LeftPocket => "left pocket",
        RightPocket => "right pocket",
        BackLeftPocket => "back left pocket",
        BackRightPocket => "back right pocket",
    }
}

#[cfg(test)]
mod tests;
