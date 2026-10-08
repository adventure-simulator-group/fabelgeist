//! Contract admission to the existing service-quest JSON presentation.

use super::service_quests::{service_quest_details, service_quest_greeting};
use crate::{
    routes::travel::active_contract_tooltip,
    spacetimedb::{BackendContract, PartyView, SettlementView},
};
use serde::Serialize;

/// Presentation roles differ from the authoritative backend contract status.
#[derive(Clone, Copy, Serialize)]
enum ServiceQuestOfferState {
    #[serde(rename = "available")]
    Available,
    #[serde(rename = "underway")]
    Underway,
    #[serde(rename = "ready")]
    Ready,
}

#[derive(Serialize)]
pub(super) struct ServiceQuestOffer {
    id: String,
    title: String,
    description: String,
    service_id: String,
    npc_name: &'static str,
    greeting: String,
    problem: String,
    follow_up: String,
    details: String,
    acceptance: &'static str,
    state: ServiceQuestOfferState,
    waiting: &'static str,
    turn_in_response: String,
    can_accept: bool,
    can_turn_in: bool,
}

impl ServiceQuestOffer {
    pub(super) fn from_contract(
        quest: &BackendContract,
        settlement: &SettlementView,
        neighboring_name: &str,
        active_party: Option<&PartyView>,
        can_accept: bool,
        can_turn_in: bool,
    ) -> Option<Self> {
        let is_current = active_party.is_some_and(|party| {
            party.active_contract_id.as_deref() == Some(quest.id.as_str())
                && quest.accepted_by.as_deref() == Some(party.id.as_str())
        });
        let state = if quest.status == adventuresim_stdb_client::ContractStatus::Offered {
            ServiceQuestOfferState::Available
        } else if is_current
            && quest.status == adventuresim_stdb_client::ContractStatus::ReadyToReport
        {
            ServiceQuestOfferState::Ready
        } else if is_current {
            ServiceQuestOfferState::Underway
        } else {
            return None;
        };
        let problem = quest.description.trim_end_matches('.').to_lowercase();
        // Fixed human-facing catalog text is projected at the JSON formatting port.
        let (npc_name, greeting) = service_quest_greeting(&quest.service_id);
        Some(Self {
            id: quest.id.clone(),
            title: quest.title.clone(),
            description: active_contract_tooltip(quest),
            service_id: quest.service_id.clone(),
            npc_name,
            greeting: greeting.to_string(),
            follow_up: format!("{problem}?"),
            problem,
            details: service_quest_details(
                &quest.service_id,
                quest,
                &settlement.name,
                neighboring_name,
            ),
            acceptance: "Excellent! Yet have a care: ye would not be the first men they have slain.",
            state,
            waiting: "Well met again; I eagerly await the fruit of these labours.",
            turn_in_response: format!(
                "Excellent work. Here is the promised sum: {} coin. Ye have earned it.",
                quest.gold_reward
            ),
            can_accept,
            can_turn_in: can_turn_in && matches!(state, ServiceQuestOfferState::Ready),
        })
    }
}
