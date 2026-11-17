//! Labels for authored fit alternatives, protection and current conflicts.

use super::*;
use crate::spacetimedb::CatalogItemView;

pub(super) fn placement_labels(
    inventory: &InventoryItem,
    definition: &CatalogItemView,
    equip: Option<&CharacterEquipmentGraph>,
) -> String {
    definition
        .equipment_placements
        .iter()
        .map(|placement| {
            let anchors = placement
                .occupancy
                .iter()
                .map(|requirement| {
                    format!(
                        "{} · {} · depth {}",
                        equipment_location_wire_label(requirement.location),
                        equipment_channel_label(requirement.channel),
                        requirement.order
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let parent = placement
                .parents
                .iter()
                .map(|parent| {
                    format!(
                        "attached via {} · depth {}",
                        equipment_channel_label(parent.channel),
                        parent.order
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let protection = placement
                .protection
                .iter()
                .copied()
                .map(equipment_body_part_display)
                .collect::<Vec<_>>()
                .join(", ");
            let conflicts = equip
                .into_iter()
                .flat_map(|equip| {
                    equip
                        .equipment_occupancies
                        .iter()
                        .map(move |row| (equip, row))
                })
                .filter(|(_, occupied)| occupied.inventory_item_id != inventory.id)
                .filter(|(equip, occupied)| {
                    placement.occupancy.iter().any(|requirement| {
                        fit::reservation_conflicts(equip, occupied, *requirement)
                    })
                })
                .map(|(_, occupied)| format!("#{}", occupied.inventory_item_id))
                .collect::<Vec<_>>();
            let anchor_or_parent = if parent.is_empty() { anchors } else { parent };
            format!(
                "{}: {}{}{}",
                placement.id,
                anchor_or_parent,
                if !protection.is_empty() {
                    {
                        format!(
                            "; protects {protection} ({:.0}% coverage)",
                            definition.coverage * 100.0
                        )
                    }
                } else {
                    Default::default()
                },
                if !conflicts.is_empty() {
                    format!("; conflict with {}", conflicts.join(", "))
                } else {
                    Default::default()
                }
            )
        })
        .collect::<Vec<_>>()
        .join("|")
}
