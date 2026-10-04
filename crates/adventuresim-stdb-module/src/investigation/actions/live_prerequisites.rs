//! Validate live party, journal and lead prerequisites before action commit.

use super::*;

pub(super) fn validate_live_action_prerequisites(
    ctx: &ReducerContext,
    actor: &crate::Character,
    party_id: &adventuresim_core::identity::PartyId,
    capability: &InvestigationActionCapability,
    kind: action::InvestigationActionKind,
) -> Result<Vec<adventuresim_core::identity::CharacterId>, InvestigationAdmissionError> {
    if !tracking_capability_chain_is_coherent(
        capability,
        kind,
        |id| {
            ctx.db
                .investigation_action_capability()
                .id()
                .find(id.to_owned())
        },
        |id| {
            ctx.db
                .investigation_action_attempt()
                .capability_id()
                .filter(id)
                .any(|attempt| attempt.success)
        },
    ) {
        return Err(InvestigationAdmissionError::InvalidRoute);
    }
    if !capability_has_live_support_reducer(ctx, capability, kind) {
        return Err(InvestigationAdmissionError::UnsupportedJournalRoute);
    }
    require_party_ready(ctx, party_id.as_str())?;
    require_no_unresolved_encounter(ctx, party_id)?;
    let party = ctx
        .db
        .party_authority()
        .id()
        .find(party_id.as_str().to_owned())
        .ok_or(InvestigationAdmissionError::PartyMissing)?;
    if party.camp_destination.is_some()
        || party.camp_remaining_minutes > 0
        || ctx
            .db
            .party_journey_authority()
            .party_id()
            .find(party_id.as_str().to_owned())
            .is_some()
    {
        return Err(InvestigationAdmissionError::JourneyOrCamp);
    }
    let members = living_party_member_ids(ctx, party_id.as_str());
    if members.len() < usize::from(action::prerequisites(kind).minimum_party_members) {
        return Err(InvestigationAdmissionError::InsufficientMembers {
            members: members.clone(),
            kind,
        });
    }
    let actor_site = character_case_site_id(ctx, (actor.id).into());
    for member_id in &members {
        let member = ctx
            .db
            .character()
            .id()
            .find(u64::from(*member_id))
            .ok_or(InvestigationAdmissionError::MemberMissing { member: *member_id })?;
        if member.current_settlement_id != actor.current_settlement_id
            || character_case_site_id(ctx, (*member_id).into()) != actor_site
        {
            return Err(InvestigationAdmissionError::MembersSeparated { member: *member_id });
        }
    }
    if !capability.required_action_id.is_empty() {
        let predecessor = ctx
            .db
            .investigation_action_capability()
            .id()
            .find(&capability.required_action_id)
            .ok_or(InvestigationAdmissionError::PredecessorMissing)?;
        if predecessor.owner_character_id != capability.owner_character_id
            || predecessor.case_id != capability.case_id
            || !ctx
                .db
                .investigation_action_attempt()
                .capability_id()
                .filter(&predecessor.id)
                .any(|attempt| attempt.success)
        {
            return Err(InvestigationAdmissionError::PredecessorIncomplete);
        }
    }
    let prereqs = action::prerequisites(kind);
    let observer_case_id = reducer_action_public_case_id(ctx, capability)
        .ok_or(InvestigationAdmissionError::ObserverCaseMissing)?;
    if prereqs.requires_contact_referral
        && !ctx
            .db
            .investigation_lead()
            .owner_character_id()
            .filter(actor.id)
            .any(|lead| lead_is_live_contact_referral(&lead, actor.id, &observer_case_id))
    {
        return Err(InvestigationAdmissionError::NoLiveContactReferral);
    }
    if prereqs.requires_approximate_destination
        && capability.target_kind != action::InvestigationTargetKind::Area
        && !ctx
            .db
            .investigation_lead()
            .owner_character_id()
            .filter(actor.id)
            .any(|lead| {
                lead.case_id == observer_case_id
                    && lead.destination_stage == DestinationKnowledgeStage::ApproximateArea
                    && lead.corrected_by.is_empty()
            })
    {
        return Err(InvestigationAdmissionError::NoApproximateDestination);
    }
    if prereqs.requires_tracks && capability.required_action_id.is_empty() {
        return Err(InvestigationAdmissionError::NoTrackSource);
    }
    validate_action_position(ctx, actor, capability, kind)?;
    Ok(members)
}
