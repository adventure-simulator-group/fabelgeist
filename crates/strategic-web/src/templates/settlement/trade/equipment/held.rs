//! Generic hand choices share the existing keyboard picker and occupancy graph.

use super::*;
use crate::spacetimedb::core_equipment_location;
use adventuresim_core::equipment_placement::EquipmentHand;

pub(super) fn placement_options(equip: Option<&CharacterEquipmentGraph>) -> Vec<serde_json::Value> {
    [EquipmentHand::Left, EquipmentHand::Right].into_iter().map(|hand| {
        let location = web_equipment_location(hand.location());
        let bindings: Vec<_> = INPUT_ADDRESS_MAPPINGS.iter()
            .filter(|binding| equipment_binding_contains(binding, location)).collect();
        let occupant = equip.and_then(|equip| {
            equip.equipment_occupancies.iter().find(|row| {
                row.location.map(core_equipment_location) == Some(location) && core_equipment_channel(row.channel) == CoreEquipmentChannel::Held
            }).map(|row| equipment_occupant_json(equip, row.inventory_item_id))
        });
        let input_ranks: serde_json::Map<_, _> = bindings.iter().map(|binding| {
            (binding.input.to_owned(), serde_json::json!(equipment_channel_rank(CoreEquipmentChannel::Held) * 1_000))
        }).collect();
        let input_occupants: serde_json::Map<_, _> = bindings.iter().filter_map(|binding| {
            occupant.clone().map(|occupant| (binding.input.to_owned(), occupant))
        }).collect();
        serde_json::json!({
            "destination": match hand { EquipmentHand::Left => "held_left", EquipmentHand::Right => "held_right" },
            "label": equipment_location_display(hand.location()),
            "hasBody": true,
            "inputRanks": input_ranks,
            "inputOccupants": input_occupants,
            "requirements": [],
        })
    }).collect()
}
