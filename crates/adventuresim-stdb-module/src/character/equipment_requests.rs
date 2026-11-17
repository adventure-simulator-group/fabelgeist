//! Framework equipment requests share one checked effect across authorities.

use adventuresim_core::{
    equipment_placement::EquipmentHand,
    inventory_command::{EquipmentConflictPolicy, EquipmentPlacementIndex, InventoryRowId},
    physical_object::CustodyCharacterId,
};
use spacetimedb::{ReducerContext, SpacetimeType, reducer};

use super::{
    EquipmentAttachmentTargetSelection, EquipmentChange, EquipmentChangeSet, EquipmentChanges,
    EquipmentDestination, EquipmentEffectPolicy, EquipmentExchange,
};

#[derive(SpacetimeType, Clone, Debug)]
pub struct EquipmentChangeSetRequest {
    pub changes: EquipmentChangesRequest,
    pub conflicts: EquipmentConflictPolicy,
}

#[derive(SpacetimeType, Clone, Debug)]
pub struct EquipmentChangeRequest {
    pub inventory_item_id: u64,
    pub destination: EquipmentDestinationRequest,
}

#[derive(SpacetimeType, Clone, Debug)]
pub struct EquipmentExchangeRequest {
    pub source: EquipmentChangeRequest,
    pub destination: EquipmentChangeRequest,
}

#[derive(SpacetimeType, Clone, Debug)]
pub struct AuthoredEquipmentPlacementRequest {
    pub placement_index: u16,
    pub targets: Vec<EquipmentAttachmentTargetSelection>,
}

#[derive(SpacetimeType, Clone, Debug)]
pub enum EquipmentChangesRequest {
    Single(EquipmentChangeRequest),
    Exchange(EquipmentExchangeRequest),
}

#[derive(SpacetimeType, Clone, Debug)]
pub enum EquipmentDestinationRequest {
    Inventory,
    Held(EquipmentHand),
    Authored(AuthoredEquipmentPlacementRequest),
}

impl TryFrom<EquipmentChangesRequest> for EquipmentChanges {
    type Error = String;

    fn try_from(request: EquipmentChangesRequest) -> Result<Self, Self::Error> {
        Ok(match request {
            EquipmentChangesRequest::Single(change) => Self::Single(change.try_into()?),
            EquipmentChangesRequest::Exchange(exchange) => Self::Exchange(EquipmentExchange {
                source: exchange.source.try_into()?,
                destination: exchange.destination.try_into()?,
            }),
        })
    }
}

impl TryFrom<EquipmentChangeRequest> for EquipmentChange {
    type Error = String;

    fn try_from(request: EquipmentChangeRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            inventory: InventoryRowId::try_new(request.inventory_item_id)
                .map_err(|error| error.to_string())?,
            destination: request.destination.try_into()?,
        })
    }
}

impl TryFrom<EquipmentDestinationRequest> for EquipmentDestination {
    type Error = String;

    fn try_from(request: EquipmentDestinationRequest) -> Result<Self, Self::Error> {
        match request {
            EquipmentDestinationRequest::Inventory => Ok(Self::Inventory),
            EquipmentDestinationRequest::Held(hand) => Ok(Self::Held(hand)),
            EquipmentDestinationRequest::Authored(authored) => Self::authored(
                EquipmentPlacementIndex::new(authored.placement_index),
                authored.targets,
            ),
        }
    }
}

/// Abstract travel and camp retain account/gateway authority. Live actors must
/// use the registered host command, including its expected state and receipt.
#[reducer]
pub fn change_equipment(
    ctx: &ReducerContext,
    character_id: u64,
    request: EquipmentChangeSetRequest,
) -> Result<(), String> {
    crate::inventory_effects::require_non_live_actor(ctx, character_id)?;
    let actor = CustodyCharacterId::try_new(character_id).map_err(|error| error.to_string())?;
    EquipmentChangeSet::new(
        actor,
        request.changes.try_into()?,
        EquipmentEffectPolicy::Gameplay,
        request.conflicts,
    )
    .apply(ctx)
}
