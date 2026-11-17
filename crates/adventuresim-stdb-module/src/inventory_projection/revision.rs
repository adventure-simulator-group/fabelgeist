//! Hash all durable command inputs separately from the actor's admission scope.

use std::collections::BTreeSet;

use adventuresim_core::inventory_command::{InventoryRevision, InventoryScopeId, InventorySnapshotToken};
use sha2::{Digest, Sha256};
use spacetimedb::{SpacetimeType, ViewContext, sats::{bsatn, ser::Serialize}};

use super::{InventoryProjectionError, footprint::InventoryFootprint};
use crate::{
    InventoryItem,
    character::{CharacterEquippedItem, EquipmentOccupancy, character_equipped_item__view, equipment_occupancy__view},
    food::{FoodLot, food_lot__view},
    inventory_amount::{InventoryItemAmount, inventory_item_amount__view},
    item::inventory_item__view,
    repair::{ItemCondition, item_condition__view},
    tactical::{ConnectedPlayerItem, TacticalServer, local_time::TacticalActorTime},
    weapon_instance::{WeaponHolderInstance, WeaponInstance, weapon_holder_instance__view, weapon_instance__view},
};

#[derive(SpacetimeType)]
struct InventoryScopeInput {
    server: spacetimedb::Identity,
    connection: spacetimedb::ConnectionId,
    instance_id: String,
    character_id: u64,
    time: TacticalActorTime,
}

#[derive(SpacetimeType)]
struct InventoryRevisionInput {
    rows: Vec<InventoryItem>,
    items: Vec<ConnectedPlayerItem>,
    footprint: InventoryFootprint,
    amounts: Vec<InventoryItemAmount>,
    food: Vec<FoodLot>,
    equipment: Vec<CharacterEquippedItem>,
    occupancies: Vec<EquipmentOccupancy>,
    condition: Vec<ItemCondition>,
    weapons: Vec<WeaponInstance>,
    holders: Vec<WeaponHolderInstance>,
}

type ProjectionResult<T> = std::result::Result<T, InventoryProjectionError>;

impl InventoryRevisionInput {
    fn capture(ctx: &ViewContext, character_id: u64, items: &[ConnectedPlayerItem]) -> ProjectionResult<Self> {
        let mut rows: Vec<_> = ctx.db.inventory_item().character_id().filter(character_id).collect();
        rows.sort_by_key(|row| row.id);
        let footprint = InventoryFootprint::capture(ctx, character_id, &rows)?;
        let ids: BTreeSet<_> = rows.iter().map(|row| row.id).collect();
        let amounts = rows.iter().filter_map(|row| {
            ctx.db.inventory_item_amount().inventory_item_id().find(row.id)
        }).collect();
        let mut food: Vec<_> = ctx.db.food_lot().material_revision().filter(0u64..)
            .filter(|lot| lot.inventory_item_id.is_some_and(|id| ids.contains(&id))).collect();
        food.sort_by_key(|lot| lot.id);
        let equipment: Vec<_> = rows.iter().filter_map(|row| {
            ctx.db.character_equipped_item().inventory_item_id().find(row.id)
        }).collect();
        let mut occupancies: Vec<_> = rows.iter().flat_map(|row| {
            ctx.db.equipment_occupancy().inventory_item_id().filter(row.id)
        }).collect();
        occupancies.sort_by(|first, second| first.id.cmp(&second.id));
        if equipment.iter().any(|worn| worn.character_id != character_id)
            || occupancies.iter().any(|occupancy| occupancy.character_id != character_id)
        {
            return Err(InventoryProjectionError::InvalidIdentity);
        }
        let condition = rows.iter().filter_map(|row| {
            ctx.db.item_condition().inventory_item_id().find(row.id)
        }).collect();
        let weapons = footprint.objects.iter().filter_map(|object| {
            ctx.db.weapon_instance().physical_object_id().find(object.id)
        }).collect();
        let holders = footprint.objects.iter().filter_map(|object| {
            ctx.db.weapon_holder_instance().physical_object_id().find(object.id)
        }).collect();
        Ok(Self { rows, items: items.to_vec(), footprint, amounts, food, equipment, occupancies, condition, weapons, holders })
    }
}

pub(crate) fn snapshot_token(
    ctx: &ViewContext,
    server: &TacticalServer,
    character_id: u64,
    time: &TacticalActorTime,
    items: &[ConnectedPlayerItem],
) -> ProjectionResult<InventorySnapshotToken> {
    let scope = InventoryScopeInput {
        server: server.identity,
        connection: server.hosting_connection,
        instance_id: server.instance_id.clone(),
        character_id,
        time: time.clone(),
    };
    let state = InventoryRevisionInput::capture(ctx, character_id, items)?;
    Ok(InventorySnapshotToken {
        scope: InventoryScopeId::from_sha256(hash(&scope)?),
        revision: InventoryRevision::from_sha256(hash(&state)?),
    })
}

fn hash(value: &impl Serialize) -> ProjectionResult<[u8; 32]> {
    let bytes = bsatn::to_vec(value).map_err(|_| InventoryProjectionError::InvalidEncoding)?;
    Ok(Sha256::digest(bytes).into())
}
