//! Outfit dependencies are part of a physical fit, including when it is dropped.
use super::*;
use adventuresim_character_creator::equipment_layers::LayerPlan;
use adventuresim_core::item_catalog::{EquipmentPlacement, definition};

#[cfg(test)]
#[path = "layers_tests.rs"]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct LayerSelection {
    pub item: String,
    pub placement: String,
}

impl LayerSelection {
    pub(super) fn new(item: &str, placement: &str) -> Self {
        Self {
            item: item.into(),
            placement: placement.into(),
        }
    }

    fn definition(&self) -> anyhow::Result<&'static EquipmentPlacement> {
        definition(&self.item)
            .and_then(|item| item.equipment.as_ref())
            .and_then(|equipment| equipment.placements.iter().find(|p| p.id == self.placement))
            .with_context(|| format!("no equipment placement {} {}", self.item, self.placement))
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
            .collect::<anyhow::Result<Vec<_>>>()?;
        let plan = LayerPlan::new(&placements)?;
        Ok(Self { selections, plan })
    }

    pub(super) fn ancestors(&self, selection: &LayerSelection) -> Vec<LayerSelection> {
        let index = self
            .selections
            .binary_search(selection)
            .expect("planned selection");
        self.plan
            .ancestors(index)
            .into_iter()
            .map(|i| self.selections[i].clone())
            .collect()
    }

    pub(super) fn supports(&self, selection: &LayerSelection) -> Vec<LayerSelection> {
        let index = self
            .selections
            .binary_search(selection)
            .expect("planned selection");
        self.plan
            .supports(index)
            .map(|i| self.selections[i].clone())
            .collect()
    }
}

impl generation::FitKey {
    /// Recreate every direct support's full fit key. Its own lower surfaces
    /// remain in the key; flat item IDs alone would reuse incorrectly seated
    /// geometry after a garment underneath that support changes.
    pub(super) fn support_keys(&self) -> anyhow::Result<Vec<Self>> {
        let selection = LayerSelection::new(&self.item, &self.placement);
        let mut selections = self.layers.clone();
        selections.push(selection.clone());
        let plan = OutfitPlan::new(selections)?;
        Ok(plan
            .supports(&selection)
            .into_iter()
            .map(|support| Self {
                shape: self.shape.clone(),
                layers: plan.ancestors(&support),
                item: support.item,
                placement: support.placement,
            })
            .collect())
    }
}

pub(super) fn outfits(
    selections: impl Iterator<Item = (Entity, LayerSelection)>,
) -> HashMap<Entity, Result<OutfitPlan, String>> {
    let mut owners = HashMap::<Entity, Vec<LayerSelection>>::new();
    for (owner, selection) in selections {
        owners.entry(owner).or_default().push(selection);
    }
    owners
        .into_iter()
        .map(|(owner, selections)| {
            (
                owner,
                OutfitPlan::new(selections).map_err(|error| format!("{error:#}")),
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
                    support.item,
                    support.placement
                ),
                None => return self.next_fit(&support),
            }
        }
        Ok(key.clone())
    }
}
