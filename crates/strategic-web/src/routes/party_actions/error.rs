//! Failures retain their operation, identity, and original provider cause.
use super::{
    super::{coordinates::CoordinateAdmissionError, party_readiness::PartyReadinessError},
    location::CaseSiteObservationError,
};
use crate::spacetimedb::SpacetimeError;
use adventuresim_core::identity::{CharacterId, IdentityError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PartyActionStage {
    Actor,
    Party,
    Execute,
    Request,
    TemporaryLeader,
    Approve,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PartyActionError {
    #[error("{source}")]
    Database {
        stage: PartyActionStage,
        actor: CharacterId,
        #[source]
        source: SpacetimeError,
    },
    #[error("Character not found")]
    MissingActor(CharacterId),
    #[error("Character has no party")]
    NoParty(CharacterId),
    #[error("Party not found")]
    MissingParty(CharacterId),
    #[error("{source}")]
    PartyIdentity {
        actor: CharacterId,
        #[source]
        source: IdentityError,
    },
    #[error("{source}")]
    Payload {
        actor: CharacterId,
        #[source]
        source: serde_json::Error,
    },
    #[error("{0}")]
    CaseSite(#[from] CaseSiteObservationError),
    #[error("{0}")]
    Readiness(#[from] PartyReadinessError),
    #[error("{0}")]
    Travel(#[from] PartyTravelError),
}

impl PartyActionError {
    pub(in crate::routes) fn reducer(actor: CharacterId, source: SpacetimeError) -> Self {
        Self::Database {
            stage: PartyActionStage::Execute,
            actor,
            source,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TerrainProfileStage {
    Membership,
    ObservedMember,
    Attributes,
    Limbs,
    Skills,
}

#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub(crate) struct PartyTerrainProfileError {
    pub(super) stage: TerrainProfileStage,
    pub(super) member: Option<CharacterId>,
    #[source]
    pub(super) source: SpacetimeError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DepartureQueryStage {
    Membership,
    ObservedMember,
    MemberClock,
}

#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub(crate) struct PartyDepartureError {
    pub(super) stage: DepartureQueryStage,
    pub(super) member: Option<CharacterId>,
    #[source]
    pub(super) source: SpacetimeError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TravelQueryStage {
    Actor,
    DestinationSettlement,
    DestinationCaseSite,
    OriginSettlement,
    OriginCaseSite,
    CampJourney,
    CampRoute,
    Execute,
    Approve,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PartyTravelError {
    #[error("{source}")]
    Query {
        stage: TravelQueryStage,
        actor: CharacterId,
        #[source]
        source: SpacetimeError,
    },
    #[error("Character not found")]
    MissingActor(CharacterId),
    #[error("Settlement not found")]
    MissingDestinationSettlement,
    #[error("Known exact case site not found")]
    MissingDestinationCaseSite(CharacterId),
    #[error("Origin settlement not found")]
    MissingOriginSettlement,
    #[error("Known exact origin case site not found")]
    MissingOriginCaseSite(CharacterId),
    #[error("Camp journey not found")]
    MissingCampJourney,
    #[error("Camp terrain route not found")]
    MissingCampRoute,
    #[error("Camp terrain route position is unavailable")]
    UnavailableCampPosition,
    #[error("travel origin is outside WGS84 bounds")]
    InvalidOriginCoordinates,
    #[error("{0}")]
    Coordinate(#[from] CoordinateAdmissionError),
    #[error("{0}")]
    CaseSite(#[from] CaseSiteObservationError),
    #[error("{0}")]
    Profile(#[from] PartyTerrainProfileError),
    #[error("{0}")]
    Departure(#[from] PartyDepartureError),
}

impl PartyTravelError {
    pub(super) fn query(
        stage: TravelQueryStage,
        actor: CharacterId,
        source: SpacetimeError,
    ) -> Self {
        Self::Query {
            stage,
            actor,
            source,
        }
    }
}
impl PartyTerrainProfileError {
    pub(super) fn query(
        stage: TerrainProfileStage,
        member: Option<CharacterId>,
        source: SpacetimeError,
    ) -> Self {
        Self {
            stage,
            member,
            source,
        }
    }
}
impl PartyDepartureError {
    pub(super) fn query(
        stage: DepartureQueryStage,
        member: Option<CharacterId>,
        source: SpacetimeError,
    ) -> Self {
        Self {
            stage,
            member,
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spacetimedb::{DatabaseOperation, RemoteDatabaseFailure};
    use std::error::Error as _;

    #[test]
    fn nested_travel_failure_retains_stage_member_and_provider_cause() {
        let member = CharacterId::from(u64::MAX);
        let source = SpacetimeError::Remote(RemoteDatabaseFailure::from_response(
            DatabaseOperation::Query,
            reqwest::StatusCode::SERVICE_UNAVAILABLE,
            Ok("member skills unavailable".into()),
        ));
        let notice = source.to_string();
        let error = PartyActionError::from(PartyTravelError::from(PartyTerrainProfileError {
            stage: TerrainProfileStage::Skills,
            member: Some(member),
            source,
        }));
        assert_eq!(error.to_string(), notice);
        let travel = error
            .source()
            .unwrap()
            .downcast_ref::<PartyTravelError>()
            .unwrap();
        let profile = travel
            .source()
            .unwrap()
            .downcast_ref::<PartyTerrainProfileError>()
            .unwrap();
        assert_eq!(profile.stage, TerrainProfileStage::Skills);
        assert_eq!(profile.member, Some(member));
        assert!(
            profile
                .source()
                .unwrap()
                .downcast_ref::<SpacetimeError>()
                .is_some()
        );
    }

    #[test]
    fn missing_resources_and_readiness_keep_the_existing_notices() {
        let actor = CharacterId::from(7);
        for (error, notice) in [
            (PartyActionError::MissingActor(actor), "Character not found"),
            (PartyActionError::NoParty(actor), "Character has no party"),
            (PartyActionError::MissingParty(actor), "Party not found"),
            (
                PartyActionError::from(PartyTravelError::MissingOriginSettlement),
                "Origin settlement not found",
            ),
            (
                PartyActionError::from(PartyTravelError::UnavailableCampPosition),
                "Camp terrain route position is unavailable",
            ),
            (
                PartyActionError::from(PartyReadinessError::Incapacitated(actor)),
                "An incapacitated party member must recover before the party can act",
            ),
        ] {
            assert_eq!(error.to_string(), notice);
        }
    }
}
