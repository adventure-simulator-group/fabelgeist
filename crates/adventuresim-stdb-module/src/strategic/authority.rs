//! Admit the registered gateway or a disposable character's simulation owner.

use super::*;
use adventuresim_core::identity::CharacterId;
mod policy;
use policy::{CharacterAuthorityAdmission, RegisteredStrategicGateway};
pub(crate) use policy::{GatewayAdmissionError, StrategicCharacterAuthorityError};

pub(crate) fn require_strategic_gateway(
    ctx: &ReducerContext,
) -> Result<StrategicGatewayAuthority, GatewayAdmissionError> {
    let authority = ctx
        .db
        .strategic_gateway_authority()
        .id()
        .find(0)
        .ok_or(GatewayAdmissionError::Unregistered)?;
    RegisteredStrategicGateway::new(authority.identity).require_sender(ctx.sender())?;
    Ok(authority)
}

pub(crate) fn require_strategic_character_authority(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<(), StrategicCharacterAuthorityError> {
    let gateway = require_strategic_gateway(ctx).map(|_| ());
    CharacterAuthorityAdmission::new(character_id, gateway).require_with(|character_id| {
        crate::simulation::require_simulation_character_authority(ctx, character_id)
    })
}
