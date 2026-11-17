//! Authored equipment effects shared by scene hosts and non-live entry points.

use std::collections::BTreeSet;

use adventuresim_core::{
    inventory_command::{EquipmentConflictPolicy, EquipmentPlacementIndex, InventoryRowId},
    item_catalog::OccupancyRequirement,
    physical_object::{CarriedInventoryScope, CustodyCharacterId},
};

use super::*;
use attachments::SelectedAttachments;

mod attachments;

type Result<T> = std::result::Result<T, String>;

pub(crate) struct EquipmentPlacementChange {
    pub(crate) actor: CustodyCharacterId,
    pub(crate) inventory: InventoryRowId,
    pub(crate) placement: EquipmentPlacementIndex,
    /// Framework selections are admitted against the actual parent catalog.
    pub(crate) targets: Vec<EquipmentAttachmentTargetSelection>,
    pub(crate) policy: EquipmentEffectPolicy,
    pub(crate) conflicts: EquipmentConflictPolicy,
}

pub(crate) struct EquipmentRemoval {
    pub(crate) actor: CustodyCharacterId,
    pub(crate) inventory: InventoryRowId,
}

struct PlacementPlan {
    actor: CustodyCharacterId,
    inventory: InventoryRowId,
    placement_id: String,
    roots: Vec<RootOccupancy>,
    attachments: SelectedAttachments,
    displaced: BTreeSet<InventoryRowId>,
}

struct RootOccupancy {
    requirement_index: u16,
    requirement: OccupancyRequirement,
}

#[derive(Clone, Copy)]
pub(crate) enum EquipmentEffectPolicy {
    Gameplay,
    /// The caller refreshes derived state after assembling the whole loadout.
    /// Initial character creation also applies its final compliance pass.
    LoadoutAssembly,
}

impl EquipmentPlacementChange {
    pub(crate) fn apply(self, ctx: &ReducerContext) -> Result<()> {
        require_living_character(ctx, self.actor.get())?;
        let inventory = ctx
            .db
            .inventory_item()
            .id()
            .find(self.inventory.get())
            .filter(|row| row.character_id == self.actor.get())
            .ok_or("Inventory item is not carried by this character")?;
        let definition = ctx
            .db
            .item()
            .id()
            .find(&inventory.item_id)
            .ok_or("Inventory item has no catalog definition")?;
        let placement = definition
            .equipment_placements
            .get(usize::from(self.placement.get()))
            .ok_or("Equipment placement is not authored for this item")?;
        if definition.kind == crate::item::CatalogItemKind::Weapon {
            require_weapon_carry_placement(&inventory.item_id, placement)?;
        }
        if matches!(self.policy, EquipmentEffectPolicy::Gameplay) {
            crate::equipment_law::require_item_legal(ctx, self.actor.get(), self.inventory.get())?;
        }
        // Any later rejection propagates to the reducer transaction, including
        // this detachment. Equipment tables are changed only by the final plan.
        crate::inventory_container::detach_row_for_action(
            ctx,
            CarriedInventoryScope::Personal,
            self.inventory.get(),
        )?;
        PlacementPlan::prepare(ctx, &self, &definition, placement)?.commit(ctx);
        match self.policy {
            EquipmentEffectPolicy::Gameplay => refresh_equipment_dependents(ctx, self.actor.get()),
            EquipmentEffectPolicy::LoadoutAssembly => Ok(()),
        }
    }
}

impl EquipmentRemoval {
    pub(crate) fn apply(self, ctx: &ReducerContext) -> Result<()> {
        require_living_character(ctx, self.actor.get())?;
        ctx.db
            .inventory_item()
            .id()
            .find(self.inventory.get())
            .filter(|row| row.character_id == self.actor.get())
            .ok_or("Inventory item is not carried by this character")?;
        ctx.db
            .character_equipped_item()
            .inventory_item_id()
            .find(self.inventory.get())
            .filter(|row| row.character_id == self.actor.get())
            .ok_or("Item is not equipped by this character")?;
        require_no_equipped_children(ctx, self.inventory.get())?;
        unequip_wearable(ctx, self.inventory.get());
        refresh_equipment_dependents(ctx, self.actor.get())
    }
}

impl PlacementPlan {
    fn prepare(
        ctx: &ReducerContext,
        change: &EquipmentPlacementChange,
        definition: &crate::Item,
        placement: &crate::item::PersistedEquipmentPlacement,
    ) -> Result<Self> {
        if ctx
            .db
            .character_equipped_item()
            .inventory_item_id()
            .find(change.inventory.get())
            .is_some()
        {
            require_no_equipped_children(ctx, change.inventory.get())?;
        }
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
            placement_id: placement.id.clone(),
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
                placement_id: self.placement_id,
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
                    &entry.point_id,
                    entry.capacity_index,
                ),
                character_id: self.actor.get(),
                inventory_item_id: self.inventory.get(),
                anchor_kind: EquipmentAnchorKind::ItemAttachment,
                location: None,
                parent_inventory_item_id: Some(entry.parent.get()),
                attachment_point_id: Some(entry.point_id),
                channel: entry.requirement.channel,
                order: entry.requirement.order,
                requirement_index: entry.requirement_index,
                capacity_index: entry.capacity_index,
            });
        }
    }
}

fn conflicting_roots(
    ctx: &ReducerContext,
    change: &EquipmentPlacementChange,
    definition: &crate::Item,
    placement: &crate::item::PersistedEquipmentPlacement,
) -> Result<BTreeSet<InventoryRowId>> {
    let conflicts =
        conflicting_equipment_roots(ctx, change.actor.get(), change.inventory.get(), placement)?;
    if change.conflicts == EquipmentConflictPolicy::RejectOccupied && !conflicts.is_empty() {
        let details = conflicts
            .iter()
            .map(|(requirement, _)| {
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
    conflicts
        .into_iter()
        .map(|(_, row)| InventoryRowId::try_new(row).map_err(|error| error.to_string()))
        .collect()
}
