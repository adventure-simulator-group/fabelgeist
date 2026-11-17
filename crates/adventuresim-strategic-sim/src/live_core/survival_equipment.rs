//! Project only authored worn protection into public survival insulation.

use super::*;

impl LiveRunner {
    pub(super) fn public_equipped_insulation_bps(&self, character_id: u64) -> Option<u16> {
        let mut layers = Vec::new();
        for equipped in self
            .connection
            .db
            .character_equipped_item()
            .iter()
            .filter(|row| row.character_id == character_id)
        {
            let adventuresim_stdb_client::EquippedPlacement::Authored(placement_id) =
                &equipped.placement
            else {
                continue;
            };
            let inventory = self
                .connection
                .db
                .inventory_item()
                .iter()
                .find(|row| row.id == equipped.inventory_item_id)?;
            if inventory.character_id != character_id {
                return None;
            }
            let item = self
                .connection
                .db
                .item()
                .iter()
                .find(|row| row.id == inventory.item_id)?;
            let placement = item
                .equipment_placements
                .iter()
                .find(|placement| placement_id.value == placement.id)?;
            let protected_regions = placement
                .protection
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .len();
            layers.extend(std::iter::repeat_n(
                (item.padding, item.coverage),
                protected_regions,
            ));
        }
        Some(adventuresim_core::survival::insulation_from_layers(layers))
    }
}
