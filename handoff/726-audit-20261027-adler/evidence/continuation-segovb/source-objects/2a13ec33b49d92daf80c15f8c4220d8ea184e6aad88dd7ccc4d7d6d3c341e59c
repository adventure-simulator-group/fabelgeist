//! Resolve authored fixture attachment requirements to already equipped garments.
use super::*;

pub(super) fn occupancies(
    item: &ItemDefinition,
    placement: &EquipmentPlacement,
    previous: &[(&ItemDefinition, Entity, &EquipmentPlacement)],
) -> Vec<EquipmentTopologyOccupancy> {
    let mut result = placement
        .occupancy
        .iter()
        .enumerate()
        .map(|(index, requirement)| EquipmentTopologyOccupancy {
            occupancy_id: format!("armor-review:{}:{}:{index}", item.id, placement.id),
            anchor: TacticalEquipmentAnchor::CharacterLocation(requirement.location),
            channel: requirement.channel,
            order: requirement.order,
            requirement_index: index as u16,
            capacity_index: 0,
        })
        .collect::<Vec<_>>();
    let equipment = item.equipment.as_ref().expect("harness equipment");
    for (index, requirement) in placement.parents.iter().enumerate() {
        let (parent, point) = previous
            .iter()
            .find_map(|(definition, entity, parent_placement)| {
                if requirement.location.is_some_and(|location| {
                    !parent_placement
                        .occupancy
                        .iter()
                        .any(|occupied| occupied.location == location)
                }) {
                    return None;
                }
                definition
                    .equipment
                    .as_ref()?
                    .attachment_points
                    .iter()
                    .find(|point| {
                        point.channel == requirement.channel
                            && point.order == requirement.order
                            && point.capacity > 0
                            && (point.accepts_tags.is_empty()
                                || point
                                    .accepts_tags
                                    .iter()
                                    .any(|tag| equipment.attachment_tags.contains(tag)))
                    })
                    .map(|point| (*entity, point))
            })
            .expect("harness must equip a compatible supporting garment first");
        result.push(EquipmentTopologyOccupancy {
            occupancy_id: format!("armor-review:{}:{}:parent:{index}", item.id, placement.id),
            anchor: TacticalEquipmentAnchor::ItemAttachment {
                parent,
                attachment_point_id: point.id.clone(),
            },
            channel: requirement.channel,
            order: requirement.order,
            requirement_index: index as u16,
            capacity_index: 0,
        });
    }
    result
}
