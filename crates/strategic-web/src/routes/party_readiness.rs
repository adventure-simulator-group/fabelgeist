//! Readiness uses the observer's member projection before refreshing condition.

use super::{AppState, data};
use crate::spacetimedb::{
    self as db, CharacterStrategicCondition, PartyMember, PartyView, SpacetimeError, SqlQuery,
    sql_string_literal,
};
use adventuresim_core::identity::CharacterId;
use data::ObservedLife;
use serde_json::json;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReadinessQueryStage {
    Membership,
    ObservedMember,
    RefreshCondition,
    MemberCondition,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PartyReadinessError {
    #[error("{source}")]
    Query {
        stage: ReadinessQueryStage,
        character: Option<CharacterId>,
        #[source]
        source: SpacetimeError,
    },
    #[error("Party member not found")]
    MissingMember(CharacterId),
    #[error("Party member condition not found")]
    MissingCondition(CharacterId),
    #[error("An incapacitated party member must recover before the party can act")]
    Incapacitated(CharacterId),
}

pub(super) struct PartyReadiness<'a> {
    party: &'a PartyView,
    observer: CharacterId,
}

impl<'a> PartyReadiness<'a> {
    pub(super) fn new(party: &'a PartyView, observer: CharacterId) -> Self {
        Self { party, observer }
    }

    pub(super) async fn require(
        self,
        state: &AppState,
    ) -> std::result::Result<(), PartyReadinessError> {
        let members = state
            .db
            .query_sats::<PartyMember>(SqlQuery::from(format!(
                "SELECT * FROM party_member WHERE party_id = {}",
                sql_string_literal(&self.party.id)
            )))
            .await
            .map_err(|source: SpacetimeError| -> PartyReadinessError {
                PartyReadinessError::Query {
                    stage: ReadinessQueryStage::Membership,
                    character: None,
                    source,
                }
            })?;
        for membership in members {
            let subject = CharacterId::from(membership.character_id);
            let member = data::character_as_observed(state, subject, self.observer)
                .await
                .map_err(|source: SpacetimeError| -> PartyReadinessError {
                    PartyReadinessError::Query {
                        stage: ReadinessQueryStage::ObservedMember,
                        character: Some(subject),
                        source,
                    }
                })?
                .ok_or(PartyReadinessError::MissingMember(subject))?;
            // Corpses remain in rendering and history, without gating survivors.
            if ObservedLife::from_character(&member) != ObservedLife::Alive {
                continue;
            }
            let member_id = CharacterId::from(member.id);
            state
                .db
                .call("refresh_strategic_condition", &[json!(member_id)])
                .await
                .map_err(|source: SpacetimeError| -> PartyReadinessError {
                    PartyReadinessError::Query {
                        stage: ReadinessQueryStage::RefreshCondition,
                        character: Some(member_id),
                        source,
                    }
                })?;
            let condition = state
                .db
                .query_one_sats::<CharacterStrategicCondition>(
                    db::character_strategic_condition_by_character_id(member_id),
                )
                .await
                .map_err(|source: SpacetimeError| -> PartyReadinessError {
                    PartyReadinessError::Query {
                        stage: ReadinessQueryStage::MemberCondition,
                        character: Some(member_id),
                        source,
                    }
                })?
                .ok_or(PartyReadinessError::MissingCondition(member_id))?;
            if condition.status == adventuresim_stdb_client::IncapacitationStatus::Incapacitated {
                return Err(PartyReadinessError::Incapacitated(member_id));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
