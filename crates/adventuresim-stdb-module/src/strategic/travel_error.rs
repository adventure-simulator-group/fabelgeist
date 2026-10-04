//! Travel retains admission, synchronization and arrival causes until reducer presentation.

use adventuresim_core::{identity::IdentityError, strategic_place::PlaceIdentityError};

#[derive(Debug)]
pub(crate) enum TravelError {
    PendingEncounter(super::PendingEncounterError),
    Route(super::route_error::RouteAdmissionError),
    CampJourneyMissing,
    CampRouteMissing,
    CampPositionUnavailable,
    NotCampEndpoint,
    AlreadyAtJourneyEndpoint,

    OriginCoordinate,
    DestinationCoordinate,
    CharacterMissing,
    PartyRequired,
    PartyMissing,
    LeaderRequired,
    Camped,
    UndisclosedSite,
    AmbiguousPartyLocation,
    LeaderLocationMismatch,
    AlreadyAtSite,
    DestinationKnowledgeChanged,
    CurrentSettlementMissing,
    CurrentSettlementCoordinate,
    CurrentSiteMissing,
    CurrentSiteCoordinate,
    DestinationSiteCoordinate,
    PartyChanged,
    MemberMissing,
    SettlementMissing,
    EvacuationAuthority,
    CharacterSettlementMissing,
    DisconnectedSettlement,
    CharacterSiteMissing,
    UnknownLocation,
    OriginKind,
    Living(crate::character::LivingCharacterError),
    PartyReadiness(crate::strategic::PartyReadinessError),
    CharacterReadiness(crate::condition::CharacterReadinessError),
    Departure(super::departure::DepartureRevalidationError),
    Clock(crate::time::WorldClockError),
    DepartureClock(crate::time::DepartureClockError),
    Condition(crate::condition::StrategicConditionError),
    Capability(crate::capability::CapabilityEvaluationError),
    PartyIdentity(IdentityError),
    SettlementIdentity(IdentityError),
    CaseSiteIdentity(PlaceIdentityError),
}
impl From<crate::character::LivingCharacterError> for TravelError {
    fn from(source: crate::character::LivingCharacterError) -> Self {
        Self::Living(source)
    }
}
impl From<crate::strategic::PartyReadinessError> for TravelError {
    fn from(source: crate::strategic::PartyReadinessError) -> Self {
        Self::PartyReadiness(source)
    }
}
impl From<crate::condition::CharacterReadinessError> for TravelError {
    fn from(source: crate::condition::CharacterReadinessError) -> Self {
        Self::CharacterReadiness(source)
    }
}
impl From<super::departure::DepartureRevalidationError> for TravelError {
    fn from(source: super::departure::DepartureRevalidationError) -> Self {
        Self::Departure(source)
    }
}
impl From<super::PendingEncounterError> for TravelError {
    fn from(source: super::PendingEncounterError) -> Self {
        Self::PendingEncounter(source)
    }
}
impl From<super::route_error::RouteAdmissionError> for TravelError {
    fn from(source: super::route_error::RouteAdmissionError) -> Self {
        Self::Route(source)
    }
}
impl From<crate::time::DepartureClockError> for TravelError {
    fn from(source: crate::time::DepartureClockError) -> Self {
        Self::DepartureClock(source)
    }
}
impl From<crate::time::WorldClockError> for TravelError {
    fn from(source: crate::time::WorldClockError) -> Self {
        Self::Clock(source)
    }
}
impl From<crate::condition::StrategicConditionError> for TravelError {
    fn from(source: crate::condition::StrategicConditionError) -> Self {
        Self::Condition(source)
    }
}
impl From<crate::capability::CapabilityEvaluationError> for TravelError {
    fn from(source: crate::capability::CapabilityEvaluationError) -> Self {
        Self::Capability(source)
    }
}
impl std::fmt::Display for TravelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PendingEncounter(source) => source.fmt(f),
            Self::Route(source) => source.fmt(f),
            Self::CampJourneyMissing => f.write_str("Camp journey not found"),
            Self::CampRouteMissing => f.write_str("Camp has no persisted terrain route"),
            Self::CampPositionUnavailable => f.write_str("Camp route position is unavailable"),
            Self::NotCampEndpoint => f.write_str("That settlement is not an endpoint of this camp journey"),
            Self::AlreadyAtJourneyEndpoint => f.write_str("The party is already at that journey endpoint"),

            Self::OriginCoordinate => f.write_str("Journey origin is not a valid WGS84 coordinate"),
            Self::DestinationCoordinate => f.write_str("Journey destination is not a valid WGS84 coordinate"),
            Self::CharacterMissing => f.write_str("Character not found"),
            Self::PartyRequired => f.write_str("Must be in a party to travel to a case site"),
            Self::PartyMissing => f.write_str("Party not found"),
            Self::LeaderRequired => f.write_str("Only the party leader can travel"),
            Self::Camped => f.write_str("Break camp and continue the current journey first"),
            Self::UndisclosedSite => f.write_str("That exact site has not been disclosed to this observer"),
            Self::AmbiguousPartyLocation => f.write_str("Party must be at one authoritative location to travel"),
            Self::LeaderLocationMismatch => f.write_str("Party leader location does not match the party"),
            Self::AlreadyAtSite => f.write_str("The party is already at that case site"),
            Self::DestinationKnowledgeChanged => f.write_str("Exact destination knowledge changed during departure synchronization"),
            Self::CurrentSettlementMissing => f.write_str("Current settlement not found"),
            Self::CurrentSettlementCoordinate => f.write_str("Current settlement has an invalid WGS84 coordinate"),
            Self::CurrentSiteMissing => f.write_str("Current case site not found"),
            Self::CurrentSiteCoordinate => f.write_str("Current case site has an invalid WGS84 coordinate"),
            Self::DestinationSiteCoordinate => f.write_str("Destination case site has an invalid WGS84 coordinate"),
            Self::PartyChanged => f.write_str("Party changed during travel"),
            Self::MemberMissing => f.write_str("Party member not found"),
            Self::SettlementMissing => f.write_str("Settlement not found"),
            Self::EvacuationAuthority => f.write_str("Only the party leader, or a ready companion evacuating an unready leader, can travel"),
            Self::CharacterSettlementMissing => f.write_str("Character's current settlement does not exist"),
            Self::DisconnectedSettlement => f.write_str("That settlement is not directly connected by land or ferry"),
            Self::CharacterSiteMissing => f.write_str("Character's current case site does not exist"),
            Self::UnknownLocation => f.write_str("Character is not at a known location"),
            Self::OriginKind => f.write_str("Journey origin kind is invalid"),
            Self::Living(source) => source.fmt(f),
            Self::PartyReadiness(source) => source.fmt(f),
            Self::CharacterReadiness(source) => source.fmt(f),
            Self::Departure(source) => source.fmt(f),
            Self::Clock(source) => source.fmt(f),
            Self::DepartureClock(source) => source.fmt(f),
            Self::Condition(source) => source.fmt(f),
            Self::Capability(source) => source.fmt(f),
            Self::PartyIdentity(_) => f.write_str("Party not found"),
            Self::SettlementIdentity(_) => f.write_str("Settlement not found"),
            Self::CaseSiteIdentity(_) => f.write_str("Case-site identity is malformed"),
        }
    }
}
impl std::error::Error for TravelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PendingEncounter(source) => Some(source),
            Self::Route(source) => Some(source),
            Self::Living(source) => Some(source),
            Self::PartyReadiness(source) => Some(source),
            Self::CharacterReadiness(source) => Some(source),
            Self::Departure(source) => Some(source),
            Self::Clock(source) => Some(source),
            Self::DepartureClock(source) => Some(source),
            Self::Condition(source) => Some(source),
            Self::Capability(source) => Some(source),
            Self::PartyIdentity(source) | Self::SettlementIdentity(source) => Some(source),
            Self::CaseSiteIdentity(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{CharacterReadinessError, readiness_error::ReadinessParticipant};
    use adventuresim_core::identity::{CharacterId, SettlementId};
    use std::error::Error;

    #[test]
    fn travel_retains_departure_party_and_member_refusal_chain() {
        let member = CharacterId::from(u64::MAX);
        let error = TravelError::from(super::super::departure::DepartureRevalidationError::from(
            crate::strategic::PartyReadinessError::from(CharacterReadinessError::Incapacitated(
                ReadinessParticipant::PartyMember(member),
            )),
        ));
        let departure = error
            .source()
            .unwrap()
            .downcast_ref::<super::super::departure::DepartureRevalidationError>()
            .unwrap();
        let party = departure
            .source()
            .unwrap()
            .downcast_ref::<crate::strategic::PartyReadinessError>()
            .unwrap();
        assert!(
            matches!(party.source().unwrap().downcast_ref::<CharacterReadinessError>(),
            Some(CharacterReadinessError::Incapacitated(ReadinessParticipant::PartyMember(actual))) if *actual == member)
        );
        assert_eq!(
            error.to_string(),
            "A party member is incapacitated and must recover before acting"
        );
    }

    #[test]
    fn malformed_destination_preserves_identity_cause_and_native_refusal_text() {
        let cause = SettlementId::try_new("bad\nsettlement").unwrap_err();
        let error = TravelError::SettlementIdentity(cause);
        assert_eq!(
            error.source().unwrap().downcast_ref::<IdentityError>(),
            Some(&cause)
        );
        assert_eq!(error.to_string(), "Settlement not found");
    }
}
