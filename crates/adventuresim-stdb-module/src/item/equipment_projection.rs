use super::*;

pub(super) fn project_equipment(
    item: &mut Item,
    definition: &adventuresim_core::item_catalog::ItemDefinition,
) {
    if let Some(equipment) = &definition.equipment {
        item.attachment_tags = equipment.attachment_tags.clone();
        item.equipment_placements = equipment
            .placements
            .iter()
            .map(|placement| PersistedEquipmentPlacement {
                id: placement.id.clone(),
                occupancy: placement.occupancy.clone(),
                parents: placement.parents.clone(),
                protection: placement.protection.clone(),
            })
            .collect();
        item.attachment_points = equipment
            .attachment_points
            .iter()
            .map(|point| PersistedEquipmentAttachmentPoint {
                id: point.id.clone(),
                channel: point.channel,
                capacity: point.capacity,
                order: point.order,
                accepts_tags: point.accepts_tags.clone(),
            })
            .collect();
        if let Some(protection) = equipment.protection {
            item.coverage = protection.coverage;
            item.resistance = protection.resistance;
            item.padding = protection.padding;
            item.flexibility = protection.flexibility;
            item.range_of_motion = protection.range_of_motion;
        }
    }
}
