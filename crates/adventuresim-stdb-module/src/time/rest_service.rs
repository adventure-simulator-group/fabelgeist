//! Admit ordinary public rest from the character's current settlement.

mod availability;
pub(super) mod error;
use availability::require_settlement_rest_service;
use error::RestServiceAdmissionError;

use crate::{character::require_living_character, strategic::settlement};
use adventuresim_core::identity::CharacterId;
use adventuresim_world_schema::SettlementActionService;
use spacetimedb::ReducerContext;

pub(super) fn require_character_rest_service(
    ctx: &ReducerContext,
    character_id: CharacterId,
    service: SettlementActionService,
) -> Result<(), RestServiceAdmissionError> {
    let character = require_living_character(ctx, character_id)?;
    let settlement_id = character.current_settlement_id.as_deref().ok_or(
        RestServiceAdmissionError::NotAtSettlement {
            character: character_id,
        },
    )?;
    let settlement = ctx
        .db
        .settlement()
        .id()
        .find(settlement_id.to_owned())
        .ok_or(RestServiceAdmissionError::MissingSettlement {
            character: character_id,
        })?;
    require_settlement_rest_service(&settlement.economy, service)
}
