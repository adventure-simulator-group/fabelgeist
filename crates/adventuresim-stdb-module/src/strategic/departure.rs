//! Revalidate strategic departure after synchronizing participant clocks.

use super::*;
use adventuresim_core::identity::{CharacterId, PartyId};

mod error;
mod policy;
pub(crate) use error::DepartureRevalidationError;
pub(super) use policy::{DepartureLocation, DepartureReadinessRule};
use policy::{
    DepartureReadinessRequirement, DepartureStop, PartyDepartureSnapshot, PendingDepartureIncidents,
};

pub(super) fn revalidate_party_after_departure_sync(
    ctx: &ReducerContext,
    party_id: &PartyId,
    leader_id: CharacterId,
    expected_location: &DepartureLocation,
    readiness: DepartureReadinessRule,
) -> Result<Party, DepartureRevalidationError> {
    let party = ctx
        .db
        .party_authority()
        .id()
        .find(party_id.as_str().to_owned())
        .ok_or(DepartureRevalidationError::PartyChanged)?;
    let snapshot = PartyDepartureSnapshot::new(
        CharacterId::from(party.leader_id),
        DepartureLocation::from_storage(
            party.current_settlement_id.clone(),
            party.current_case_site_id.clone(),
        )?,
        if party.camp_destination.is_some() {
            DepartureStop::Camped
        } else {
            DepartureStop::AtLocation
        },
    );
    let pending_incidents = PendingDepartureIncidents::new(
        ctx.db
            .strategic_incident()
            .party_id()
            .filter(party_id.as_str())
            .filter(|incident| incident.status == IncidentStatus::Pending)
            .map(|incident| incident.case_site_id),
    );
    snapshot.admit(leader_id, expected_location, &pending_incidents)?;
    let members = living_party_member_ids(ctx, party_id.as_str());
    expected_location.admit_members(members.iter().map(|id| {
        ctx.db
            .character()
            .id()
            .find(u64::from(*id))
            .map(|member| {
                if member.current_settlement_id.as_deref()
                    != expected_location
                        .settlement()
                        .map(adventuresim_core::identity::SettlementId::as_str)
                {
                    return Ok(None);
                }
                let case_site = crate::investigation::character_case_site_id(ctx, *id)
                    .map(CaseSiteId::try_new)
                    .transpose()?;
                Ok(Some(DepartureLocation::from_storage(
                    member.current_settlement_id,
                    case_site,
                )?))
            })
            .unwrap_or(Ok(None))
    }))?;
    if readiness.requirement(expected_location) == DepartureReadinessRequirement::ReadyParty {
        require_party_ready(ctx, party_id.as_str())?;
    }
    Ok(party)
}
