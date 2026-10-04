//! Location resolution retains the failed authority read and provider cause.

use super::super::coordinates::CoordinateAdmissionError;
use crate::spacetimedb::SpacetimeError;
use adventuresim_core::identity::CharacterId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::routes) enum VicinityReadStage {
    Settlement,
    CaseSite,
    Party,
    Journey,
    Route,
}

#[derive(Debug, thiserror::Error)]
pub(in crate::routes) enum VicinityError {
    #[error("{source}")]
    Database {
        stage: VicinityReadStage,
        actor: CharacterId,
        #[source]
        source: SpacetimeError,
    },
    #[error("Current settlement not found")]
    MissingSettlement,
    #[error("persisted settlement coordinate is outside WGS84 bounds")]
    InvalidSettlementCoordinate,
    #[error("Current case site is not exact")]
    InexactCaseSite,
    #[error("{0}")]
    Coordinate(#[from] CoordinateAdmissionError),
    #[error("Character has no stationary vicinity")]
    NoStationaryVicinity,
    #[error("Party not found")]
    MissingParty,
    #[error("Foraging is unavailable while moving or without a known location")]
    MovingOrUnknownLocation,
    #[error("Camp journey not found")]
    MissingJourney,
    #[error("Camp terrain route not found")]
    MissingRoute,
    #[error("Camp terrain position is unavailable")]
    UnavailableCampPosition,
}

impl VicinityError {
    pub(super) fn query(
        stage: VicinityReadStage,
        actor: CharacterId,
        source: SpacetimeError,
    ) -> Self {
        Self::Database {
            stage,
            actor,
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn persisted_coordinate_failures_keep_their_classification_and_notice() {
        let error = VicinityError::from(CoordinateAdmissionError::Persisted);
        assert!(matches!(
            &error,
            VicinityError::Coordinate(CoordinateAdmissionError::Persisted)
        ));
        assert!(error.source().unwrap().is::<CoordinateAdmissionError>());
        assert_eq!(
            error.to_string(),
            "persisted coordinate is outside WGS84 bounds"
        );
        assert_eq!(
            VicinityError::InvalidSettlementCoordinate.to_string(),
            "persisted settlement coordinate is outside WGS84 bounds"
        );
    }
}
