//! Borrowed equipment identities admitted against their owning catalogue.

use crate::item_catalog::{EquipmentPlacement, ItemDefinition};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

/// One actual record in an admitted catalogue. Exact-name lookup keeps the
/// catalogue's first duplicate; record admission can retain every duplicate.
#[derive(Clone, Copy, Debug)]
pub struct CatalogItem<'a> {
    definition: &'a ItemDefinition,
}

/// A placement borrowed from the equipment definition of its actual item.
/// Construction proves that relationship without imposing clothing eligibility
/// or a fixed set of placement names.
#[derive(Clone, Copy, Debug)]
pub struct CatalogPlacement<'a> {
    item: CatalogItem<'a>,
    placement: &'a EquipmentPlacement,
}

/// An admitted placement retained beyond an owned catalogue borrow, such as an
/// editor exhibit. Every placement of an item shares one immutable snapshot.
#[derive(Clone, Debug)]
pub struct OwnedCatalogPlacement {
    definition: Arc<ItemDefinition>,
    placement: RetainedPlacementOrdinal,
}

#[derive(Clone, Copy, Debug)]
struct RetainedPlacementOrdinal(usize);

impl<'a> CatalogItem<'a> {
    pub fn all(catalogue: &'a [ItemDefinition]) -> impl Iterator<Item = Self> {
        catalogue.iter().map(|definition| Self { definition })
    }

    /// Decode an exact authored, CLI, manifest or replicated item key once.
    pub fn find(catalogue: &'a [ItemDefinition], id: &str) -> Option<Self> {
        catalogue
            .iter()
            .find(|definition| definition.id == id)
            .map(|definition| Self { definition })
    }

    /// Admit an actual catalogue record, preserving duplicate-ID export order.
    pub fn admit(catalogue: &'a [ItemDefinition], record: &ItemDefinition) -> Option<Self> {
        catalogue
            .iter()
            .find(|definition| std::ptr::eq(*definition, record))
            .map(|definition| Self { definition })
    }

    pub fn definition(self) -> &'a ItemDefinition {
        self.definition
    }

    /// Decode a placement key in this item's own equipment definition.
    pub fn placement(self, id: &str) -> Option<CatalogPlacement<'a>> {
        self.placements()
            .find(|selection| selection.placement.id == id)
    }

    /// Authored placement order; consumers may explicitly choose lexical order.
    pub fn placements(self) -> impl Iterator<Item = CatalogPlacement<'a>> {
        self.definition
            .equipment
            .iter()
            .flat_map(|equipment| &equipment.placements)
            .map(move |placement| CatalogPlacement {
                item: self,
                placement,
            })
    }

    /// Retain the actual authored placements once at a long-lived selection
    /// producer. No key is decoded again when the selection is later borrowed.
    pub fn retained_placements(self) -> impl Iterator<Item = OwnedCatalogPlacement> {
        let count = self
            .definition
            .equipment
            .as_ref()
            .map_or(0, |equipment| equipment.placements.len());
        let definition = Arc::new((*self.definition).clone());
        (0..count).map(move |placement| OwnedCatalogPlacement {
            definition: Arc::clone(&definition),
            placement: RetainedPlacementOrdinal(placement),
        })
    }
}

impl OwnedCatalogPlacement {
    pub fn item(&self) -> CatalogItem<'_> {
        CatalogItem {
            definition: self.definition.as_ref(),
        }
    }

    pub fn selection(&self) -> CatalogPlacement<'_> {
        // Only retained_placements constructs this record. Its ordinal comes
        // from this exact snapshot's present equipment definition and bounds.
        let equipment = &self.definition.equipment.as_slice()[0];
        CatalogPlacement {
            item: self.item(),
            placement: &equipment.placements[self.placement.0],
        }
    }
}

impl<'a> CatalogPlacement<'a> {
    /// Decode an item/placement pair at an external input boundary.
    pub fn find(catalogue: &'a [ItemDefinition], item: &str, placement: &str) -> Option<Self> {
        CatalogItem::find(catalogue, item)?.placement(placement)
    }

    pub fn item(self) -> CatalogItem<'a> {
        self.item
    }

    pub fn definition(self) -> &'a EquipmentPlacement {
        self.placement
    }
}

impl PartialEq for CatalogItem<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.definition, other.definition)
    }
}

impl Eq for CatalogItem<'_> {}

impl Hash for CatalogItem<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.definition, state);
    }
}

impl PartialEq for CatalogPlacement<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item && std::ptr::eq(self.placement, other.placement)
    }
}

impl Eq for CatalogPlacement<'_> {}

impl Hash for CatalogPlacement<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.item.hash(state);
        std::ptr::hash(self.placement, state);
    }
}

impl Deref for CatalogItem<'_> {
    type Target = ItemDefinition;

    fn deref(&self) -> &Self::Target {
        self.definition
    }
}

impl Deref for CatalogPlacement<'_> {
    type Target = EquipmentPlacement;

    fn deref(&self) -> &Self::Target {
        self.placement
    }
}
