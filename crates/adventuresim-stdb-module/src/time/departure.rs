//! Establish the canonical journey clock without changing subjective age.

use super::*;
use adventuresim_core::identity::PartyId;

mod error;
pub(crate) use error::DepartureClockError;

/// Establish a journey-local clock without changing any participant's
/// subjective age. The leader-selected time of day is placed on the canonical
/// departure day; elapsed journey progress later wraps within that frozen day.
pub(crate) fn synchronize_party_departure_time(
    ctx: &ReducerContext,
    member_ids: &[adventuresim_core::identity::CharacterId],
) -> Result<StrategicMinute, DepartureClockError> {
    if member_ids.is_empty() {
        return Err(DepartureClockError::NoLivingMembers);
    }
    for member_id in member_ids {
        initialize_character_time(ctx, *member_id)?;
    }
    let party_id = member_ids
        .iter()
        .find_map(|member_id| {
            ctx.db
                .character()
                .id()
                .find(u64::from(*member_id))
                .and_then(|character| character.party_id)
        })
        .ok_or_else(|| DepartureClockError::NoParty {
            members: member_ids.to_vec(),
        })?;
    let party_id = PartyId::try_new(party_id).map_err(DepartureClockError::PartyIdentity)?;
    let mut party = ctx
        .db
        .party_authority()
        .id()
        .find(party_id.as_str().to_owned())
        .ok_or_else(|| DepartureClockError::PartyMissing {
            party: party_id.clone(),
        })?;
    if party.wilderness_canonical_anchor_minute.is_none() {
        party.wilderness_canonical_anchor_minute = Some(refresh_clock(ctx)?);
        party.wilderness_elapsed_minutes = 0;
        ctx.db.party_authority().id().update(party.clone());
    }
    let anchor = party
        .wilderness_canonical_anchor_minute
        .ok_or(DepartureClockError::Uninitialized { party: party_id })?;
    Ok(anchor.with_wrapped_time_of_day(
        party.journey_start_minute_of_day,
        party.wilderness_elapsed_minutes,
    ))
}
