//! Shared lifecycle discriminants for storage rows and validated state parsers.
//!
//! Parsed states add lifecycle invariants; these enums own their vocabulary.
//! Generated SDK enums remain wire types and are converted at client boundaries.

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum MissionAttemptStatus {
    Bound,
    Committed,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum HostileResolutionKind {
    Defeated,
    DrivenOff,
    Surrendered,
    Captured,
    CaptureTargetKilled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum CommitmentStatus {
    Reserved,
    Fulfilled,
    Cancelled,
    Expired,
    Ended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum CommitmentTerminalReason {
    WeddingCompleted,
    ParticipantDead,
    ParticipantUnderage,
    CeremonyLocationUnavailable,
    ResidenceUnavailable,
    CancelledByParticipant,
    ReservationExpired,
    MarriageEnded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum CourtshipKind {
    Formal,
    Informal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum CourtshipStatus {
    Active,
    Exposed,
    Ended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum CourtshipSecrecyReason {
    FatherDisapproval,
    FormalRouteUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum CourtshipTerminalReason {
    EngagementScheduled,
    EndedByParticipant,
    PartnerUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum MarriageStatus {
    Active,
    Widowed,
    Ended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum PregnancyStatus {
    Active,
    Born,
    Ended,
}

impl CommitmentStatus {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Reserved => "Reserved",
            Self::Fulfilled => "Fulfilled",
            Self::Cancelled => "Cancelled",
            Self::Expired => "Expired",
            Self::Ended => "Ended",
        }
    }
}
