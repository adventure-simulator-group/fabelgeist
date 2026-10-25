//! Only offer attachment parents satisfying the authored channel, side and tags.
use super::*;

pub(super) fn accepts(
    equip: &CharacterEquipmentGraph,
    moving_id: u64,
    tags: &[String],
    target: &crate::spacetimedb::EquipmentAttachmentTarget,
    requirement: &adventuresim_core::item_catalog::ParentRequirement,
) -> bool {
    target.channel == requirement.channel
        && target.order == requirement.order
        && requirement.location.is_none_or(|location| {
            equipment_item_roots(equip, target.parent_inventory_item_id)
                .iter()
                .any(|(root, ..)| *root == location)
        })
        && (target.accepts_tags.is_empty()
            || tags.iter().any(|tag| target.accepts_tags.contains(tag)))
        && !equipment_target_is_self_or_descendant(
            equip,
            moving_id,
            target.parent_inventory_item_id,
        )
}
