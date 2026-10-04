//! Read both encounter authorities before admitting strategic activity.

use super::*;
use adventuresim_core::identity::{CharacterId, PartyId};
mod policy;
pub(crate) use policy::PendingEncounterError;
use policy::{BoundRoadChallengeState, EncounterChoiceState, PartyEncounterAdmission};

pub(super) fn unresolved_encounter(
    ctx: &ReducerContext,
    party_id: &PartyId,
) -> Option<StrategicEncounter> {
    ctx.db
        .strategic_encounter()
        .party_id()
        .find(party_id.as_str().to_owned())
        .filter(|encounter| encounter.status == StrategicEncounterStatus::AwaitingChoice)
}

pub(crate) fn require_no_unresolved_encounter(
    ctx: &ReducerContext,
    party_id: &PartyId,
) -> Result<(), PendingEncounterError> {
    let party = ctx
        .db
        .party_authority()
        .id()
        .find(party_id.as_str().to_owned());
    let narrative_pending = party.as_ref().is_some_and(|party| {
        ctx.db
            .road_challenge_authority()
            .party_id()
            .filter(&party_id.as_str().to_owned())
            .any(|occurrence| {
                occurrence.is_open() && party_at_bound_road_challenge(ctx, party, &occurrence)
            })
    });
    let choice = if unresolved_encounter(ctx, party_id).is_some() {
        EncounterChoiceState::AwaitingChoice
    } else {
        EncounterChoiceState::Clear
    };
    let road_challenge = if narrative_pending {
        BoundRoadChallengeState::Open
    } else {
        BoundRoadChallengeState::Clear
    };
    PartyEncounterAdmission::new(party_id.clone(), choice, road_challenge).require_clear()
}

pub(crate) fn require_character_no_unresolved_encounter(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<(), PendingEncounterError> {
    if let Some(party_key) = ctx
        .db
        .character()
        .id()
        .find(u64::from(character_id))
        .and_then(|character| character.party_id)
    {
        let party_id = PartyId::try_new(party_key)?;
        require_no_unresolved_encounter(ctx, &party_id)?;
    }
    Ok(())
}
