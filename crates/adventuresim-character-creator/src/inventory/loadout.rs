//! The worn articles, grouped by the generator that builds each on the body.

use super::{Article, EquipConflict, Inventory, InventoryItemId};
use crate::{
    equipment_catalog::{ItemCatalog, is_wearable},
    garment::GarmentSelection,
    item_catalog_schema::{EquipmentPlacement, ItemDefinition},
    item_design::ItemDesign,
};

/// Everything worn, validated to fit together.
#[derive(Debug, Default)]
pub struct Loadout<'a> {
    /// Catalog clothing shells offset from the body surface.
    pub clothing: Vec<CatalogPiece<'a>>,
    /// Catalog items built from a parametric design.
    pub fitted: Vec<FittedPiece<'a>>,
    /// Cloth, from the innermost layer out, in draping order.
    pub draped: Vec<DrapedPiece<'a>>,
    pub plate: Option<&'a fabelgeist_armor::Armor>,
}

#[derive(Clone, Copy, Debug)]
pub struct CatalogPiece<'a> {
    pub id: InventoryItemId,
    pub item: &'a ItemDefinition,
    pub placement: &'a EquipmentPlacement,
}

#[derive(Clone, Debug)]
pub struct FittedPiece<'a> {
    pub piece: CatalogPiece<'a>,
    pub design: ItemDesign,
    /// Ornament cut into the piece when its material is plate steel.
    pub engraving: Option<fabelgeist_armor::engraving::Engraving>,
}

#[derive(Clone, Copy, Debug)]
pub struct DrapedPiece<'a> {
    pub id: InventoryItemId,
    pub selection: &'a GarmentSelection,
}

impl Inventory {
    /// The worn articles, or the first reason they cannot be worn together.
    pub fn loadout<'a>(
        &'a self,
        catalog: &'a ItemCatalog,
    ) -> Result<Loadout<'a>, (InventoryItemId, EquipConflict)> {
        self.fit(catalog)?;
        let mut loadout = Loadout::default();
        let mut draped = Vec::new();
        for item in self.worn() {
            let fail = |conflict| (item.id, conflict);
            match &item.article {
                Article::Catalog(article) => {
                    let (definition, placement) = article.resolve(catalog).map_err(fail)?;
                    if !is_wearable(definition) || placement.surface.is_empty() {
                        return Err(fail(EquipConflict::Invalid(format!(
                            "{} has no generated body surface",
                            definition.display_name
                        ))));
                    }
                    let piece = CatalogPiece {
                        id: item.id,
                        item: definition,
                        placement,
                    };
                    match article.design(catalog).map_err(fail)? {
                        Some(design) => loadout.fitted.push(FittedPiece {
                            piece,
                            design,
                            engraving: article.engraving.clone(),
                        }),
                        None => loadout.clothing.push(piece),
                    }
                }
                Article::Draped(selection) => {
                    let layer = item.article.occupancy(catalog).map_err(fail)?.layer();
                    draped.push((
                        layer.order(),
                        DrapedPiece {
                            id: item.id,
                            selection,
                        },
                    ));
                }
                Article::Plate(armor) => loadout.plate = Some(armor),
            }
        }
        draped.sort_by_key(|(layer, _)| *layer);
        loadout.draped = draped.into_iter().map(|(_, piece)| piece).collect();
        Ok(loadout)
    }
}
