//! Read run ownership before the disposable character membership index.

use super::*;
use adventuresim_core::identity::CharacterId;
mod policy;
pub(crate) use policy::SimulationCharacterAuthorityError;
use policy::{DisposableSimulationOwner, SimulationCharacterMembership};

pub(crate) fn require_simulation_character_authority(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<(), SimulationCharacterAuthorityError> {
    let run = ctx.db.simulation_run().id().find(0).ok_or(
        SimulationCharacterAuthorityError::Unclaimed {
            character: character_id,
        },
    )?;
    DisposableSimulationOwner::new(run.owner).require_character(
        character_id,
        ctx.sender(),
        |character_id| {
            if ctx
                .db
                .simulation_character()
                .character_id()
                .find(u64::from(character_id))
                .is_some()
            {
                SimulationCharacterMembership::Member
            } else {
                SimulationCharacterMembership::OutsideRun
            }
        },
    )
}
