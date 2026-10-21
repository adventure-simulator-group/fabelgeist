//! Decode and execute the action approved by a party leader.

use super::*;

#[derive(serde::Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub(super) enum ApprovedPartyAction {
    TravelToSettlement {
        settlement_id: String,
    },
    TravelToCaseSite {
        case_site_id: String,
    },
    RemovePartyMember {
        character_id: u64,
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

impl ApprovedPartyAction {
    pub(super) fn kind(&self) -> &'static str {
        match self {
            Self::TravelToSettlement { .. } | Self::TravelToCaseSite { .. } => "travel",
            Self::RemovePartyMember { .. } => "kick",
            Self::CreateRecruitmentRole { .. } => "add_role",
            Self::UpdateRecruitmentRole { .. } => "edit_role",
            Self::DeleteRecruitmentRole { .. } => "delete_role",
            Self::AcceptJoinRequest { .. } => "accept_join",
            Self::RejectJoinRequest { .. } => "reject_join",
            Self::AcceptContract { .. } => "accept_contract",
            Self::AbandonContract { .. } => "abandon_contract",
            Self::ReportContract { .. } => "report_contract",
            Self::AutoresolveMission { .. } => "autoresolve",
            Self::UpdatePartyCheckTargets { .. } => "party_checks",
            Self::SetInventoryQuantityTarget { .. } => "party_inventory",
            Self::DisbandParty { .. } => "disband_party",
            Self::RequestTacticalServer { .. } => "initiate_combat",
            Self::CancelMission { .. } => "cancel_mission",
            Self::PerformInvestigation { .. } => "investigate",
        }
    }

    pub(super) fn execute(
        self,
        ctx: &ReducerContext,
        leader_id: u64,
        requester_id: u64,
    ) -> StrategicCombatResult<()> {
        let result = match self {
            Self::TravelToSettlement { settlement_id } => {
                travel_to_settlement(ctx, leader_id, settlement_id)
            }
            Self::TravelToCaseSite { case_site_id } => {
                travel_to_case_site(ctx, leader_id, CaseSiteId::from(case_site_id))
            }
            Self::RemovePartyMember { character_id } => {
                remove_party_member(ctx, leader_id, character_id)
            }
            Self::CreateRecruitmentRole {
                name,
                quantity,
                requirements,
                save_role,
            } => create_recruitment_role(ctx, leader_id, name, quantity, requirements, save_role),
            Self::UpdateRecruitmentRole {
                role_id,
                name,
                quantity,
                requirements,
            } => update_recruitment_role(ctx, leader_id, role_id, name, quantity, requirements),
            Self::DeleteRecruitmentRole { role_id } => {
                delete_recruitment_role(ctx, leader_id, role_id)
            }
            Self::AcceptJoinRequest { request_id } => {
                accept_party_join_request(ctx, leader_id, request_id)
            }
            Self::RejectJoinRequest { request_id } => {
                reject_party_join_request(ctx, leader_id, request_id)
            }
            Self::AcceptContract { contract_id } => accept_contract(ctx, leader_id, contract_id),
            Self::AbandonContract { contract_id } => abandon_contract(ctx, leader_id, contract_id),
            Self::ReportContract { contract_id } => report_contract(ctx, leader_id, contract_id),
            Self::AutoresolveMission { mission_id } => {
                return autoresolve_mission(ctx, leader_id, mission_id);
            }
            Self::UpdatePartyCheckTargets {
                physiology,
                command,
                religion,
            } => update_party_check_targets(ctx, leader_id, physiology, command, religion),
            Self::SetInventoryQuantityTarget { item_id, quantity } => {
                set_inventory_quantity_target(ctx, leader_id, true, item_id, quantity)
            }
            Self::DisbandParty { party_id } => disband_party(ctx, leader_id, party_id),
            Self::RequestTacticalServer {
                mission_id,
                scene_key,
            } => crate::tactical::request_tactical_server(ctx, leader_id, mission_id, scene_key),
            Self::CancelMission { mission_id } => {
                cancel_mission_request(ctx, leader_id, mission_id)
            }
            Self::PerformInvestigation {
                action_id,
                method,
                expected_version,
            } => crate::investigation::perform_investigation_action_authorized(
                ctx,
                requester_id,
                action_id,
                method,
                expected_version,
                true,
            ),
        };
        result.map_err(StrategicCombatError::from)
    }
}

