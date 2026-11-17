//! Publish validated rows without replacing bodies or surviving item entities.

use std::collections::{BTreeMap, BTreeSet};

use adventuresim_core::{
    inventory_command::InventorySnapshotToken, physical_object::PhysicalObjectId,
};
use adventuresim_stdb_client::{
    CatalogItemKind, ConnectedInventory, ConnectedPlayerItem, TacticalActorTime,
};
use adventuresim_tactical_core::{inventory::ItemProperties, prelude::*};
use adventuresim_tactical_netcode::bevy_replicon::prelude::Replicated;
use bevy::prelude::*;

use super::prepare::{PreparedAnchor, PreparedInventoryItem};
use super::{InventoryAdmissionError, InventoryAdmissionResult, PreparedInventory};

#[derive(Component, Clone, Copy)]
pub(super) struct DurableInventoryObject(PhysicalObjectId);

#[derive(Resource, Default)]
pub(super) struct AppliedInventories(pub(super) BTreeMap<CharacterId, AppliedInventory>);

pub(super) struct AppliedInventory {
    pub(super) time: TacticalActorTime,
    pub(super) token: InventorySnapshotToken,
    items: Vec<ConnectedPlayerItem>,
}

pub(super) struct ActorInventory {
    pub(super) owner: Entity,
    pub(super) character: CharacterId,
    pub(super) time: TacticalActorTime,
    pub(super) prepared: PreparedInventory,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum InventoryEntityIdentity {
    Physical(PhysicalObjectId),
    Row(TacticalInventoryItemId),
}

impl PreparedInventoryItem {
    fn identity(&self) -> InventoryEntityIdentity {
        self.object_id.map_or(
            InventoryEntityIdentity::Row(self.row_id),
            InventoryEntityIdentity::Physical,
        )
    }
}

impl AppliedInventory {
    pub(super) fn matches(&self, time: &TacticalActorTime, inventory: &ConnectedInventory) -> bool {
        let ConnectedInventory::Available(snapshot) = inventory else {
            return false;
        };
        self.time == *time
            && self.token.scope.bytes().as_slice() == snapshot.scope
            && self.token.revision.bytes().as_slice() == snapshot.revision
            && self.items == snapshot.items
    }
}

pub(super) fn apply_batch(
    world: &mut World,
    actors: Vec<ActorInventory>,
) -> InventoryAdmissionResult<()> {
    PreparedInventory::validate_disjoint(actors.iter().map(|actor| &actor.prepared))?;
    let mut desired = BTreeSet::new();
    for actor in &actors {
        for item in &actor.prepared.items {
            if !desired.insert(item.identity()) {
                return Err(InventoryAdmissionError::InvalidObjectIdentity);
            }
        }
    }
    let mut entities = BTreeMap::new();
    let mut affected: BTreeSet<_> = actors.iter().map(|actor| actor.owner).collect();
    let refreshed: Vec<_> = actors
        .iter()
        .filter(|actor| {
            world
                .get_resource::<AppliedInventories>()
                .is_some_and(|applied| applied.0.contains_key(&actor.character))
        })
        .map(|actor| actor.owner)
        .collect();
    let mut rows = world.query::<(
        Entity,
        &TacticalInventoryItemId,
        Option<&DurableInventoryObject>,
        Option<&ItemOf>,
    )>();
    let mut old = Vec::new();
    for (entity, row, physical, owner) in rows.iter(world) {
        let identity = physical.map_or(InventoryEntityIdentity::Row(*row), |physical| {
            InventoryEntityIdentity::Physical(physical.0)
        });
        if entities.insert(identity, entity).is_some() {
            return Err(InventoryAdmissionError::InvalidObjectIdentity);
        }
        if owner.is_some_and(|owner| actors.iter().any(|actor| actor.owner == owner.0)) {
            old.push((identity, entity));
        }
        if desired.contains(&identity)
            && let Some(owner) = owner
        {
            affected.insert(owner.0);
        }
    }
    // Resolve the whole cross-owner batch before removing old rows. Rehomed
    // physical objects retain their entity even when their backing row changes.
    for identity in &desired {
        entities
            .entry(*identity)
            .or_insert_with(|| world.spawn(Transform::default()).id());
    }
    for actor in actors {
        let items = actor
            .prepared
            .items
            .iter()
            .map(|item| item.source.clone())
            .collect();
        let item_entities: BTreeMap<_, _> = actor
            .prepared
            .items
            .iter()
            .map(|item| (item.row_id, entities[&item.identity()]))
            .collect();
        for item in actor.prepared.items {
            let entity = entities[&item.identity()];
            project_item(world, actor.owner, entity, item, &item_entities);
        }
        world.init_resource::<AppliedInventories>();
        world.resource_mut::<AppliedInventories>().0.insert(
            actor.character,
            AppliedInventory {
                time: actor.time,
                token: actor.prepared.token,
                items,
            },
        );
    }
    for (identity, entity) in old {
        if !desired.contains(&identity) {
            world.despawn(entity);
        }
    }
    for owner in affected {
        rebuild_inventory_holding_cache(world, owner);
    }
    for owner in refreshed {
        invalidate_equipment_revision(world, owner);
    }
    Ok(())
}

fn invalidate_equipment_revision(world: &mut World, owner: Entity) {
    if let Some(mut state) = world.get_mut::<EquipmentActionState>(owner) {
        state.revision = state.revision.next();
    }
    if let Some(mut retained) = world.get_mut::<super::super::DisconnectedProjection>(owner)
        && let super::super::DisconnectedProjection::Projected(snapshot) = &mut *retained
    {
        snapshot.equipment_action_state.revision = snapshot.equipment_action_state.revision.next();
    }
}

fn project_item(
    world: &mut World,
    owner: Entity,
    entity: Entity,
    item: PreparedInventoryItem,
    entities: &BTreeMap<TacticalInventoryItemId, Entity>,
) {
    let topology = EquipmentTopology {
        placement_id: item.source.selected_placement_id.clone(),
        occupancies: item
            .occupancies
            .iter()
            .enumerate()
            .map(|(index, occupancy)| EquipmentTopologyOccupancy {
                occupancy_id: format!("tactical:{}:{index}", entity.to_bits()),
                anchor: match &occupancy.anchor {
                    PreparedAnchor::CharacterLocation(location) => {
                        TacticalEquipmentAnchor::CharacterLocation(*location)
                    }
                    PreparedAnchor::ItemAttachment {
                        parent_row_id,
                        attachment_point_id,
                    } => TacticalEquipmentAnchor::ItemAttachment {
                        parent: entities[parent_row_id],
                        attachment_point_id: attachment_point_id.clone(),
                    },
                },
                channel: super::super::tactical_equipment_channel(occupancy.source.channel),
                order: occupancy.source.order,
                requirement_index: occupancy.source.requirement_index,
                capacity_index: occupancy.source.capacity_index,
            })
            .collect(),
    };
    let mut target = world.entity_mut(entity);
    target.remove::<(
        EquipSlot,
        WeaponItem,
        ShieldItem,
        ArmorItem,
        TacticalEquipmentPhysical,
        WeaponAppearance,
        WeaponHolderAppearance,
        DurableInventoryObject,
    )>();
    target.insert((
        Replicated,
        item.row_id,
        ItemOf(owner),
        item.quantity,
        ItemProperties {
            weight: item.unit_mass.kilograms(),
            id: item.source.item.id.clone(),
        },
        TacticalInventoryMass::Measured(item.carried_mass),
        topology,
    ));
    if let Some(object_id) = item.object_id {
        target.insert(DurableInventoryObject(object_id));
    }
    if let Some(physical) = item.physical {
        target.insert(physical);
    }
    if let Some(weapon) = item.weapon {
        target.insert(weapon);
    }
    if let Some(armor) = item.armor {
        target.insert(armor);
    }
    if let Some(appearance) = item.weapon_appearance {
        target.insert(appearance);
    }
    if let Some(appearance) = item.holder_appearance {
        target.insert(appearance);
    }
    if item.source.item.kind == CatalogItemKind::Shield {
        target.insert(ShieldItem {
            block: item.source.item.block,
        });
    }
    let held =
        item.source.occupancies.iter().find(|occupancy| {
            occupancy.channel == adventuresim_stdb_client::EquipmentChannel::Held
        });
    match held.and_then(|occupancy| occupancy.location) {
        Some(adventuresim_stdb_client::EquipmentLocation::LeftHand) => {
            target.insert(EquipSlot::HoldingLeft);
        }
        Some(adventuresim_stdb_client::EquipmentLocation::RightHand) => {
            target.insert(EquipSlot::HoldingRight);
        }
        _ => {}
    }
}
