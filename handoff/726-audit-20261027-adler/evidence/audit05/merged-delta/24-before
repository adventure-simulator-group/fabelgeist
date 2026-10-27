//! Outfit dependencies are part of a physical fit, including when it is dropped.
use super::*;
use adventuresim_character_creator::equipment_layers::LayerPlan;
use adventuresim_core::equipment_assets::{CatalogItem, CatalogPlacement};
use adventuresim_core::item_catalog::{self, EquipmentPlacement};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[cfg(test)]
#[path = "layers_tests.rs"]
mod tests;

/// A cache selection retains the actual catalogue records. Its value identity
/// remains the exact item/placement pair, so fit keys keep their lexical order.
#[derive(Clone, Copy, Debug)]
pub(super) struct LayerSelection {
    pub selection: CatalogPlacement<'static>,
}

impl LayerSelection {
    /// Item/placement text from ECS presentation enters the catalogue here.
    pub(super) fn admit(item: &str, placement: &str) -> anyhow::Result<Self> {
        let selection = CatalogItem::find(item_catalog::catalog(), item)
            .and_then(|item| item.placement(placement))
            .with_context(|| format!("no equipment placement {item} {placement}"))?;
        Ok(Self { selection })
    }

    fn definition(&self) -> &'static EquipmentPlacement {
        self.selection.definition()
    }

    fn key(&self) -> (&'static str, &'static str) {
        (
            self.selection.item().definition().id.as_str(),
            self.selection.definition().id.as_str(),
        )
    }
}

impl From<CatalogPlacement<'static>> for LayerSelection {
    fn from(selection: CatalogPlacement<'static>) -> Self {
        Self { selection }
    }
}

impl PartialEq for LayerSelection {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for LayerSelection {}

impl Hash for LayerSelection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}

impl PartialOrd for LayerSelection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LayerSelection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

/// A deterministic list permits sharing fits between identically dressed
/// wearers regardless of ECS entity allocation or inventory iteration order.
pub(super) struct OutfitPlan {
    selections: Vec<LayerSelection>,
    plan: LayerPlan,
}

impl OutfitPlan {
    pub(super) fn new(mut selections: Vec<LayerSelection>) -> anyhow::Result<Self> {
        selections.sort();
        selections.dedup();
        let placements = selections
            .iter()
            .map(LayerSelection::definition)
            .collect::<Vec<_>>();
        let plan = LayerPlan::new(&placements)?;
        Ok(Self { selections, plan })
    }

    pub(super) fn ancestors(
        &self,
        selection: &LayerSelection,
    ) -> anyhow::Result<Vec<LayerSelection>> {
        let index = self
            .selections
            .binary_search(selection)
            .map_err(|_| anyhow::anyhow!("equipment selection is absent from the fitted outfit"))?;
        Ok(self
            .plan
            .ancestors(index)
            .into_iter()
            .map(|i| self.selections[i])
            .collect())
    }

    pub(super) fn supports(
        &self,
        selection: &LayerSelection,
    ) -> anyhow::Result<Vec<LayerSelection>> {
        let index = self
            .selections
            .binary_search(selection)
            .map_err(|_| anyhow::anyhow!("equipment selection is absent from the fitted outfit"))?;
        Ok(self
            .plan
            .supports(index)
            .map(|i| self.selections[i])
            .collect())
    }
}

impl generation::FitKey {
    /// Recreate every direct support's full fit key. Its own lower surfaces
    /// remain in the key; flat item IDs alone would reuse incorrectly seated
    /// geometry after a garment underneath that support changes.
    pub(super) fn support_keys(&self) -> anyhow::Result<Vec<Self>> {
        let selection = self.selection;
        let mut selections = self.layers.clone();
        selections.push(selection);
        let plan = OutfitPlan::new(selections)?;
        plan.supports(&selection)?
            .into_iter()
            .map(|support| {
                Ok(Self {
                    shape: self.shape.clone(),
                    layers: plan.ancestors(&support)?,
                    selection: support,
                })
            })
            .collect()
    }
}

pub(super) fn outfits(
    selections: impl Iterator<Item = (Entity, anyhow::Result<LayerSelection>)>,
) -> HashMap<Entity, Result<OutfitPlan, String>> {
    let mut owners = HashMap::<Entity, Result<Vec<LayerSelection>, String>>::new();
    for (owner, selection) in selections {
        let accumulated = owners.entry(owner).or_insert_with(|| Ok(Vec::new()));
        if let Ok(selections) = accumulated {
            match selection {
                Ok(selection) => selections.push(selection),
                Err(error) => *accumulated = Err(format!("{error:#}")),
            }
        }
    }
    owners
        .into_iter()
        .map(|(owner, selections)| {
            (
                owner,
                selections.and_then(|selections| {
                    OutfitPlan::new(selections).map_err(|error| format!("{error:#}"))
                }),
            )
        })
        .collect()
}

impl RuntimeEquipmentBodyCache {
    /// Select an innermost missing support before the requested item. Failure
    /// propagates outward instead of silently fitting through a missing layer.
    pub(super) fn next_fit(
        &mut self,
        key: &generation::FitKey,
    ) -> anyhow::Result<generation::FitKey> {
        for support in key.support_keys()? {
            self.last_used.insert(support.clone(), self.use_clock);
            match self.models.get(&support) {
                Some(Ok(_)) => {}
                Some(Err(error)) => bail!(
                    "lower equipment {} {} failed: {error}",
                    support.selection.item().id,
                    support.selection.id
                ),
                None => return self.next_fit(&support),
            }
        }
        Ok(key.clone())
    }
}
