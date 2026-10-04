//! Closed set of party commands that can be queued for leader approval.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::spacetimedb::{
    CaseSiteId, PartyView, RoleRequirements, SpacetimeClient, SpacetimeError,
};
use adventuresim_core::identity::CharacterId;
mod error;
mod execution;
mod kind;
mod location;
mod payload;
mod planning;
pub(super) mod terrain_profile;
pub(crate) use error::PartyActionError;
pub(crate) use execution::{approve_party_action, execute_or_request_party_action};
use kind::PartyActionKind;
pub(crate) use location::{CaseSiteObservationError, character_case_site_id};
pub(crate) use terrain_profile::party_terrain_profile;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PartyReadinessRequirement {
    Required,
    Exempt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub(crate) enum PartyAction {
    TravelToSettlement {
        settlement_id: String,
    },
    TravelToCaseSite {
        case_site_id: String,
    },
    RemovePartyMember {
        character_id: CharacterId,
    },
    CreateRecruitmentRole {
        name: String,
        quantity: u32,
        requirements: RoleRequirements,
        save_role: bool,
    },
    UpdateRecruitmentRole {
        role_id: u64,
        name: String,
        quantity: u32,
        requirements: RoleRequirements,
    },
    DeleteRecruitmentRole {
        role_id: u64,
    },
    AcceptJoinRequest {
        request_id: u64,
    },
    RejectJoinRequest {
        request_id: u64,
    },
    AcceptContract {
        contract_id: String,
    },
    AbandonContract {
        contract_id: String,
    },
    ReportContract {
        contract_id: String,
    },
    AutoresolveMission {
        mission_id: String,
    },
    UpdatePartyCheckTargets {
        physiology: f32,
        command: f32,
        religion: f32,
    },
    SetInventoryQuantityTarget {
        item_id: String,
        quantity: u32,
    },
    DisbandParty {
        party_id: String,
    },
    RequestTacticalServer {
        mission_id: String,
        scene_key: String,
    },
    CancelMission {
        mission_id: String,
    },
    PerformInvestigation {
        action_id: String,
        method: String,
        expected_version: u32,
    },
}

impl PartyAction {
    pub(super) fn readiness(
        &self,
        actor_site: Option<&CaseSiteId>,
        party: &PartyView,
    ) -> PartyReadinessRequirement {
        match self {
            Self::TravelToSettlement { .. }
                if actor_site.is_some() && actor_site == party.current_case_site_id.as_ref() =>
            {
                PartyReadinessRequirement::Exempt
            }
            Self::TravelToSettlement { .. }
            | Self::TravelToCaseSite { .. }
            | Self::AutoresolveMission { .. }
            | Self::RequestTacticalServer { .. }
            | Self::PerformInvestigation { .. } => PartyReadinessRequirement::Required,
            _ => PartyReadinessRequirement::Exempt,
        }
    }

    fn kind(&self) -> PartyActionKind {
        match self {
            Self::TravelToSettlement { .. } | Self::TravelToCaseSite { .. } => {
                PartyActionKind::Travel
            }
            Self::RemovePartyMember { .. } => PartyActionKind::RemoveMember,
            Self::CreateRecruitmentRole { .. } => PartyActionKind::CreateRole,
            Self::UpdateRecruitmentRole { .. } => PartyActionKind::UpdateRole,
            Self::DeleteRecruitmentRole { .. } => PartyActionKind::DeleteRole,
            Self::AcceptJoinRequest { .. } => PartyActionKind::AcceptJoin,
            Self::RejectJoinRequest { .. } => PartyActionKind::RejectJoin,
            Self::AcceptContract { .. } => PartyActionKind::AcceptContract,
            Self::AbandonContract { .. } => PartyActionKind::AbandonContract,
            Self::ReportContract { .. } => PartyActionKind::ReportContract,
            Self::AutoresolveMission { .. } => PartyActionKind::Autoresolve,
            Self::UpdatePartyCheckTargets { .. } => PartyActionKind::CheckTargets,
            Self::SetInventoryQuantityTarget { .. } => PartyActionKind::InventoryTarget,
            Self::DisbandParty { .. } => PartyActionKind::Disband,
            Self::RequestTacticalServer { .. } => PartyActionKind::TacticalServer,
            Self::CancelMission { .. } => PartyActionKind::CancelMission,
            Self::PerformInvestigation { .. } => PartyActionKind::Investigation,
        }
    }

    pub(super) async fn execute(
        &self,
        actor_id: CharacterId,
        db: &SpacetimeClient,
    ) -> std::result::Result<(), PartyActionError> {
        let (reducer, mut args): (&str, Vec<Value>) = match self {
            Self::TravelToSettlement { settlement_id } => {
                ("travel_to_settlement", vec![json!(settlement_id)])
            }
            Self::TravelToCaseSite { case_site_id } => (
                "travel_to_case_site",
                vec![json!({ "value": case_site_id })],
            ),
            Self::RemovePartyMember { character_id } => {
                ("remove_party_member", vec![json!(character_id)])
            }
            Self::CreateRecruitmentRole {
                name,
                quantity,
                requirements,
                save_role,
            } => (
                "create_recruitment_role",
                vec![
                    json!(name),
                    json!(quantity),
                    json!(requirements),
                    json!(save_role),
                ],
            ),
            Self::UpdateRecruitmentRole {
                role_id,
                name,
                quantity,
                requirements,
            } => (
                "update_recruitment_role",
                vec![
                    json!(role_id),
                    json!(name),
                    json!(quantity),
                    json!(requirements),
                ],
            ),
            Self::DeleteRecruitmentRole { role_id } => {
                ("delete_recruitment_role", vec![json!(role_id)])
            }
            Self::AcceptJoinRequest { request_id } => {
                ("accept_party_join_request", vec![json!(request_id)])
            }
            Self::RejectJoinRequest { request_id } => {
                ("reject_party_join_request", vec![json!(request_id)])
            }
            Self::AcceptContract { contract_id } => ("accept_contract", vec![json!(contract_id)]),
            Self::AbandonContract { contract_id } => ("abandon_contract", vec![json!(contract_id)]),
            Self::ReportContract { contract_id } => ("report_contract", vec![json!(contract_id)]),
            Self::AutoresolveMission { mission_id } => {
                ("autoresolve_mission", vec![json!(mission_id)])
            }
            Self::UpdatePartyCheckTargets {
                physiology,
                command,
                religion,
            } => (
                "update_party_check_targets",
                vec![json!(physiology), json!(command), json!(religion)],
            ),
            Self::SetInventoryQuantityTarget { item_id, quantity } => (
                "set_inventory_quantity_target",
                vec![json!(true), json!(item_id), json!(quantity)],
            ),
            Self::DisbandParty { party_id } => ("disband_party", vec![json!(party_id)]),
            Self::RequestTacticalServer {
                mission_id,
                scene_key,
            } => (
                "request_tactical_server",
                vec![json!(mission_id), json!(scene_key)],
            ),
            Self::CancelMission { mission_id } => {
                ("cancel_mission_request", vec![json!(mission_id)])
            }
            Self::PerformInvestigation {
                action_id,
                method,
                expected_version,
            } => (
                "perform_investigation_action",
                vec![json!(action_id), json!(method), json!(expected_version)],
            ),
        };
        args.insert(0, json!(actor_id));
        db.call(reducer, &args)
            .await
            .map_err(|source: SpacetimeError| -> PartyActionError {
                PartyActionError::reducer(actor_id, source)
            })
    }
}

impl std::fmt::Display for PartyAction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary: String = match self {
            Self::TravelToSettlement { settlement_id, .. } => {
                format!("Travel to settlement {settlement_id}")
            }
            Self::TravelToCaseSite { .. } => "Travel to investigation site".into(),
            Self::RemovePartyMember { character_id } => {
                format!("Remove party member {character_id}")
            }
            Self::CreateRecruitmentRole { name, quantity, .. } => {
                format!("Add {quantity} {name} slot(s)")
            }
            Self::UpdateRecruitmentRole { name, .. } => format!("Edit recruitment role {name}"),
            Self::DeleteRecruitmentRole { .. } => "Delete a recruitment role".into(),
            Self::AcceptJoinRequest { request_id } => format!("Accept join request {request_id}"),
            Self::RejectJoinRequest { request_id } => format!("Reject join request {request_id}"),
            Self::AcceptContract { contract_id } => format!("Accept contract {contract_id}"),
            Self::AbandonContract { contract_id } => format!("Abandon contract {contract_id}"),
            Self::ReportContract { contract_id } => format!("Report contract {contract_id}"),
            Self::AutoresolveMission { mission_id } => {
                format!("Autoresolve mission {mission_id}")
            }
            Self::UpdatePartyCheckTargets { .. } => "Change party skill targets".into(),
            Self::SetInventoryQuantityTarget { .. } => "Manage party inventory targets".into(),
            Self::DisbandParty { .. } => "Disband the party".into(),
            Self::RequestTacticalServer { .. } => "Initiate tactical combat".into(),
            Self::CancelMission { .. } => "Cancel tactical combat".into(),
            Self::PerformInvestigation { method, .. } => {
                format!("Perform investigation action: {}", method.replace('_', " "))
            }
        };
        formatter.write_str(&summary)
    }
}

#[cfg(test)]
mod tests;
