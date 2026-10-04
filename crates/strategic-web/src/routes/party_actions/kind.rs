//! Stable storage classification of an approval request.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) enum PartyActionKind {
    #[serde(rename = "travel")]
    Travel,
    #[serde(rename = "kick")]
    RemoveMember,
    #[serde(rename = "add_role")]
    CreateRole,
    #[serde(rename = "edit_role")]
    UpdateRole,
    #[serde(rename = "delete_role")]
    DeleteRole,
    #[serde(rename = "accept_join")]
    AcceptJoin,
    #[serde(rename = "reject_join")]
    RejectJoin,
    #[serde(rename = "accept_contract")]
    AcceptContract,
    #[serde(rename = "abandon_contract")]
    AbandonContract,
    #[serde(rename = "report_contract")]
    ReportContract,
    #[serde(rename = "autoresolve")]
    Autoresolve,
    #[serde(rename = "party_checks")]
    CheckTargets,
    #[serde(rename = "party_inventory")]
    InventoryTarget,
    #[serde(rename = "disband_party")]
    Disband,
    #[serde(rename = "initiate_combat")]
    TacticalServer,
    #[serde(rename = "cancel_mission")]
    CancelMission,
    #[serde(rename = "investigate")]
    Investigation,
}
impl std::fmt::Display for PartyActionKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Travel => "travel",
            Self::RemoveMember => "kick",
            Self::CreateRole => "add_role",
            Self::UpdateRole => "edit_role",
            Self::DeleteRole => "delete_role",
            Self::AcceptJoin => "accept_join",
            Self::RejectJoin => "reject_join",
            Self::AcceptContract => "accept_contract",
            Self::AbandonContract => "abandon_contract",
            Self::ReportContract => "report_contract",
            Self::Autoresolve => "autoresolve",
            Self::CheckTargets => "party_checks",
            Self::InventoryTarget => "party_inventory",
            Self::Disband => "disband_party",
            Self::TacticalServer => "initiate_combat",
            Self::CancelMission => "cancel_mission",
            Self::Investigation => "investigate",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::PartyActionKind;
    #[test]
    fn storage_tokens_round_trip_and_unknown_kinds_are_rejected() {
        for kind in [
            PartyActionKind::Travel,
            PartyActionKind::RemoveMember,
            PartyActionKind::CreateRole,
            PartyActionKind::UpdateRole,
            PartyActionKind::DeleteRole,
            PartyActionKind::AcceptJoin,
            PartyActionKind::RejectJoin,
            PartyActionKind::AcceptContract,
            PartyActionKind::AbandonContract,
            PartyActionKind::ReportContract,
            PartyActionKind::Autoresolve,
            PartyActionKind::CheckTargets,
            PartyActionKind::InventoryTarget,
            PartyActionKind::Disband,
            PartyActionKind::TacticalServer,
            PartyActionKind::CancelMission,
            PartyActionKind::Investigation,
        ] {
            let wire = serde_json::to_value(kind).unwrap();
            assert_eq!(wire, serde_json::json!(kind.to_string()));
            assert_eq!(
                serde_json::from_value::<PartyActionKind>(wire).unwrap(),
                kind
            );
        }
        for invalid in [
            serde_json::json!("Travel"),
            serde_json::json!("unknown"),
            serde_json::json!(7),
        ] {
            assert!(serde_json::from_value::<PartyActionKind>(invalid).is_err());
        }
    }
}
