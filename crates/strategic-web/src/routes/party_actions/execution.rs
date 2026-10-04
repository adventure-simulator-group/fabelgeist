//! Immediate leader execution, queued member intent, and approval policy.
use super::super::{AppState, PartyActionOutcome, party_readiness::PartyReadiness};
use super::{
    PartyAction, PartyReadinessRequirement,
    error::{PartyActionError, PartyActionStage},
    location::character_case_site_id,
    planning::planned_travel_call,
};
use crate::spacetimedb::{
    self as db, CharacterView, PartyActionRequestView, PartyView, SpacetimeError, SqlQuery,
    sql_string_literal,
};
use adventuresim_core::identity::{CharacterId, IdentityError, PartyId};
use serde_json::json;

const TEMPORARY_CAPTAIN_APPROVAL_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

/// Execute a leader action immediately, or persist the same validated intent for
/// the party leader when a member attempts it.
pub(crate) async fn execute_or_request_party_action(
    state: &AppState,
    actor_id: CharacterId,
    action: PartyAction,
) -> std::result::Result<PartyActionOutcome, PartyActionError> {
    let character = state
        .db
        .query_one_sats_into::<adventuresim_stdb_client::Character, CharacterView>(
            db::character_by_id(actor_id),
        )
        .await
        .map_err(|source: SpacetimeError| -> PartyActionError {
            PartyActionError::Database {
                stage: PartyActionStage::Actor,
                actor: actor_id,
                source,
            }
        })?
        .ok_or(PartyActionError::MissingActor(actor_id))?;
    let party_id = character
        .party_id
        .ok_or(PartyActionError::NoParty(actor_id))?;
    let party_id =
        PartyId::try_new(party_id).map_err(|source: IdentityError| -> PartyActionError {
            PartyActionError::PartyIdentity {
                actor: actor_id,
                source,
            }
        })?;
    let party = state
        .db
        .query_one_sats_into::<adventuresim_stdb_client::Party, PartyView>(
            crate::spacetimedb::party_by_id(party_id.as_str()),
        )
        .await
        .map_err(|source: SpacetimeError| -> PartyActionError {
            PartyActionError::Database {
                stage: PartyActionStage::Party,
                actor: actor_id,
                source,
            }
        })?
        .ok_or(PartyActionError::MissingParty(actor_id))?;
    let actor_case_site_id = if matches!(&action, PartyAction::TravelToSettlement { .. }) {
        character_case_site_id(state, actor_id).await?
    } else {
        None
    };
    if action.readiness(actor_case_site_id.as_ref(), &party) == PartyReadinessRequirement::Required
    {
        PartyReadiness::new(&party, actor_id).require(state).await?;
    }

    let leader_id = CharacterId::from(party.leader_id);
    if leader_id == actor_id {
        match planned_travel_call(state, actor_id, &action).await? {
            Some(planned) => planned.execute(&state.db).await?,
            None => action.execute(actor_id, &state.db).await?,
        }
        return Ok(PartyActionOutcome::Executed);
    }
    QueuedPartyIntent {
        actor: actor_id,
        leader: leader_id,
        party: party_id,
        action: &action,
    }
    .request(state)
    .await?;
    Ok(PartyActionOutcome::Requested)
}

pub(crate) async fn approve_party_action(
    state: &AppState,
    leader_id: CharacterId,
    request: &PartyActionRequestView,
) -> std::result::Result<(), PartyActionError> {
    if let Ok(action) = serde_json::from_str::<PartyAction>(&request.payload)
        && let Some(planned) = planned_travel_call(state, leader_id, &action).await?
    {
        return planned
            .approve(leader_id, request, &state.db)
            .await
            .map_err(PartyActionError::Travel);
    }
    state
        .db
        .call(
            "approve_party_action_request",
            &[json!(leader_id), json!(request.id)],
        )
        .await
        .map_err(|source: SpacetimeError| -> PartyActionError {
            PartyActionError::Database {
                stage: PartyActionStage::Approve,
                actor: leader_id,
                source,
            }
        })
}

struct QueuedPartyIntent<'a> {
    actor: CharacterId,
    leader: CharacterId,
    party: PartyId,
    action: &'a PartyAction,
}
impl QueuedPartyIntent<'_> {
    async fn request(self, state: &AppState) -> std::result::Result<(), PartyActionError> {
        let actor_id = self.actor;
        let leader_id = self.leader;
        let party_id = self.party;
        let action = self.action;
        let kind = action.kind();
        let payload = serde_json::to_string(&action).map_err(
            |source: serde_json::Error| -> PartyActionError {
                PartyActionError::Payload {
                    actor: actor_id,
                    source,
                }
            },
        )?;
        state
            .db
            .call(
                "request_party_action",
                &[
                    json!(actor_id),
                    json!(&kind),
                    json!(action.to_string()),
                    json!(payload),
                ],
            )
            .await
            .map_err(|source: SpacetimeError| -> PartyActionError {
                PartyActionError::Database {
                    stage: PartyActionStage::Request,
                    actor: actor_id,
                    source,
                }
            })?;

        // Temporary NPC captains always approve after a short, visible delay.
        let leader = state
            .db
            .query_one_sats_into::<adventuresim_stdb_client::Character, CharacterView>(
                db::character_by_id(leader_id),
            )
            .await
            .map_err(|source: SpacetimeError| -> PartyActionError {
                PartyActionError::Database {
                    stage: PartyActionStage::TemporaryLeader,
                    actor: actor_id,
                    source,
                }
            })?;
        if CaptainApprovalPolicy::for_leader(leader.as_ref())
            == CaptainApprovalPolicy::AfterVisibleDelay
        {
            let state = state.clone();
            tokio::spawn(async move {
                tokio::time::sleep(TEMPORARY_CAPTAIN_APPROVAL_DELAY).await;
                let requests = state
                .db
                .query_sats_into::<adventuresim_stdb_client::PartyActionRequest, PartyActionRequestView>(SqlQuery::from(format!(
                    "SELECT * FROM party_action_request WHERE party_id = {}",
                    sql_string_literal(party_id.as_str())
                )))
                .await
                .unwrap_or_default();
                for request in
                    requests
                        .into_iter()
                        .filter(|request: &PartyActionRequestView| -> bool {
                            CharacterId::from(request.requester_id) == actor_id
                                && request.action_kind == kind.to_string()
                        })
                {
                    if let Err(error) = approve_party_action(&state, leader_id, &request).await {
                        tracing::warn!(%error, "temporary captain could not approve party action");
                    }
                }
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CaptainApprovalPolicy {
    AfterVisibleDelay,
    Manual,
}
impl CaptainApprovalPolicy {
    fn for_leader(leader: Option<&CharacterView>) -> Self {
        match leader {
            Some(leader) if leader.temporary => Self::AfterVisibleDelay,
            _ => Self::Manual,
        }
    }
}
