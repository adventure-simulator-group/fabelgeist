//! Initial live dialogue excludes actions whose scene authority is not ported.
//!
//! Investigation interactions return in #1025/#1040; organization and other
//! local activities return through their owning overhaul follow-ups.

use super::*;
use adventuresim_dialogue::{Effect, Fragment, Response, RuntimeSlot, Turn};

pub(super) fn allows_response(session: &DialogueSession, response: &Response) -> bool {
    matches!(session.authority, DialogueSessionAuthority::Venue)
        || (response.effects.iter().all(allows_effect)
            && response.turns.iter().all(allows_turn)
            && response.prompt.as_ref().is_none_or(|prompt| {
                prompt.choices.iter().all(|choice| {
                    choice.effects.iter().all(allows_effect)
                        && choice.result_turns.iter().all(allows_turn)
                })
            }))
}

pub(super) fn require_response(
    session: &DialogueSession,
    response: &Response,
) -> Result<(), String> {
    if !allows_response(session, response) {
        return Err("This activity is not available in a live scene yet".into());
    }
    Ok(())
}

pub(super) fn require_effect(session: &DialogueSession, effect: &Effect) -> Result<(), String> {
    if matches!(session.authority, DialogueSessionAuthority::Tactical(_)) && !allows_effect(effect)
    {
        return Err("This activity is not available in a live scene yet".into());
    }
    Ok(())
}

pub(super) fn clear_topic_options(ctx: &ReducerContext, session: &DialogueSession) {
    let existing: Vec<_> = ctx
        .db
        .dialogue_topic_option()
        .session_id()
        .filter(&session.id)
        .map(|row| row.id)
        .collect();
    for id in existing {
        ctx.db.dialogue_topic_option().id().delete(&id);
    }
}

fn allows_effect(effect: &Effect) -> bool {
    matches!(effect, Effect::LearnTopic { .. })
}

fn allows_turn(turn: &Turn) -> bool {
    turn.fragments.iter().all(|fragment| match fragment {
        Fragment::Runtime { slot } => allows_slot(slot),
        _ => true,
    })
}

fn allows_slot(slot: &RuntimeSlot) -> bool {
    match slot {
        RuntimeSlot::SpeakerName
        | RuntimeSlot::SpeakerDescription
        | RuntimeSlot::Settlement
        | RuntimeSlot::Location
        | RuntimeSlot::TimeWindow
        | RuntimeSlot::OrganizationName
        | RuntimeSlot::OrganizationAdmissionTerms
        | RuntimeSlot::OrganizationDuesTerms
        | RuntimeSlot::OrganizationRoleStanding
        | RuntimeSlot::OrganizationRepresentativeName
        | RuntimeSlot::AddresseeTitle
        | RuntimeSlot::SecondPersonSubject
        | RuntimeSlot::SecondPersonObject
        | RuntimeSlot::SecondPersonPossessive
        | RuntimeSlot::SecondPersonPossessivePronoun
        | RuntimeSlot::SecondPersonReflexive
        | RuntimeSlot::SecondPersonBe
        | RuntimeSlot::SecondPersonHave
        | RuntimeSlot::SecondPersonDo
        | RuntimeSlot::SecondPersonWill
        | RuntimeSlot::SecondPersonMay
        | RuntimeSlot::SecondPersonShould => true,
        RuntimeSlot::Landmark
        | RuntimeSlot::Symptom
        | RuntimeSlot::WitnessCircumstance
        | RuntimeSlot::Claim
        | RuntimeSlot::Uncertainty
        | RuntimeSlot::ReferralName
        | RuntimeSlot::ReferralDescription
        | RuntimeSlot::ReferralRole
        | RuntimeSlot::ReferralLocation
        | RuntimeSlot::DescribedLocation
        | RuntimeSlot::Evidence
        | RuntimeSlot::Proof
        | RuntimeSlot::Testimony
        | RuntimeSlot::ContractTerms => false,
    }
}
