//! Validate durable inventory before publishing initial or refreshed entities.

use adventuresim_stdb_client::InventoryProjectionError;
use adventuresim_stdb_client::TacticalActorTime;
use adventuresim_tactical_core::prelude::CharacterId;
use bevy::prelude::*;

pub(crate) use apply::AppliedInventories;
pub(super) use prepare::PreparedInventory;
pub(crate) use refresh::refresh_connected_inventory;

mod apply;
mod prepare;
mod refresh;

pub(super) type InventoryAdmissionResult<T> = Result<T, InventoryAdmissionError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum InventoryAdmissionError {
    #[error("inventory snapshot has an invalid admission or state token")]
    InvalidSnapshotToken,
    #[error("multiple tactical bodies claim the same inventory owner")]
    AmbiguousActor,
    #[error("durable inventory projection is unavailable: {0:?}")]
    Unavailable(InventoryProjectionError),
    #[error("inventory contains an invalid or duplicate row identity")]
    InvalidRowIdentity,
    #[error("inventory contains an invalid or duplicate physical object identity")]
    InvalidObjectIdentity,
    #[error("inventory quantity must be nonzero")]
    InvalidQuantity,
    #[error("inventory mass must be finite and nonnegative")]
    InvalidMass,
    #[error("inventory equipment metadata is invalid")]
    InvalidEquipment,
    #[error("inventory attachment topology is invalid")]
    InvalidTopology,
    #[error("inventory recipe does not authenticate its catalog construction")]
    InvalidRecipe,
}

pub(super) fn apply_initial(
    world: &mut World,
    owner: Entity,
    character: CharacterId,
    time: TacticalActorTime,
    prepared: PreparedInventory,
) -> Result {
    apply::apply_batch(
        world,
        vec![apply::ActorInventory {
            owner,
            character,
            time,
            prepared,
        }],
    )?;
    Ok(())
}

#[cfg(test)]
mod behavior;
