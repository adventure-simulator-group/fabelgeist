use crate::spacetimedb::{
    CharacterEquipmentGraph, EquipmentOccupancy, core_equipment_channel, core_equipment_location,
};
use adventuresim_core::item_catalog::{self, OccupancyRequirement};

pub(super) fn reservation_conflicts(
    graph: &CharacterEquipmentGraph,
    occupied: &EquipmentOccupancy,
    proposed: OccupancyRequirement,
) -> bool {
    let Some(location) = occupied.location.map(core_equipment_location) else {
        return false;
    };
    let fit_zone = graph
        .equipment_nodes
        .iter()
        .find(|node| node.inventory_item_id == occupied.inventory_item_id)
        .and_then(|node| {
            item_catalog::definition(&node.item_name)?
                .equipment
                .as_ref()?
                .placements
                .iter()
                .find(|placement| placement.id == node.placement_id)
        })
        .and_then(|placement| {
            placement
                .occupancy
                .get(usize::from(occupied.requirement_index))
        })
        .and_then(|requirement| requirement.fit_zone);
    proposed.conflicts_with(OccupancyRequirement {
        location,
        channel: core_equipment_channel(occupied.channel),
        order: occupied.order,
        fit_zone,
    })
}
