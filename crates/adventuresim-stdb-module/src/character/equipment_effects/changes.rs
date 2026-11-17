//! An explicit single change or two-item exchange commits as one transaction.

use adventuresim_core::{
    equipment_placement::EquipmentHand, inventory_command::EquipmentPlacementIndex,
    physical_object::CarriedInventoryScope,
};

use super::*;

pub(crate) struct EquipmentChange {
    pub(crate) inventory: InventoryRowId,
    pub(crate) destination: EquipmentDestination,
}

pub(crate) struct EquipmentExchange {
    pub(crate) source: EquipmentChange,
    pub(crate) destination: EquipmentChange,
}

pub(crate) struct AuthoredEquipmentDestination {
    placement: EquipmentPlacementIndex,
    targets: ParentSelections,
}

pub(crate) struct EquipmentChangeSet {
    actor: CustodyCharacterId,
    changes: EquipmentChanges,
    policy: EquipmentEffectPolicy,
    conflicts: EquipmentConflictPolicy,
}

struct PreparedChange {
    request: PlacementRequest,
    definition: crate::Item,
    destination: PreparedDestination,
}

struct DestinationPlan {
    destination: PreparedDestination,
    targets: ParentSelections,
}

pub(crate) enum EquipmentChanges {
    Single(EquipmentChange),
    Exchange(EquipmentExchange),
}

pub(crate) enum EquipmentDestination {
    Inventory,
    Held(EquipmentHand),
    Authored(AuthoredEquipmentDestination),
}

#[derive(Clone, Copy)]
pub(crate) enum EquipmentEffectPolicy {
    Gameplay,
    /// The caller finalizes derived state and compliance for its complete loadout.
    LoadoutAssembly,
}

enum PreparedDestination {
    Inventory,
    Placement(ResolvedEquipmentPlacement),
}

impl EquipmentDestination {
    /// Admit the framework's attachment selections before shared effect logic.
    pub(crate) fn authored(
        placement: EquipmentPlacementIndex,
        targets: Vec<EquipmentAttachmentTargetSelection>,
    ) -> Result<Self> {
        Ok(Self::Authored(AuthoredEquipmentDestination {
            placement,
            targets: ParentSelections::try_from(targets)?,
        }))
    }
}

impl EquipmentChangeSet {
    pub(crate) fn new(
        actor: CustodyCharacterId,
        changes: EquipmentChanges,
        policy: EquipmentEffectPolicy,
        conflicts: EquipmentConflictPolicy,
    ) -> Self {
        Self {
            actor,
            changes,
            policy,
            conflicts,
        }
    }

    pub(crate) fn apply(self, ctx: &ReducerContext) -> Result<()> {
        require_living_character(ctx, self.actor.get())?;
        let changes = self.changes.into_changes();
        let mut selected = BTreeSet::new();
        for change in &changes {
            if !selected.insert(change.inventory) {
                return Err("Equipment exchange selects the same inventory item twice".into());
            }
        }
        let mut prepared = changes
            .into_iter()
            .map(|change| PreparedChange::new(ctx, self.actor, change, self.policy, self.conflicts))
            .collect::<Result<Vec<_>>>()?;
        require_selected_children(ctx, self.actor, &selected)?;
        for change in &prepared {
            if matches!(change.destination, PreparedDestination::Placement(_)) {
                crate::inventory_container::detach_row_for_action(
                    ctx,
                    CarriedInventoryScope::Personal,
                    change.request.inventory.get(),
                )?;
            }
        }
        // Sources vacate together inside this reducer transaction. Any later
        // error rolls back their detachment and all reconstructed placements.
        for change in &prepared {
            unequip_wearable(ctx, change.request.inventory.get());
        }
        while !prepared.is_empty() {
            let ready = prepared
                .iter()
                .position(|change| {
                    change.request.targets.iter().all(|parent| {
                        !prepared
                            .iter()
                            .any(|pending| pending.request.inventory == parent.parent)
                    })
                })
                .ok_or("Equipment exchange would create an attachment cycle")?;
            let change = prepared.remove(ready);
            let placement = match change.destination {
                PreparedDestination::Inventory => continue,
                PreparedDestination::Placement(placement) => placement,
            };
            let plan =
                PlacementPlan::prepare(ctx, &change.request, &change.definition, &placement)?;
            if plan
                .displaced
                .iter()
                .any(|inventory| selected.contains(inventory))
            {
                return Err("Equipment exchange destinations conflict with each other".into());
            }
            plan.commit(ctx);
        }
        match self.policy {
            EquipmentEffectPolicy::Gameplay => refresh_equipment_dependents(ctx, self.actor.get()),
            EquipmentEffectPolicy::LoadoutAssembly => Ok(()),
        }
    }
}

impl EquipmentChanges {
    fn into_changes(self) -> Vec<EquipmentChange> {
        match self {
            Self::Single(change) => vec![change],
            Self::Exchange(exchange) => vec![exchange.source, exchange.destination],
        }
    }
}

impl PreparedChange {
    fn new(
        ctx: &ReducerContext,
        actor: CustodyCharacterId,
        change: EquipmentChange,
        policy: EquipmentEffectPolicy,
        conflicts: EquipmentConflictPolicy,
    ) -> Result<Self> {
        let inventory = ctx
            .db
            .inventory_item()
            .id()
            .find(change.inventory.get())
            .filter(|row| row.character_id == actor.get())
            .ok_or("Inventory item is not carried by this character")?;
        let definition = ctx
            .db
            .item()
            .id()
            .find(&inventory.item_id)
            .ok_or("Inventory item has no catalog definition")?;
        let equipped = ctx
            .db
            .character_equipped_item()
            .inventory_item_id()
            .find(change.inventory.get());
        if equipped
            .as_ref()
            .is_some_and(|row| row.character_id != actor.get())
        {
            return Err("Equipped item belongs to another character".into());
        }
        if matches!(&change.destination, EquipmentDestination::Inventory) && equipped.is_none() {
            return Err("Item is not equipped by this character".into());
        }
        let plan = DestinationPlan::new(&definition, change.destination)?;
        if matches!(policy, EquipmentEffectPolicy::Gameplay)
            && matches!(&plan.destination, PreparedDestination::Placement(_))
        {
            crate::equipment_law::require_item_legal(ctx, actor.get(), change.inventory.get())?;
        }
        Ok(Self {
            request: PlacementRequest {
                actor,
                inventory: change.inventory,
                targets: plan.targets,
                conflicts,
            },
            definition,
            destination: plan.destination,
        })
    }
}

impl DestinationPlan {
    fn new(definition: &crate::Item, destination: EquipmentDestination) -> Result<Self> {
        match destination {
            EquipmentDestination::Inventory => Ok(Self {
                destination: PreparedDestination::Inventory,
                targets: ParentSelections::empty(),
            }),
            EquipmentDestination::Held(hand) => Ok(Self {
                destination: PreparedDestination::Placement(ResolvedEquipmentPlacement::held(hand)),
                targets: ParentSelections::empty(),
            }),
            EquipmentDestination::Authored(authored) => {
                let placement = definition
                    .equipment_placements
                    .get(usize::from(authored.placement.get()))
                    .ok_or("Equipment placement is not authored for this item")?;
                if definition.kind == crate::item::CatalogItemKind::Weapon {
                    require_weapon_carry_placement(&definition.id, placement)?;
                }
                Ok(Self {
                    destination: PreparedDestination::Placement(
                        ResolvedEquipmentPlacement::authored(placement)?,
                    ),
                    targets: authored.targets,
                })
            }
        }
    }
}

fn require_selected_children(
    ctx: &ReducerContext,
    actor: CustodyCharacterId,
    selected: &BTreeSet<InventoryRowId>,
) -> Result<()> {
    for parent in selected {
        for occupancy in ctx
            .db
            .equipment_occupancy()
            .iter()
            .filter(|row| row.parent_inventory_item_id == Some(parent.get()))
        {
            let child = InventoryRowId::try_new(occupancy.inventory_item_id)
                .map_err(|error| error.to_string())?;
            if occupancy.character_id != actor.get() || !selected.contains(&child) {
                return Err("Detach or remove attached/contained items first".into());
            }
        }
    }
    Ok(())
}
