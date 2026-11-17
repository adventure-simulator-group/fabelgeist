//! Save and restore authored or generic-held equipment through repair custody.

use adventuresim_core::{inventory_command::InventoryRowId, physical_object::CustodyCharacterId};
use spacetimedb::ReducerContext;

use super::RepairOrder;

impl RepairOrder {
    pub(super) fn restore_equipment(
        &self,
        ctx: &ReducerContext,
        actor: CustodyCharacterId,
    ) -> Result<(), String> {
        if let Some(placement) = self.equipped_placement.as_ref() {
            crate::character::restore_equipment_placement(
                ctx,
                actor,
                InventoryRowId::try_new(self.inventory_item_id)
                    .map_err(|error| error.to_string())?,
                placement,
                self.attachment_targets.clone(),
            )
            .map_err(|error| {
                format!(
                    "Repair is complete, but its saved equipment graph cannot be restored: {error}"
                )
            })?;
        }
        Ok(())
    }
}

pub(super) fn saved_attachment_targets(
    rows: impl IntoIterator<Item = crate::character::EquipmentOccupancy>,
) -> Vec<crate::character::EquipmentAttachmentTargetSelection> {
    let mut targets = rows
        .into_iter()
        .filter_map(|row| {
            Some(crate::character::EquipmentAttachmentTargetSelection {
                requirement_index: row.requirement_index,
                parent_inventory_item_id: row.parent_inventory_item_id?,
                attachment_point_id: row.attachment_point_id?,
            })
        })
        .collect::<Vec<_>>();
    targets.sort_by_key(|target| target.requirement_index);
    targets
}
