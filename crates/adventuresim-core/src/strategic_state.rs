//! Parsed domain states for flattened strategic persistence rows.
//!
//! SpacetimeDB rows remain flat for queryability. Callers should construct these
//! types at the storage boundary so contradictory option/status combinations do
//! not escape into reducer logic.

use adventuresim_world_schema::calendar::StrategicMinute;

pub mod vocabulary;

use crate::{case::ContractStatus, identity::PartyId, strategic_place::CaseSiteId};
use vocabulary::{
    CommitmentStatus, CommitmentTerminalReason, CourtshipKind, CourtshipSecrecyReason,
    CourtshipStatus, CourtshipTerminalReason, HostileResolutionKind, MarriageStatus,
    MissionAttemptStatus, PregnancyStatus,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContractState {
    Offered,
    Accepted {
        party_id: PartyId,
        accepted_at: StrategicMinute,
    },
    ReadyToReport {
        party_id: PartyId,
        accepted_at: StrategicMinute,
    },
    Paid {
        party_id: PartyId,
        accepted_at: StrategicMinute,
        paid_at: StrategicMinute,
    },
    Withdrawn {
        prior_acceptance: Option<ContractAcceptance>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractAcceptance {
    pub party_id: PartyId,
    pub accepted_at: StrategicMinute,
}

impl ContractState {
    pub fn parse(
        status: ContractStatus,
        party_id: Option<String>,
        accepted_at: Option<StrategicMinute>,
        paid_at: Option<StrategicMinute>,
    ) -> Result<Self, StateParseError> {
        let party_id = party_id
            .map(PartyId::try_new)
            .transpose()
            .map_err(StateParseError::PartyIdentity)?;
        match (status, party_id, accepted_at, paid_at) {
            (ContractStatus::Offered, None, None, None) => Ok(Self::Offered),
            (ContractStatus::Accepted, Some(party_id), Some(accepted_at), None) => {
                Ok(Self::Accepted {
                    party_id,
                    accepted_at,
                })
            }
            (ContractStatus::ReadyToReport, Some(party_id), Some(accepted_at), None) => {
                Ok(Self::ReadyToReport {
                    party_id,
                    accepted_at,
                })
            }
            (ContractStatus::Paid, Some(party_id), Some(accepted_at), Some(paid_at))
                if paid_at >= accepted_at =>
            {
                Ok(Self::Paid {
                    party_id,
                    accepted_at,
                    paid_at,
                })
            }
            (ContractStatus::Withdrawn, None, None, None) => Ok(Self::Withdrawn {
                prior_acceptance: None,
            }),
            (ContractStatus::Withdrawn, Some(party_id), Some(accepted_at), None) => {
                Ok(Self::Withdrawn {
                    prior_acceptance: Some(ContractAcceptance {
                        party_id,
                        accepted_at,
                    }),
                })
            }
            _ => Err(StateParseError::ContractLifecycle),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionBinding {
    pub case_site_id: CaseSiteId,
    pub hostile_group_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostileResolution {
    Defeated,
    DrivenOff,
    Surrendered,
    Captured {
        subject_id: String,
        custody_version: CustodyVersion,
    },
    CaptureTargetKilled {
        subject_id: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CustodyVersion(u32);

impl CustodyVersion {
    pub fn new(version: u32) -> Self {
        Self(version)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MissionAttemptState {
    Bound {
        binding: Option<MissionBinding>,
    },
    Committed {
        binding: Option<MissionBinding>,
        resolution: HostileResolution,
    },
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatMissionState {
    pub status: MissionAttemptStatus,
    pub case_site_id: Option<CaseSiteId>,
    pub hostile_group_id: Option<String>,
    pub resolution: Option<HostileResolutionKind>,
    pub subject_id: Option<String>,
    pub custody_version: Option<u32>,
}

impl MissionAttemptState {
    pub fn parse(flat: FlatMissionState) -> Result<Self, StateParseError> {
        let FlatMissionState {
            status,
            case_site_id,
            hostile_group_id,
            resolution,
            subject_id,
            custody_version,
        } = flat;
        let binding = match (case_site_id, hostile_group_id) {
            (None, None) => None,
            (Some(case_site_id), Some(hostile_group_id)) => Some(MissionBinding {
                case_site_id,
                hostile_group_id,
            }),
            _ => {
                return Err(StateParseError::IncompleteMissionBinding);
            }
        };
        match (status, resolution, subject_id, custody_version) {
            (MissionAttemptStatus::Bound, None, None, None) => Ok(Self::Bound { binding }),
            (
                MissionAttemptStatus::Committed,
                Some(HostileResolutionKind::Defeated),
                None,
                None,
            ) => Ok(Self::Committed {
                binding,
                resolution: HostileResolution::Defeated,
            }),
            (
                MissionAttemptStatus::Committed,
                Some(HostileResolutionKind::DrivenOff),
                None,
                None,
            ) => Ok(Self::Committed {
                binding,
                resolution: HostileResolution::DrivenOff,
            }),
            (
                MissionAttemptStatus::Committed,
                Some(HostileResolutionKind::Surrendered),
                None,
                None,
            ) => Ok(Self::Committed {
                binding,
                resolution: HostileResolution::Surrendered,
            }),
            (
                MissionAttemptStatus::Committed,
                Some(HostileResolutionKind::Captured),
                Some(subject_id),
                Some(version),
            ) => Ok(Self::Committed {
                binding,
                resolution: HostileResolution::Captured {
                    subject_id,
                    custody_version: CustodyVersion::new(version),
                },
            }),
            (
                MissionAttemptStatus::Committed,
                Some(HostileResolutionKind::CaptureTargetKilled),
                Some(subject_id),
                None,
            ) => Ok(Self::Committed {
                binding,
                resolution: HostileResolution::CaptureTargetKilled { subject_id },
            }),
            (MissionAttemptStatus::Failed, None, None, None) => Ok(Self::Failed),
            (MissionAttemptStatus::Cancelled, None, None, None) => Ok(Self::Cancelled),
            _ => Err(StateParseError::MissionResolution),
        }
    }
}

mod error;
pub use error::StateParseError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitmentState {
    Reserved {
        effective_minute: StrategicMinute,
    },
    Fulfilled {
        resolved_minute: StrategicMinute,
    },
    Cancelled {
        resolved_minute: StrategicMinute,
        reason: CommitmentTerminalReason,
    },
    Expired {
        resolved_minute: StrategicMinute,
    },
    Ended {
        resolved_minute: StrategicMinute,
    },
}

impl CommitmentState {
    pub fn parse(
        status: CommitmentStatus,
        effective_minute: StrategicMinute,
        resolved_minute: Option<StrategicMinute>,
        reason: Option<CommitmentTerminalReason>,
    ) -> Result<Self, StateParseError> {
        match (status, resolved_minute, reason) {
            (CommitmentStatus::Reserved, None, None) => Ok(Self::Reserved { effective_minute }),
            (
                CommitmentStatus::Fulfilled,
                Some(resolved_minute),
                Some(CommitmentTerminalReason::WeddingCompleted),
            ) => Ok(Self::Fulfilled { resolved_minute }),
            (
                CommitmentStatus::Cancelled,
                Some(resolved_minute),
                Some(
                    reason @ (CommitmentTerminalReason::ParticipantDead
                    | CommitmentTerminalReason::ParticipantUnderage
                    | CommitmentTerminalReason::ResidenceUnavailable
                    | CommitmentTerminalReason::CeremonyLocationUnavailable
                    | CommitmentTerminalReason::CancelledByParticipant),
                ),
            ) => Ok(Self::Cancelled {
                resolved_minute,
                reason,
            }),
            (
                CommitmentStatus::Expired,
                Some(resolved_minute),
                Some(CommitmentTerminalReason::ReservationExpired),
            ) => Ok(Self::Expired { resolved_minute }),
            (
                CommitmentStatus::Ended,
                Some(resolved_minute),
                Some(CommitmentTerminalReason::MarriageEnded),
            ) => Ok(Self::Ended { resolved_minute }),
            _ => Err(StateParseError::CommitmentLifecycle),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CourtshipRoute {
    Formal {
        approved_father_id: u64,
        planned_dowry: u32,
    },
    Informal {
        secrecy_reason: CourtshipSecrecyReason,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CourtshipState {
    Active,
    Exposed,
    Ended {
        resolved_minute: StrategicMinute,
        reason: CourtshipTerminalReason,
    },
}

pub fn parse_courtship(
    kind: CourtshipKind,
    secrecy_reason: Option<CourtshipSecrecyReason>,
    approved_father_id: Option<u64>,
    planned_dowry: u32,
    status: CourtshipStatus,
    resolved_minute: Option<StrategicMinute>,
    terminal_reason: Option<CourtshipTerminalReason>,
) -> Result<(CourtshipRoute, CourtshipState), StateParseError> {
    let route = match (kind, secrecy_reason, approved_father_id) {
        (CourtshipKind::Formal, None, Some(approved_father_id)) => CourtshipRoute::Formal {
            approved_father_id,
            planned_dowry,
        },
        (CourtshipKind::Informal, Some(secrecy_reason), None) if planned_dowry == 0 => {
            CourtshipRoute::Informal { secrecy_reason }
        }
        _ => return Err(StateParseError::CourtshipRoute),
    };
    let state = match (status, resolved_minute, terminal_reason) {
        (CourtshipStatus::Active, None, None) => CourtshipState::Active,
        (CourtshipStatus::Exposed, None, None) => CourtshipState::Exposed,
        (CourtshipStatus::Ended, Some(resolved_minute), Some(reason)) => CourtshipState::Ended {
            resolved_minute,
            reason,
        },
        _ => {
            return Err(StateParseError::CourtshipLifecycle);
        }
    };
    Ok((route, state))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarriageState {
    Active,
    Widowed { resolved_minute: StrategicMinute },
    Ended { resolved_minute: StrategicMinute },
}

impl MarriageState {
    pub fn parse(
        status: MarriageStatus,
        resolved_minute: Option<StrategicMinute>,
    ) -> Result<Self, StateParseError> {
        match (status, resolved_minute) {
            (MarriageStatus::Active, None) => Ok(Self::Active),
            (MarriageStatus::Widowed, Some(resolved_minute)) => {
                Ok(Self::Widowed { resolved_minute })
            }
            (MarriageStatus::Ended, Some(resolved_minute)) => Ok(Self::Ended { resolved_minute }),
            _ => Err(StateParseError::MarriageLifecycle),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PregnancyState {
    Active,
    Born {
        child_id: u64,
        resolved_minute: StrategicMinute,
    },
    Ended {
        resolved_minute: StrategicMinute,
    },
}

impl PregnancyState {
    pub fn parse(
        status: PregnancyStatus,
        child_id: Option<u64>,
        resolved_minute: Option<StrategicMinute>,
    ) -> Result<Self, StateParseError> {
        match (status, child_id, resolved_minute) {
            (PregnancyStatus::Active, None, None) => Ok(Self::Active),
            (PregnancyStatus::Born, Some(child_id), Some(resolved_minute)) => Ok(Self::Born {
                child_id,
                resolved_minute,
            }),
            (PregnancyStatus::Ended, None, Some(resolved_minute)) => {
                Ok(Self::Ended { resolved_minute })
            }
            _ => Err(StateParseError::PregnancyOutcome),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_serialized_statuses_obey_the_same_lifecycle_contract() {
        let status: ContractStatus = serde_json::from_str("\"Paid\"").unwrap();
        assert!(
            ContractState::parse(
                status,
                Some("party".into()),
                Some(StrategicMinute::new(10)),
                Some(StrategicMinute::new(11))
            )
            .is_ok()
        );
        assert!(
            ContractState::parse(
                status,
                Some("party".into()),
                Some(StrategicMinute::new(10)),
                None
            )
            .is_err()
        );
        let status: MissionAttemptStatus = serde_json::from_str("\"Committed\"").unwrap();
        let resolution: HostileResolutionKind = serde_json::from_str("\"Captured\"").unwrap();
        let mut flat = FlatMissionState {
            status,
            case_site_id: None,
            hostile_group_id: None,
            resolution: Some(resolution),
            subject_id: Some("subject".into()),
            custody_version: Some(3),
        };
        assert!(MissionAttemptState::parse(flat.clone()).is_ok());
        flat.custody_version = None;
        assert!(MissionAttemptState::parse(flat).is_err());
        assert_eq!(
            serde_json::to_string(&CourtshipKind::Informal).unwrap(),
            "\"Informal\""
        );
        assert_eq!(
            serde_json::to_string(&CommitmentTerminalReason::WeddingCompleted).unwrap(),
            "\"WeddingCompleted\""
        );
    }
    #[test]
    fn contract_rejects_partial_acceptance_and_backwards_payment() {
        assert!(
            ContractState::parse(ContractStatus::Accepted, Some("p".into()), None, None).is_err()
        );
        assert!(
            ContractState::parse(
                ContractStatus::Paid,
                Some("p".into()),
                Some(StrategicMinute::new(10)),
                Some(StrategicMinute::new(9))
            )
            .is_err()
        );
    }
    #[test]
    fn mission_requires_complete_binding_and_capture_payload() {
        assert!(
            MissionAttemptState::parse(FlatMissionState {
                status: MissionAttemptStatus::Bound,
                case_site_id: Some(CaseSiteId::from("s".to_owned())),
                hostile_group_id: None,
                resolution: None,
                subject_id: None,
                custody_version: None
            })
            .is_err()
        );
        assert!(
            MissionAttemptState::parse(FlatMissionState {
                status: MissionAttemptStatus::Committed,
                case_site_id: Some(CaseSiteId::from("s".to_owned())),
                hostile_group_id: Some("h".into()),
                resolution: Some(HostileResolutionKind::Captured),
                subject_id: Some("target".into()),
                custody_version: None
            })
            .is_err()
        );
        assert!(
            MissionAttemptState::parse(FlatMissionState {
                status: MissionAttemptStatus::Committed,
                case_site_id: Some(CaseSiteId::from("s".to_owned())),
                hostile_group_id: Some("h".into()),
                resolution: Some(HostileResolutionKind::Captured),
                subject_id: Some("target".into()),
                custody_version: Some(0)
            })
            .is_ok()
        );
    }
    #[test]
    fn commitment_terminal_fields_are_all_or_nothing() {
        assert!(
            CommitmentState::parse(
                CommitmentStatus::Reserved,
                StrategicMinute::new(10),
                Some(StrategicMinute::new(11)),
                None,
            )
            .is_err()
        );
        assert!(
            CommitmentState::parse(
                CommitmentStatus::Cancelled,
                StrategicMinute::new(10),
                Some(StrategicMinute::new(11)),
                Some(CommitmentTerminalReason::CancelledByParticipant)
            )
            .is_ok()
        );
        assert!(
            CommitmentState::parse(
                CommitmentStatus::Fulfilled,
                StrategicMinute::new(10),
                Some(StrategicMinute::new(11)),
                Some(CommitmentTerminalReason::ParticipantDead)
            )
            .is_err()
        );
    }

    #[test]
    fn relationship_states_reject_partial_routes_and_outcomes() {
        assert!(
            parse_courtship(
                CourtshipKind::Formal,
                None,
                None,
                10,
                CourtshipStatus::Active,
                None,
                None
            )
            .is_err()
        );
        assert!(
            MarriageState::parse(MarriageStatus::Active, Some(StrategicMinute::new(1))).is_err()
        );
        assert!(
            PregnancyState::parse(PregnancyStatus::Born, None, Some(StrategicMinute::new(10)),)
                .is_err()
        );
        assert!(PregnancyState::parse(PregnancyStatus::Active, None, None).is_ok());
    }
}
