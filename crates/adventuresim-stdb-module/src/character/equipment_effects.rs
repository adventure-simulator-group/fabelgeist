//! Planned equipment topology shared by scene hosts and non-live entry points.

use std::collections::BTreeSet;

use adventuresim_core::{
    equipment_placement::EquippedPlacement,
    inventory_command::{EquipmentConflictPolicy, InventoryRowId},
    item_catalog::OccupancyRequirement,
    physical_object::CustodyCharacterId,
};

use super::equipment_state::ResolvedEquipmentPlacement;
use super::*;
use attachments::{ParentSelections, SelectedAttachments};
pub(crate) use changes::{
    EquipmentChange, EquipmentChangeSet, EquipmentChanges, EquipmentDestination,
    EquipmentEffectPolicy, EquipmentExchange,
};

mod attachments;
mod changes;

type Result<T> = std::result::Result<T, String>;

struct PlacementRequest {
    actor: CustodyCharacterId,
    inventory: InventoryRowId,
    targets: ParentSelections,
    conflicts: EquipmentConflictPolicy,
}

struct PlacementPlan {
    actor: CustodyCharacterId,
    inventory: InventoryRowId,
    placement: EquippedPlacement,
    roots: Vec<RootOccupancy>,
    attachments: SelectedAttachments,
    displaced: BTreeSet<InventoryRowId>,
}

struct RootOccupancy {
    requirement_index: u16,
    requirement: OccupancyRequirement,
}

impl PlacementPlan {
    fn prepare(
        ctx: &ReducerContext,
        change: &PlacementRequest,
        definition: &crate::Item,
        placement: &ResolvedEquipmentPlacement,
    ) -> Result<Self> {
        let roots = placement
            .occupancy
            .iter()
            .copied()
            .enumerate()
            .map(|(index, requirement)| {
                Ok(RootOccupancy {
                    requirement_index: u16::try_from(index)
                        .map_err(|_| "Too many physical occupancy requirements")?,
                    requirement,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let mut displaced = conflicting_roots(ctx, change, definition, placement)?;
        let attachments = SelectedAttachments::resolve(ctx, change, definition, placement)?;
        displaced.extend(attachments.displaced.iter().copied());
        if attachments
            .occupancies
            .iter()
            .any(|entry| displaced.contains(&entry.parent))
        {
            return Err("Equipment placement would displace its own attachment parent".into());
        }
        if attachment_would_create_cycle(
            change.inventory.get(),
            attachments
                .occupancies
                .iter()
                .map(|entry| entry.parent.get()),
            |ancestor| {
                ctx.db
                    .equipment_occupancy()
                    .inventory_item_id()
                    .filter(ancestor)
                    .filter_map(|row| row.parent_inventory_item_id)
                    .collect()
            },
        ) {
            return Err("Attachment would create an equipment cycle".into());
        }
        for item in &displaced {
            ctx.db
                .character_equipped_item()
                .inventory_item_id()
                .find(item.get())
                .filter(|row| row.character_id == change.actor.get())
                .ok_or("Displaced item is not equipped by this character")?;
            require_no_equipped_children(ctx, item.get())?;
        }
        Ok(Self {
            actor: change.actor,
            inventory: change.inventory,
            placement: placement.placement.clone(),
            roots,
            attachments,
            displaced,
        })
    }

    fn commit(self, ctx: &ReducerContext) {
        unequip_wearable(ctx, self.inventory.get());
        for item in self.displaced {
            unequip_wearable(ctx, item.get());
        }
        ctx.db
            .character_equipped_item()
            .insert(CharacterEquippedItem {
                inventory_item_id: self.inventory.get(),
                character_id: self.actor.get(),
                placement: self.placement,
            });
        for entry in self.roots {
            ctx.db.equipment_occupancy().insert(EquipmentOccupancy {
                id: character_occupancy_id(
                    self.actor.get(),
                    self.inventory.get(),
                    entry.requirement_index,
                ),
                character_id: self.actor.get(),
                inventory_item_id: self.inventory.get(),
                anchor_kind: EquipmentAnchorKind::CharacterLocation,
                location: Some(entry.requirement.location),
                parent_inventory_item_id: None,
                attachment_point_id: None,
                channel: entry.requirement.channel,
                order: entry.requirement.order,
                requirement_index: entry.requirement_index,
                capacity_index: 0,
            });
        }
        for entry in self.attachments.occupancies {
            ctx.db.equipment_occupancy().insert(EquipmentOccupancy {
                id: attachment_occupancy_id(
                    entry.parent.get(),
                    entry.point_id.as_str(),
                    entry.capacity_index,
                ),
                character_id: self.actor.get(),
                inventory_item_id: self.inventory.get(),
                anchor_kind: EquipmentAnchorKind::ItemAttachment,
                location: None,
                parent_inventory_item_id: Some(entry.parent.get()),
                attachment_point_id: Some(entry.point_id.into()),
                channel: entry.requirement.channel,
                order: entry.requirement.order,
                requirement_index: entry.requirement_index.get(),
                capacity_index: entry.capacity_index,
            });
        }
    }
}

fn conflicting_roots(
    ctx: &ReducerContext,
    change: &PlacementRequest,
    definition: &crate::Item,
    placement: &ResolvedEquipmentPlacement,
) -> Result<BTreeSet<InventoryRowId>> {
    let conflicts = conflicting_equipment_roots(ctx, change.actor, change.inventory, placement)?;
    if change.conflicts == EquipmentConflictPolicy::RejectOccupied && !conflicts.is_empty() {
        let details = conflicts
            .iter()
            .map(|conflict| {
                let requirement = conflict.requirement;
                format!(
                    "{:?} ({:?}, order {})",
                    requirement.location, requirement.channel, requirement.order,
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "Can't equip {}: occupied at {details}",
            definition.id
        ));
    }
    Ok(conflicts
        .into_iter()
        .map(|conflict| conflict.inventory)
        .collect())
}
