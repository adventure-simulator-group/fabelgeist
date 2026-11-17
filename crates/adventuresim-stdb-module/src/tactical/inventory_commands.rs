//! Atomic registered-host inventory commits and immutable successful receipts.

use adventuresim_core::{
    inventory_command::{
        EquipmentConflictPolicy, EquipmentPlacementIndex, InventoryCommandId, InventoryRevision,
        InventoryRowId, InventoryScopeId, InventorySnapshotToken,
    },
    inventory_measurement::ItemQuantity,
    material::Milliliters,
    physical_object::{CustodyCharacterId, InventoryLocation, PhysicalObjectId},
};
use spacetimedb::sats::bsatn;

use super::inventory::{InventoryAdmission, InventorySnapshot};
use super::*;
use request::InventoryCommand;

mod authority;
mod request;

type CommandResult<T> = std::result::Result<T, String>;

#[derive(SpacetimeType, Clone, Debug)]
pub struct TacticalInventoryActorState {
    pub character_id: u64,
    pub scope: Vec<u8>,
    pub revision: Vec<u8>,
}

#[derive(SpacetimeType, Clone, Debug)]
pub struct TacticalInventoryCommand {
    pub command_id: String,
    pub actor: TacticalInventoryActorState,
    pub action: TacticalInventoryAction,
}

#[derive(SpacetimeType, Clone, Debug)]
pub enum TacticalInventoryAction {
    PlaceEquipment(TacticalInventoryEquipmentPlacement),
    RemoveEquipment(u64),
    Transfer(TacticalInventoryTransfer),
    InsertIntoContainer(TacticalInventoryContainerInsertion),
    RemoveFromContainer(u64),
    DiscardWater(TacticalInventoryWaterDisposal),
}

#[derive(SpacetimeType, Clone, Debug)]
pub struct TacticalInventoryEquipmentPlacement {
    pub inventory_item_id: u64,
    pub placement_index: u16,
    pub targets: Vec<crate::character::EquipmentAttachmentTargetSelection>,
    pub conflicts: EquipmentConflictPolicy,
}

#[derive(SpacetimeType, Clone, Debug)]
pub struct TacticalInventoryTransfer {
    pub inventory_item_id: u64,
    pub quantity: u32,
    pub recipient: TacticalInventoryActorState,
}

#[derive(SpacetimeType, Clone, Debug)]
pub struct TacticalInventoryContainerInsertion {
    pub child_object_id: u64,
    pub parent_object_id: u64,
}

#[derive(SpacetimeType, Clone, Debug)]
pub struct TacticalInventoryWaterDisposal {
    pub container_object_id: u64,
    pub amount_ml: u64,
}

#[derive(Clone, Debug)]
#[table(accessor = tactical_inventory_receipt_authority)]
pub struct TacticalInventoryReceipt {
    #[primary_key]
    pub id: String,
    #[index(btree)]
    pub server: Identity,
    pub hosting_connection: spacetimedb::ConnectionId,
    pub actor_character_id: u64,
    pub command_id: String,
    pub request_digest: Vec<u8>,
    pub participants: Vec<TacticalInventoryActorState>,
}

struct PayloadDigest([u8; 32]);

struct ReceiptKey(String);

impl PayloadDigest {
    fn for_request(request: &TacticalInventoryCommand) -> CommandResult<Self> {
        let bytes = bsatn::to_vec(request).map_err(|_| "Inventory command cannot be encoded")?;
        Ok(Self(Sha256::digest(bytes).into()))
    }
}

impl ReceiptKey {
    fn for_command(scope: InventoryScopeId, id: &InventoryCommandId) -> Self {
        let prefix: String = scope
            .bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        Self(format!("{prefix}:{}", id.as_str()))
    }
}

/// An error aborts the entire reducer transaction; rejected effects are never
/// caught and recorded after partial writes. Only successful commits get receipts.
#[reducer]
pub fn commit_tactical_inventory(
    ctx: &ReducerContext,
    request: TacticalInventoryCommand,
) -> Result<(), String> {
    let server = authority::require_host(ctx)?;
    let command = InventoryCommand::try_from(request)?;
    command.require_admissions(ctx)?;
    let key = ReceiptKey::for_command(command.actor.token.scope, &command.id);
    if let Some(receipt) = ctx
        .db
        .tactical_inventory_receipt_authority()
        .id()
        .find(&key.0)
    {
        if receipt.request_digest != command.digest.0 {
            return Err(
                "Inventory command identity was already committed with another payload".into(),
            );
        }
        return Ok(());
    }
    command.require_current(ctx)?;
    command.effect.apply(ctx, command.actor.id)?;
    let participants = command
        .actors()
        .map(|actor| actor.result(ctx))
        .collect::<CommandResult<Vec<_>>>()?;
    ctx.db
        .tactical_inventory_receipt_authority()
        .insert(TacticalInventoryReceipt {
            id: key.0,
            server: server.identity,
            hosting_connection: server.hosting_connection,
            actor_character_id: command.actor.id.get(),
            command_id: command.id.into(),
            request_digest: command.digest.0.to_vec(),
            participants,
        });
    Ok(())
}

/// Receipts belong to the currently registered serving connection. A retired
/// host sees no results, even if it keeps its former identity credential.
#[view(accessor = tactical_inventory_receipt, public)]
pub fn tactical_inventory_receipt(ctx: &ViewContext) -> Vec<TacticalInventoryReceipt> {
    let Some(server) = ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(ctx.sender())
    else {
        return Vec::new();
    };
    ctx.db
        .tactical_inventory_receipt_authority()
        .server()
        .filter(ctx.sender())
        .filter(|receipt| receipt.hosting_connection == server.hosting_connection)
        .collect()
}

#[cfg(feature = "authority-tests")]
mod authority_tests;
