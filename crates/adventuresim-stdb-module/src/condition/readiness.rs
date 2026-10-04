//! Authoritative life, condition and holy-day checks for action admission.

use super::{
    CharacterReadinessError, StrategicConditionError, ensure_holy_day_demand,
    projection::{
        refresh_character_strategic_condition, refresh_party_strategic_condition_projection,
    },
    readiness_error::ReadinessParticipant,
};
use crate::character::{LivingCharacterError, require_living_character};
use adventuresim_core::identity::CharacterId;
use spacetimedb::ReducerContext;

pub(crate) fn require_character_ready(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<(), CharacterReadinessError> {
    let participant = ReadinessParticipant::Individual(character_id);
    require_living_character(ctx, character_id).map_err(|source: LivingCharacterError| {
        CharacterReadinessError::Living {
            participant,
            source,
        }
    })?;
    let condition = refresh_character_strategic_condition(ctx, character_id).map_err(
        |source: StrategicConditionError| CharacterReadinessError::CharacterProjection {
            character: character_id,
            source,
        },
    )?;
    participant.require_status(condition.status)
}

pub(crate) fn require_characters_ready(
    ctx: &ReducerContext,
    character_ids: &[CharacterId],
) -> Result<(), CharacterReadinessError> {
    for character_id in character_ids {
        require_living_character(ctx, *character_id).map_err(|source: LivingCharacterError| {
            CharacterReadinessError::Living {
                participant: ReadinessParticipant::PartyMember(*character_id),
                source,
            }
        })?;
    }
    let conditions = refresh_party_strategic_condition_projection(ctx, character_ids).map_err(
        |source: StrategicConditionError| CharacterReadinessError::PartyProjection {
            members: character_ids.to_vec(),
            source,
        },
    )?;
    for condition in &conditions {
        ensure_holy_day_demand(ctx, condition).map_err(|source: StrategicConditionError| {
            CharacterReadinessError::PartyProjection {
                members: character_ids.to_vec(),
                source,
            }
        })?;
        ReadinessParticipant::PartyMember(condition.character_id.into())
            .require_status(condition.status)?;
    }
    Ok(())
}
