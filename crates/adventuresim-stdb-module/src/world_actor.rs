//! Unified strategic presence and contextual role authority for every Character.

use adventuresim_world_schema::calendar::StrategicMinute;
use spacetimedb::{ReducerContext, SpacetimeType, Table, ViewContext, reducer, table, view};

use crate::{
    character::{character, character__view, character_death__view},
    condition::{character_strategic_condition, character_strategic_condition__view},
    investigation::{
        canonical_case_site_place, case_context_presence_for_observer, case_site_authority__view,
        case_site_presence_for_observer, case_site_provenance_view,
        character_case_site_occupancy__view, investigation_lead__view,
    },
    relationship::character_birth__view,
    strategic::{
        StrategicEncounterStatus, party_authority, party_authority__view, road_challenge_authority,
        road_challenge_authority__view, strategic_encounter, strategic_encounter__view,
        strategic_gateway_authority__view,
    },
    surgery::limb_injury__view,
    time::{character_time, character_time__view},
};

const EXACT_CASE_CONTEXT_CONTACT_REF: &str = "exact_case_context";

#[derive(Clone, Copy, Debug, PartialEq, Eq, SpacetimeType)]
pub enum CharacterContextKind {
    HostileGroup,
    CaseSite,
    StrategicEncounter,
    RoadEncounter,
}

pub use adventuresim_core::strategic_presence::{
    CharacterContextRole, ContextualDecisionState, InteractionPresentationDecision,
};

/// A Character's role and presence in a strategic context. Hostility is
/// deliberately contextual; it is never intrinsic Character state.
#[derive(Clone, Debug)]
#[table(accessor = character_context_membership)]
pub struct CharacterContextMembership {
    #[primary_key]
    pub id: String,
    #[index(btree)]
    pub context_id: String,
    #[index(btree)]
    pub location_id: String,
    #[index(btree)]
    pub character_id: u64,
    pub context_kind: CharacterContextKind,
    pub role: CharacterContextRole,
    pub ordinal: u16,
    pub entered_at: StrategicMinute,
    pub left_at: Option<StrategicMinute>,
    pub revision: u32,
    pub contact_decision: ContextualDecisionState,
    /// Explicit treatment answer. Narrow emergency bandaging is evaluated live
    /// and is not copied into contextual authority.
    pub treatment_decision: ContextualDecisionState,
}

impl CharacterContextMembership {
    /// The interval owns closure. Open membership has no separate status column.
    pub fn is_open(&self) -> bool {
        self.left_at.is_none()
    }
}

#[derive(Clone, Debug, SpacetimeType)]
pub struct BackendContextCharacter {
    pub party_id: String,
    /// Public encounter/road-challenge ID, or the fixed action/context
    /// discriminator for case-site and hostile actors. Private case and
    /// hostile-group IDs never cross this view.
    pub contact_ref: String,
    pub context_kind: CharacterContextKind,
    pub location_id: String,
    pub character_id: u64,
    pub role: CharacterContextRole,
    pub ordinal: u16,
    pub alive: bool,
    pub revision: u32,
    pub membership_revision: u32,
    pub contact_decision: InteractionPresentationDecision,
    pub treatment_decision: InteractionPresentationDecision,
    pub treatment_limb_slug: Option<String>,
}

/// Row existence records completed party contact and mutual awareness. Contact
/// commits this authority with encounter awareness and an immutable retry receipt
/// in one transaction. `context_id` remains private; callers use a public context
/// reference and target Character. Mission snapshots derive surprise from presence.
#[derive(Clone, Debug)]
#[table(accessor = party_context_contact_authority)]
pub struct PartyContextContactAuthority {
    #[primary_key]
    pub id: String,
    #[index(btree)]
    pub party_id: String,
    pub context_id: String,
    pub location_id: String,
    pub revision: u32,
}

#[derive(Clone, Debug)]
#[table(accessor = contextual_contact_receipt)]
pub struct ContextualContactReceipt {
    #[primary_key]
    pub id: String,
    pub actor_id: u64,
    pub target_id: u64,
    pub context_id: String,
    pub action_id: String,
    pub expected_revision: u32,
    pub resulting_revision: u32,
}

/// Gateway-only, role-minimal projection. Callers must query by exact context;
/// no private group composition or future encounter is exposed to players.
#[view(accessor = backend_context_characters, public)]
pub fn backend_context_characters(ctx: &ViewContext) -> Vec<BackendContextCharacter> {
    let gateway = ctx
        .db
        .strategic_gateway_authority()
        .id()
        .find(0)
        .is_some_and(|row| row.identity == ctx.sender());
    if !gateway {
        return Vec::new();
    }
    let mut result = Vec::new();
    for row in ctx
        .db
        .character_context_membership()
        .character_id()
        .filter(0u64..)
    {
        if !context_membership_interval_is_well_formed(&row) {
            continue;
        }
        if !matches!(
            row.context_kind,
            CharacterContextKind::CaseSite | CharacterContextKind::HostileGroup
        ) && !row.is_open()
        {
            continue;
        }
        let Some(character) = ctx.db.character().id().find(row.character_id) else {
            continue;
        };
        let parties = match row.context_kind {
            CharacterContextKind::CaseSite | CharacterContextKind::HostileGroup => ctx
                .db
                .party_authority()
                .gateway_bucket()
                .filter(0u8)
                .filter(|party| case_context_party_visible_at_view(ctx, &row, party))
                .filter_map(|party| {
                    let minute = ctx
                        .db
                        .character_time()
                        .character_id()
                        .find(party.leader_id)?
                        .minutes;
                    let party_id = party.id;
                    Some((
                        party_id,
                        EXACT_CASE_CONTEXT_CONTACT_REF.to_string(),
                        character_alive_at_for_view(ctx, row.character_id, minute),
                    ))
                })
                .collect(),
            CharacterContextKind::StrategicEncounter => ctx
                .db
                .party_authority()
                .gateway_bucket()
                .filter(0u8)
                .filter_map(|party| {
                    ctx.db
                        .strategic_encounter()
                        .party_id()
                        .find(&party.id)
                        .filter(|encounter| {
                            encounter.encounter_id == row.context_id
                                && encounter.status == StrategicEncounterStatus::AwaitingChoice
                        })
                        .map(|encounter| {
                            (encounter.party_id, row.context_id.clone(), character.alive)
                        })
                })
                .collect::<Vec<_>>(),
            CharacterContextKind::RoadEncounter => ctx
                .db
                .road_challenge_authority()
                .gateway_bucket()
                .filter(0u8)
                .filter(|challenge| challenge.id == row.context_id && challenge.is_open())
                .map(|challenge| (challenge.party_id, challenge.id, character.alive))
                .collect(),
        };
        for (party_id, contact_ref, alive_at_frontier) in parties {
            let contact_id = party_context_contact_id(&party_id, &row.context_id);
            let revision = if matches!(
                row.context_kind,
                CharacterContextKind::CaseSite | CharacterContextKind::HostileGroup
            ) {
                row.revision
            } else {
                ctx.db
                    .party_context_contact_authority()
                    .id()
                    .find(&contact_id)
                    .map_or_else(
                        || {
                            if row.context_kind == CharacterContextKind::StrategicEncounter {
                                ctx.db
                                    .strategic_encounter()
                                    .party_id()
                                    .find(&party_id)
                                    .map_or(row.revision, |encounter| encounter.revision)
                            } else {
                                1
                            }
                        },
                        |contact| contact.revision,
                    )
            };
            let treatment_limb = adventuresim_core::physiology::BodyRegion::ALL
                .into_iter()
                .find(|limb| {
                    ctx.db
                        .limb_injury()
                        .character_id()
                        .filter(row.character_id)
                        .find(|injury| injury.limb == *limb)
                        .is_some_and(|injury| injury.cut_damage > 0.0 && !injury.bandaged)
                });
            let incapacitated = ctx
                .db
                .character_strategic_condition()
                .character_id()
                .find(row.character_id)
                .is_some_and(|condition| {
                    condition.status
                        == adventuresim_core::morale::IncapacitationStatus::Incapacitated
                });
            let emergency_bandage = row.treatment_decision != ContextualDecisionState::Refused
                && treatment_limb.is_some_and(|limb| {
                    ctx.db
                        .limb_injury()
                        .character_id()
                        .filter(row.character_id)
                        .find(|injury| injury.limb == limb)
                        .is_some_and(|injury| {
                            adventuresim_core::strategic_action::emergency_bandage_is_necessary(
                                incapacitated,
                                adventuresim_core::surgery::SurgeryProcedure::Bandage,
                                injury.cut_damage,
                                injury.bandaged,
                            )
                        })
                });
            result.push(BackendContextCharacter {
                party_id,
                contact_ref,
                context_kind: row.context_kind,
                location_id: row.location_id.clone(),
                character_id: row.character_id,
                role: row.role,
                ordinal: row.ordinal,
                alive: alive_at_frontier,
                revision,
                membership_revision: row.revision,
                contact_decision: row.contact_decision.presentation(),
                treatment_decision: if row.treatment_decision
                    == ContextualDecisionState::Unavailable
                    && emergency_bandage
                {
                    InteractionPresentationDecision::EmergencyTreatment
                } else {
                    row.treatment_decision.presentation()
                },
                treatment_limb_slug: treatment_limb.map(|limb| limb.slug().to_owned()),
            });
        }
    }
    result
}

fn party_context_contact_id(party_id: &str, context_id: &str) -> String {
    format!("party-context-contact:{party_id}:{context_id}")
}

pub(crate) fn context_contact_revision_view(
    ctx: &ViewContext,
    party_id: &str,
    context_id: &str,
    fallback: u32,
) -> u32 {
    ctx.db
        .party_context_contact_authority()
        .id()
        .find(party_context_contact_id(party_id, context_id))
        .map_or(fallback, |contact| contact.revision)
}

pub(crate) fn party_contacted_context(
    ctx: &ReducerContext,
    party_id: &str,
    context_id: &str,
) -> bool {
    ctx.db
        .party_context_contact_authority()
        .id()
        .find(party_context_contact_id(party_id, context_id))
        .is_some()
}

pub(crate) fn context_members(
    ctx: &ReducerContext,
    context_id: &str,
) -> Vec<CharacterContextMembership> {
    let mut rows = ctx
        .db
        .character_context_membership()
        .context_id()
        .filter(&context_id.to_string())
        .filter(|row| context_membership_interval_is_well_formed(row) && row.is_open())
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.ordinal);
    rows
}

pub(crate) fn context_character_ids(ctx: &ReducerContext, context_id: &str) -> Vec<u64> {
    context_members(ctx, context_id)
        .into_iter()
        .filter_map(|row| {
            ctx.db
                .character()
                .id()
                .find(row.character_id)
                .filter(|character| character.alive)
                .map(|character| character.id)
        })
        .collect()
}

fn field_character_id(context_id: &str, ordinal: u16) -> u64 {
    adventuresim_core::settlement_population::stable_hash(&format!(
        "field-character:{context_id}:{ordinal}"
    )) | (1u64 << 63)
}

pub(crate) fn materialize_context_roster(
    ctx: &ReducerContext,
    kind: CharacterContextKind,
    context_id: &str,
    location_id: &str,
    count: u32,
) -> Result<Vec<u64>, String> {
    let entered_at = crate::time::refresh_clock(ctx)?;
    let expected = count.min(u32::from(u16::MAX));
    let existing = context_members(ctx, context_id);
    if !existing.is_empty() {
        if existing.len() > expected as usize
            || existing.iter().any(|row| {
                row.context_kind != kind
                    || row.location_id != location_id
                    || row.role != CharacterContextRole::Counterparty
            })
        {
            return Err("Context roster conflicts with its immutable materialization".into());
        }
        if existing.len() == expected as usize {
            return Ok(existing.into_iter().map(|row| row.character_id).collect());
        }
    }
    let mut ids = Vec::with_capacity(expected as usize);
    ids.extend(existing.iter().map(|row| row.character_id));
    for ordinal in existing.len() as u16..expected as u16 {
        let id = field_character_id(context_id, ordinal);
        if ctx.db.character().id().find(id).is_some() {
            return Err("Deterministic field-character identity collision".into());
        }
        crate::character::insert_persistent_field_character(
            ctx,
            "Pending generated name".into(),
            id,
            id,
            None,
        )?;
        crate::character::assign_generated_historical_name_for_age(
            ctx,
            adventuresim_core::identity::CharacterId::from(id),
            adventuresim_world_schema::person_names::NameStableSeed::new(id),
            entered_at,
            None,
        )
        .map_err(|error| error.to_string())?;
        ctx.db
            .character_context_membership()
            .insert(CharacterContextMembership {
                id: format!("context:{context_id}:{ordinal}"),
                context_id: context_id.to_string(),
                location_id: location_id.to_string(),
                character_id: id,
                context_kind: kind,
                role: CharacterContextRole::Counterparty,
                ordinal,
                entered_at,
                left_at: None,
                revision: 1,
                contact_decision: ContextualDecisionState::Allowed,
                treatment_decision: ContextualDecisionState::Unavailable,
            });
        ids.push(id);
    }
    Ok(ids)
}

/// Carry already-materialized mortal road counterparties into a combat
/// follow-up without replacing their Character identity or components.
pub(crate) fn rebind_road_cast_to_strategic_encounter(
    ctx: &ReducerContext,
    road_context_id: &str,
    encounter_id: &str,
    count: u32,
) -> Result<Vec<u64>, String> {
    let entered_at = crate::time::refresh_clock(ctx)?;
    let mut eligible = context_members(ctx, road_context_id)
        .into_iter()
        .filter(|membership| {
            membership.context_kind == CharacterContextKind::RoadEncounter
                && membership.role == CharacterContextRole::Counterparty
                && ctx
                    .db
                    .character()
                    .id()
                    .find(membership.character_id)
                    .is_some_and(|character| character.alive)
        })
        .take(usize::try_from(count).unwrap_or(usize::MAX))
        .collect::<Vec<_>>();
    eligible.sort_by_key(|membership| membership.ordinal);
    for (ordinal, road_membership) in eligible.iter().enumerate() {
        let ordinal = u16::try_from(ordinal)
            .map_err(|_| "Strategic encounter roster exceeds the supported size")?;
        let id = format!("context:{encounter_id}:{ordinal}");
        let rebound = CharacterContextMembership {
            id: id.clone(),
            context_id: encounter_id.into(),
            location_id: encounter_id.into(),
            character_id: road_membership.character_id,
            context_kind: CharacterContextKind::StrategicEncounter,
            role: CharacterContextRole::Counterparty,
            ordinal,
            entered_at,
            left_at: None,
            revision: 1,
            contact_decision: ContextualDecisionState::Allowed,
            treatment_decision: ContextualDecisionState::Unavailable,
        };
        if let Some(existing) = ctx.db.character_context_membership().id().find(&id) {
            if existing.context_id != rebound.context_id
                || existing.character_id != rebound.character_id
                || existing.context_kind != rebound.context_kind
                || existing.role != rebound.role
            {
                return Err("Road-to-combat Character identity collision".into());
            }
        } else {
            ctx.db.character_context_membership().insert(rebound);
        }
    }
    materialize_context_roster(
        ctx,
        CharacterContextKind::StrategicEncounter,
        encounter_id,
        encounter_id,
        count,
    )
}

include!("world_actor/presence.rs");

include!("world_actor/roster.rs");

#[cfg(feature = "authority-tests")]
include!("world_actor/authority_tests.rs");
#[cfg(feature = "authority-tests")]
include!("world_actor/contact_authority_tests.rs");

/// Materialize every individualized mortal in a compiled road cast as an
/// ordinary, fully componentized Character. Cast order is the stable identity
/// coordinate; narrative collectives and explicitly blocked figures never
/// receive a surrogate Character row.
pub(crate) fn materialize_road_encounter_cast(
    ctx: &ReducerContext,
    context_id: &str,
    definition: &adventuresim_core::road_encounter_catalog::EncounterDefinition,
    absolute_minute: StrategicMinute,
) -> Result<Vec<u64>, String> {
    use adventuresim_core::road_encounter_catalog::SpeakerBacking;

    let mut materialized = Vec::new();
    for (cast_ordinal, speaker) in definition.cast.iter().enumerate() {
        let SpeakerBacking::Character {
            role,
            contact_decision,
            treatment_decision,
        } = &speaker.backing
        else {
            continue;
        };
        let ordinal = u16::try_from(cast_ordinal)
            .map_err(|_| "Road encounter cast exceeds the supported roster size")?;
        let membership_id = format!("context:{context_id}:{ordinal}");
        let character_id = field_character_id(context_id, ordinal);
        let expected_role = *role;
        let existing_membership = ctx
            .db
            .character_context_membership()
            .id()
            .find(&membership_id);
        let existing_character = ctx.db.character().id().find(character_id);
        match (existing_membership, existing_character) {
            (Some(membership), Some(character)) => {
                if membership.context_id != context_id
                    || membership.location_id != context_id
                    || membership.character_id != character_id
                    || membership.context_kind != CharacterContextKind::RoadEncounter
                    || membership.role != expected_role
                    || membership.ordinal != ordinal
                    || !context_membership_interval_is_well_formed(&membership)
                    || !membership.is_open()
                    || membership.contact_decision != *contact_decision
                    || membership.treatment_decision != *treatment_decision
                    || character.name != speaker.name
                {
                    return Err(
                        "Road cast retry conflicts with immutable Character authority".into(),
                    );
                }
                materialized.push(character_id);
                continue;
            }
            (Some(_), None) | (None, Some(_)) => {
                return Err("Road cast retry found partial Character authority".into());
            }
            (None, None) => {}
        }
        crate::character::insert_persistent_field_character(
            ctx,
            speaker.name.clone(),
            character_id,
            character_id,
            Some(absolute_minute),
        )?;
        ctx.db
            .character_context_membership()
            .insert(CharacterContextMembership {
                id: membership_id,
                context_id: context_id.into(),
                location_id: context_id.into(),
                character_id,
                context_kind: CharacterContextKind::RoadEncounter,
                role: expected_role,
                ordinal,
                entered_at: absolute_minute,
                left_at: None,
                revision: 1,
                contact_decision: *contact_decision,
                treatment_decision: *treatment_decision,
            });
        if expected_role == CharacterContextRole::Patient {
            crate::surgery::seed_field_cut(
                ctx,
                character_id,
                adventuresim_core::physiology::BodyRegion::LeftArm,
                0.35,
                absolute_minute,
            );
            if *treatment_decision == ContextualDecisionState::Unavailable {
                crate::condition::apply_blood_loss(ctx, (character_id).into(), 0.30)?;
            }
        }
        materialized.push(character_id);
    }
    Ok(materialized)
}

pub(crate) fn characters_are_contextually_present(
    ctx: &ReducerContext,
    actor_id: adventuresim_core::identity::CharacterId,
    target_id: adventuresim_core::identity::CharacterId,
) -> bool {
    let Some(actor) = ctx.db.character().id().find(u64::from(actor_id)) else {
        return false;
    };
    let Some(target) = ctx.db.character().id().find(u64::from(target_id)) else {
        return false;
    };
    if actor.current_settlement_id.is_some()
        && actor.current_settlement_id == target.current_settlement_id
    {
        return true;
    }
    let actor_case_presence = actor_case_presence(ctx, (actor_id).into());
    if let Some((actor_presence, minute)) = actor_case_presence.as_ref()
        && case_site_presence_for_observer(ctx, (actor_id).into(), (target_id).into(), *minute)
            .is_some_and(|target_presence| {
                adventuresim_core::strategic_presence::are_co_present(
                    actor_presence,
                    &target_presence,
                )
            })
    {
        return true;
    }
    ctx.db
        .character_context_membership()
        .character_id()
        .filter(u64::from(target_id))
        .filter(|row| {
            context_membership_interval_is_well_formed(row)
                && (row.is_open()
                    || matches!(
                        row.context_kind,
                        CharacterContextKind::CaseSite | CharacterContextKind::HostileGroup
                    ))
        })
        .any(|row| match row.context_kind {
            CharacterContextKind::CaseSite | CharacterContextKind::HostileGroup => {
                actor_case_presence
                    .as_ref()
                    .is_some_and(|(actor_presence, minute)| {
                        let minute = *minute;
                        let Some((projected_id, projected_revision)) =
                            projected_case_context_claim(ctx, (actor_id).into(), &row, minute)
                        else {
                            return false;
                        };
                        case_context_presence_for_observer(
                            ctx,
                            (actor_id).into(),
                            &row,
                            &projected_id,
                            projected_revision,
                            minute,
                        )
                        .is_some_and(|target_presence| {
                            adventuresim_core::strategic_presence::are_co_present(
                                actor_presence,
                                &target_presence,
                            )
                        })
                    })
            }
            CharacterContextKind::StrategicEncounter => {
                actor.party_id.as_ref().is_some_and(|party_id| {
                    ctx.db
                        .strategic_encounter()
                        .party_id()
                        .find(party_id)
                        .is_some_and(|encounter| {
                            encounter.encounter_id == row.context_id
                                && encounter.status == StrategicEncounterStatus::AwaitingChoice
                        })
                })
            }
            CharacterContextKind::RoadEncounter => {
                actor.party_id.as_ref().is_some_and(|party_id| {
                    ctx.db
                        .party_authority()
                        .id()
                        .find(party_id)
                        .is_some_and(|party| {
                            ctx.db
                                .road_challenge_authority()
                                .id()
                                .find(&row.context_id)
                                .is_some_and(|challenge| {
                                    challenge.party_id == *party_id
                                        && challenge.is_open()
                                        && crate::strategic::party_at_bound_road_challenge(
                                            ctx, &party, &challenge,
                                        )
                                })
                        })
                })
            }
        })
}

fn contextual_membership_is_visible(
    ctx: &ReducerContext,
    actor_id: u64,
    membership: &CharacterContextMembership,
) -> bool {
    let Some(actor) = ctx.db.character().id().find(actor_id) else {
        return false;
    };
    let Some(party_id) = actor.party_id.as_deref() else {
        return false;
    };
    let Some(actor_minute) = ctx
        .db
        .character_time()
        .character_id()
        .find(actor_id)
        .map(|row| row.minutes)
    else {
        return false;
    };
    (if matches!(
        membership.context_kind,
        CharacterContextKind::CaseSite | CharacterContextKind::HostileGroup
    ) {
        context_membership_valid_at(membership, actor_minute)
    } else {
        context_membership_interval_is_well_formed(membership) && membership.is_open()
    }) && characters_are_contextually_present(
        ctx,
        (actor_id).into(),
        (membership.character_id).into(),
    ) && match membership.context_kind {
        CharacterContextKind::CaseSite => crate::outbreak::case_patient_visible_to_character(
            ctx,
            actor_id,
            &membership.context_id,
            actor_minute,
        ),
        CharacterContextKind::RoadEncounter => ctx
            .db
            .road_challenge_authority()
            .id()
            .find(&membership.context_id)
            .is_some_and(|challenge| challenge.party_id == party_id),
        CharacterContextKind::StrategicEncounter => ctx
            .db
            .strategic_encounter()
            .party_id()
            .find(party_id.to_owned())
            .is_some_and(|encounter| encounter.encounter_id == membership.context_id),
        CharacterContextKind::HostileGroup => true,
    }
}

fn public_decision(
    state: ContextualDecisionState,
) -> adventuresim_core::strategic_action::ContextualActionDecision {
    use adventuresim_core::strategic_action::{ContextualActionDecision, ContextualActionReason};
    match state {
        ContextualDecisionState::Allowed => {
            ContextualActionDecision::Allowed(ContextualActionReason::TargetPermission)
        }
        ContextualDecisionState::Refused => ContextualActionDecision::Refused,
        ContextualDecisionState::Unavailable => ContextualActionDecision::Unavailable,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ContextualTreatmentClaim {
    pub contact_ref: String,
    pub expected_membership_revision: u32,
}

fn contextual_treatment_claim_matches(
    membership: &CharacterContextMembership,
    claim: &ContextualTreatmentClaim,
) -> bool {
    let reference_matches = match membership.context_kind {
        CharacterContextKind::CaseSite | CharacterContextKind::HostileGroup => {
            claim.contact_ref == EXACT_CASE_CONTEXT_CONTACT_REF
        }
        CharacterContextKind::StrategicEncounter | CharacterContextKind::RoadEncounter => {
            claim.contact_ref == membership.context_id
        }
    };
    reference_matches && claim.expected_membership_revision == membership.revision
}

fn treatment_target_answer(
    contextual_answer: Option<ContextualDecisionState>,
    party_preference: Option<ContextualDecisionState>,
) -> adventuresim_core::strategic_action::ContextualActionDecision {
    contextual_answer.or(party_preference).map_or(
        adventuresim_core::strategic_action::ContextualActionDecision::Unavailable,
        public_decision,
    )
}

fn contextual_contact_decision(
    ctx: &ReducerContext,
    actor_id: u64,
    target_id: u64,
    membership: &CharacterContextMembership,
) -> adventuresim_core::strategic_action::ContextualActionDecision {
    use adventuresim_core::strategic_action::{ContextualActionDecision, ContextualActionReason};
    if actor_id == target_id {
        return ContextualActionDecision::Allowed(ContextualActionReason::SelfAction);
    }
    if membership.character_id != target_id
        || !ctx
            .db
            .character()
            .id()
            .find(target_id)
            .is_some_and(|target| target.alive)
        || !contextual_membership_is_visible(ctx, actor_id, membership)
    {
        return ContextualActionDecision::Unavailable;
    }
    public_decision(membership.contact_decision)
}

fn contextual_treatment_decision_with_emergency(
    ctx: &ReducerContext,
    actor_id: u64,
    patient_id: u64,
    emergency_medical_necessity: bool,
    claim: Option<&ContextualTreatmentClaim>,
) -> adventuresim_core::strategic_action::ContextualActionDecision {
    use adventuresim_core::strategic_action::ContextualActionDecision;
    let Some(actor) = ctx
        .db
        .character()
        .id()
        .find(actor_id)
        .filter(|row| row.alive)
    else {
        return ContextualActionDecision::Unavailable;
    };
    let Some(patient) = ctx
        .db
        .character()
        .id()
        .find(patient_id)
        .filter(|row| row.alive)
    else {
        return ContextualActionDecision::Unavailable;
    };
    if actor_id == patient_id {
        if claim.is_some() {
            return ContextualActionDecision::Unavailable;
        }
        return adventuresim_core::strategic_action::decide_contextual_action(
            true,
            ContextualActionDecision::Unavailable,
            false,
        );
    }
    if !characters_are_contextually_present(ctx, (actor_id).into(), (patient_id).into()) {
        return ContextualActionDecision::Unavailable;
    }

    let actor_minute = ctx
        .db
        .character_time()
        .character_id()
        .find(actor_id)
        .map_or(StrategicMinute::ZERO, |t| t.minutes);
    let contextual = ctx
        .db
        .character_context_membership()
        .character_id()
        .filter(patient_id)
        .filter(|membership| {
            if matches!(
                membership.context_kind,
                CharacterContextKind::CaseSite | CharacterContextKind::HostileGroup
            ) {
                context_membership_valid_at(membership, actor_minute)
            } else {
                context_membership_interval_is_well_formed(membership) && membership.is_open()
            }
        })
        .collect::<Vec<_>>();
    let authored = if contextual.is_empty() {
        if claim.is_some() {
            return ContextualActionDecision::Unavailable;
        }
        None
    } else {
        let Some(claim) = claim else {
            return ContextualActionDecision::Unavailable;
        };
        let matching = contextual.iter().filter(|membership| {
            contextual_treatment_claim_matches(membership, claim)
                && contextual_membership_is_visible(ctx, actor_id, membership)
        });
        let Some(membership) = exactly_one(matching) else {
            return ContextualActionDecision::Unavailable;
        };
        Some(membership.treatment_decision)
    };
    let same_party_preference = (actor.party_id.is_some() && actor.party_id == patient.party_id)
        .then_some(patient.party_treatment_decision);
    adventuresim_core::strategic_action::decide_contextual_action(
        false,
        treatment_target_answer(authored, same_party_preference),
        emergency_medical_necessity,
    )
}

pub(crate) fn contextual_treatment_decision(
    ctx: &ReducerContext,
    actor_id: u64,
    patient_id: u64,
    limb: adventuresim_core::physiology::BodyRegion,
    procedure: adventuresim_core::surgery::SurgeryProcedure,
    claim: Option<&ContextualTreatmentClaim>,
) -> adventuresim_core::strategic_action::ContextualActionDecision {
    let incapacitated = ctx
        .db
        .character_strategic_condition()
        .character_id()
        .find(patient_id)
        .is_some_and(|row| {
            row.status == adventuresim_core::morale::IncapacitationStatus::Incapacitated
        });
    let injury = crate::surgery::injury_for(ctx, (patient_id).into(), limb);
    let emergency_bandage = adventuresim_core::strategic_action::emergency_bandage_is_necessary(
        incapacitated,
        procedure,
        injury.cut_damage,
        injury.bandaged,
    );
    contextual_treatment_decision_with_emergency(
        ctx,
        actor_id,
        patient_id,
        emergency_bandage,
        claim,
    )
}

/// Treatment decision for preparations and other interventions which never
/// receive the emergency-bandage exception.
pub(crate) fn contextual_nonemergency_treatment_decision(
    ctx: &ReducerContext,
    actor_id: u64,
    patient_id: u64,
) -> adventuresim_core::strategic_action::ContextualActionDecision {
    contextual_treatment_decision_with_emergency(ctx, actor_id, patient_id, false, None)
}

pub(crate) fn context_patient_is_treated(ctx: &ReducerContext, context_id: &str) -> bool {
    context_members(ctx, context_id)
        .into_iter()
        .find(|row| row.role == CharacterContextRole::Patient)
        .is_some_and(|row| {
            adventuresim_core::physiology::BodyRegion::ALL
                .into_iter()
                .any(|limb| {
                    let injury = crate::surgery::injury_for(ctx, (row.character_id).into(), limb);
                    injury.cut_damage > 0.0 && injury.bandaged
                })
        })
}

/// Initiate ordinary social contact with any living co-present Character.
/// Contact is intentionally not a full authored-dialogue session: it lays the
/// durable relationship edge and changes encounter awareness atomically.
#[reducer]
pub fn contact_context_character(
    ctx: &ReducerContext,
    actor_id: u64,
    target_id: u64,
    contact_ref: String,
    expected_revision: u32,
    action_id: String,
) -> Result<(), String> {
    crate::strategic::require_strategic_character_authority(ctx, (actor_id).into())
        .map_err(|error: crate::strategic::StrategicCharacterAuthorityError| error.to_string())?;
    if action_id.is_empty() || action_id.len() > 160 {
        return Err("Contextual contact action ID is invalid".into());
    }
    let receipt_id = format!("context-contact:{actor_id}:{action_id}");
    if let Some(existing) = ctx.db.contextual_contact_receipt().id().find(&receipt_id) {
        return if existing.actor_id == actor_id
            && existing.target_id == target_id
            && existing.context_id == contact_ref
            && existing.expected_revision == expected_revision
        {
            Ok(())
        } else {
            Err("Conflicting contextual contact retry".into())
        };
    }
    crate::character::require_living_character(ctx, (actor_id).into())
        .map_err(|error: crate::character::LivingCharacterError| error.to_string())?;
    crate::character::require_living_character(ctx, (target_id).into())
        .map_err(|error: crate::character::LivingCharacterError| error.to_string())?;
    let actor = ctx
        .db
        .character()
        .id()
        .find(actor_id)
        .ok_or("Contact actor does not exist")?;
    let party_id = actor.party_id.ok_or("Contact requires an active party")?;
    let actor_minute = ctx
        .db
        .character_time()
        .character_id()
        .find(actor_id)
        .ok_or("Contact actor has no personal time")?
        .minutes;
    let case_presence =
        case_site_presence_for_observer(ctx, (actor_id).into(), (actor_id).into(), actor_minute);
    let candidates = ctx
        .db
        .character_context_membership()
        .character_id()
        .filter(target_id)
        .filter(|row| {
            (if matches!(
                row.context_kind,
                CharacterContextKind::CaseSite | CharacterContextKind::HostileGroup
            ) {
                context_membership_valid_at(row, actor_minute)
            } else {
                context_membership_interval_is_well_formed(row) && row.is_open()
            }) && match row.context_kind {
                CharacterContextKind::StrategicEncounter => row.context_id == contact_ref,
                CharacterContextKind::CaseSite | CharacterContextKind::HostileGroup => {
                    exact_context_claim_matches(
                        row.context_kind,
                        &contact_ref,
                        expected_revision,
                        row.revision,
                    ) && case_presence.as_ref().is_some_and(|actor_presence| {
                        case_context_presence_for_observer(
                            ctx,
                            (actor_id).into(),
                            row,
                            &row.id,
                            expected_revision,
                            actor_minute,
                        )
                        .is_some_and(|target_presence| {
                            adventuresim_core::strategic_presence::are_co_present(
                                actor_presence,
                                &target_presence,
                            )
                        })
                    })
                }
                CharacterContextKind::RoadEncounter => row.location_id == contact_ref,
            }
        })
        .collect::<Vec<_>>();
    let membership = exactly_one(candidates.into_iter())
        .ok_or("Target context claim is unavailable or ambiguous")?;
    if membership.context_kind == CharacterContextKind::CaseSite
        && membership.role == CharacterContextRole::Patient
        && !crate::outbreak::case_patient_visible_to_character(
            ctx,
            actor_id,
            &membership.context_id,
            actor_minute,
        )
    {
        return Err("Patient context is not visible at the actor frontier".into());
    }
    match contextual_contact_decision(ctx, actor_id, target_id, &membership) {
        adventuresim_core::strategic_action::ContextualActionDecision::Allowed(_) => {}
        adventuresim_core::strategic_action::ContextualActionDecision::Refused => {
            return Err("Contact was refused".into());
        }
        adventuresim_core::strategic_action::ContextualActionDecision::Unavailable => {
            return Err("Contact is unavailable".into());
        }
    }
    let mut encounter = ctx
        .db
        .strategic_encounter()
        .party_id()
        .find(
            &ctx.db
                .character()
                .id()
                .find(actor_id)
                .and_then(|character| character.party_id)
                .ok_or("Contact requires an active party")?,
        )
        .filter(|encounter| {
            encounter.encounter_id == membership.context_id
                && encounter.status == StrategicEncounterStatus::AwaitingChoice
        });
    let contact_id = party_context_contact_id(&party_id, &membership.context_id);
    let existing_contact = ctx
        .db
        .party_context_contact_authority()
        .id()
        .find(&contact_id);
    let current_revision = if matches!(
        membership.context_kind,
        CharacterContextKind::CaseSite | CharacterContextKind::HostileGroup
    ) {
        membership.revision
    } else {
        existing_contact.as_ref().map_or_else(
            || encounter.as_ref().map_or(1, |encounter| encounter.revision),
            |contact| contact.revision,
        )
    };
    if current_revision != expected_revision {
        return Err("Context contact revision is stale".into());
    }
    let resulting_revision = expected_revision.saturating_add(1);
    if let Some(encounter) = encounter.as_mut() {
        encounter.party_aware = true;
        encounter.enemy_aware = true;
        encounter
            .available_choices
            .retain(|choice| choice != "sneak");
        encounter.selection_explanation =
            "Contact established; both sides are aware and surprise is no longer possible.".into();
        encounter.revision = resulting_revision;
        ctx.db
            .strategic_encounter()
            .party_id()
            .update(encounter.clone());
    } else {
        if membership.context_kind == CharacterContextKind::StrategicEncounter {
            return Err("Strategic encounter is no longer active".into());
        }
    }
    let contact = PartyContextContactAuthority {
        id: contact_id,
        party_id,
        context_id: membership.context_id.clone(),
        location_id: membership.location_id.clone(),
        revision: resulting_revision,
    };
    if existing_contact.is_some() {
        ctx.db
            .party_context_contact_authority()
            .id()
            .update(contact);
    } else {
        ctx.db.party_context_contact_authority().insert(contact);
    }
    crate::social::begin_physiology_presence_on_contact(ctx, actor_id, target_id);
    crate::social::apply_async_socializing(ctx, (actor_id).into(), (target_id).into(), 10)?;
    ctx.db
        .contextual_contact_receipt()
        .insert(ContextualContactReceipt {
            id: receipt_id,
            actor_id,
            target_id,
            context_id: contact_ref,
            action_id,
            expected_revision,
            resulting_revision,
        });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CharacterContextKind, CharacterContextMembership, CharacterContextRole,
        ContextualDecisionState, ContextualTreatmentClaim, EXACT_CASE_CONTEXT_CONTACT_REF,
        StrategicMinute, context_interval_is_well_formed, contextual_treatment_claim_matches,
        exact_context_claim_matches, exactly_one, treatment_target_answer,
    };

    fn treatment_membership(kind: CharacterContextKind) -> CharacterContextMembership {
        CharacterContextMembership {
            id: "membership:1".into(),
            context_id: "road:1".into(),
            location_id: "road:1".into(),
            character_id: 9,
            context_kind: kind,
            role: CharacterContextRole::Patient,
            ordinal: 0,
            entered_at: StrategicMinute::new(1),
            left_at: None,
            revision: 4,
            contact_decision: ContextualDecisionState::Allowed,
            treatment_decision: ContextualDecisionState::Allowed,
        }
    }

    #[test]
    fn context_intervals_reject_reversed_chronology() {
        let entered = StrategicMinute::new(10);
        assert!(context_interval_is_well_formed(entered, None));
        assert!(context_interval_is_well_formed(entered, Some(entered)));
        assert!(!context_interval_is_well_formed(
            entered,
            Some(StrategicMinute::new(9))
        ));
    }

    #[test]
    fn exact_context_claim_rejects_forged_discriminators_and_stale_revisions() {
        assert!(exact_context_claim_matches(
            CharacterContextKind::CaseSite,
            EXACT_CASE_CONTEXT_CONTACT_REF,
            3,
            3,
        ));
        assert!(exact_context_claim_matches(
            CharacterContextKind::HostileGroup,
            EXACT_CASE_CONTEXT_CONTACT_REF,
            3,
            3,
        ));
        assert!(!exact_context_claim_matches(
            CharacterContextKind::CaseSite,
            "forged_private_context",
            3,
            3,
        ));
        assert!(!exact_context_claim_matches(
            CharacterContextKind::CaseSite,
            EXACT_CASE_CONTEXT_CONTACT_REF,
            2,
            3,
        ));
    }

    #[test]
    fn context_claim_resolution_fails_closed_on_zero_or_ambiguous_rows() {
        assert_eq!(exactly_one(std::iter::empty::<u8>()), None);
        assert_eq!(exactly_one([7].into_iter()), Some(7));
        assert_eq!(exactly_one([7, 8].into_iter()), None);
    }

    #[test]
    fn treatment_claims_bind_exact_context_and_membership_revision() {
        let road = treatment_membership(CharacterContextKind::RoadEncounter);
        assert!(contextual_treatment_claim_matches(
            &road,
            &ContextualTreatmentClaim {
                contact_ref: "road:1".into(),
                expected_membership_revision: 4,
            }
        ));
        assert!(!contextual_treatment_claim_matches(
            &road,
            &ContextualTreatmentClaim {
                contact_ref: "road:forged".into(),
                expected_membership_revision: 4,
            }
        ));
        assert!(!contextual_treatment_claim_matches(
            &road,
            &ContextualTreatmentClaim {
                contact_ref: "road:1".into(),
                expected_membership_revision: 3,
            }
        ));

        let case = treatment_membership(CharacterContextKind::CaseSite);
        assert!(contextual_treatment_claim_matches(
            &case,
            &ContextualTreatmentClaim {
                contact_ref: EXACT_CASE_CONTEXT_CONTACT_REF.into(),
                expected_membership_revision: 4,
            }
        ));
        assert!(!contextual_treatment_claim_matches(
            &case,
            &ContextualTreatmentClaim {
                contact_ref: case.context_id.clone(),
                expected_membership_revision: 4,
            }
        ));
    }

    #[test]
    fn contextual_refusal_overrides_ordinary_party_care_preference() {
        use adventuresim_core::strategic_action::{
            ContextualActionDecision, ContextualActionReason,
        };
        assert_eq!(
            treatment_target_answer(None, Some(ContextualDecisionState::Allowed)),
            ContextualActionDecision::Allowed(ContextualActionReason::TargetPermission)
        );
        assert_eq!(
            treatment_target_answer(
                Some(ContextualDecisionState::Refused),
                Some(ContextualDecisionState::Allowed)
            ),
            ContextualActionDecision::Refused
        );
    }

    #[test]
    fn contextual_actions_share_private_presence_decisions_and_physiology_authority() {
        let source = crate::production_source(include_str!("world_actor.rs"));
        let authorization = source
            .split("fn contextual_membership_is_visible")
            .nth(1)
            .and_then(|tail| tail.split("fn public_decision").next())
            .expect("contextual authorization");
        assert!(authorization.contains("case_patient_visible_to_character"));
        assert!(authorization.contains("challenge.party_id == party_id"));
        assert!(authorization.contains("characters_are_contextually_present"));

        let contact = source
            .split("pub fn contact_context_character")
            .nth(1)
            .expect("context contact reducer");
        assert!(contact.contains("contextual_contact_decision"));
        assert!(contact.contains("begin_physiology_presence_on_contact"));
        assert!(contact.contains("retain(|choice| choice != \"sneak\")"));
    }

    #[test]
    fn emergency_treatment_is_only_exact_limb_bandaging() {
        let source = crate::production_source(include_str!("world_actor.rs"));
        let treatment = source
            .split("pub(crate) fn contextual_treatment_decision")
            .nth(1)
            .and_then(|tail| {
                tail.split("pub(crate) fn context_patient_is_treated")
                    .next()
            })
            .expect("treatment decision");
        assert!(treatment.contains("emergency_bandage_is_necessary"));
        assert!(treatment.contains("injury_for(ctx, patient_id, limb)"));
        assert!(treatment.contains("IncapacitationStatus::Incapacitated"));
        assert!(treatment.contains("SurgeryProcedure"));
    }

    #[test]
    fn road_combat_reuses_cast_character_identity() {
        let source = crate::production_source(include_str!("world_actor.rs"));
        let rebound = source
            .split("pub(crate) fn rebind_road_cast_to_strategic_encounter")
            .nth(1)
            .and_then(|tail| tail.split("fn title_case").next())
            .expect("road cast rebound");
        assert!(rebound.contains("character_id: road_membership.character_id"));
        assert!(rebound.contains("CharacterContextKind::StrategicEncounter"));
        assert!(rebound.contains("materialize_context_roster"));
    }

    #[test]
    fn case_context_joins_use_typed_observer_relative_presence() {
        let source = crate::production_source(include_str!("world_actor.rs"));
        let presence = source
            .split("pub(crate) fn characters_are_contextually_present")
            .nth(1)
            .and_then(|tail| tail.split("fn contextual_membership_is_visible").next())
            .expect("contextual presence projection");
        assert!(presence.contains("case_site_presence_for_observer"));
        assert!(presence.contains("case_context_presence_for_observer"));
        assert!(presence.contains("are_co_present"));
        assert!(!presence.contains("actor_site == character_case_site_id"));
        assert!(!presence.contains("actor_site.as_ref() == Some(&row.location_id)"));
    }
}
