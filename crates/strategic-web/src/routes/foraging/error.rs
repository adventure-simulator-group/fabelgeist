//! Forage hydration and submission retain their own stage and concrete cause.

use super::super::{
    coordinates::CoordinateAdmissionError, travel::TerrainForageError, vicinity::VicinityError,
};
use super::{notice::ForageFeedback, receipt::ForageReceiptError};
use crate::spacetimedb::SpacetimeError;
use adventuresim_core::identity::CharacterId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ForageReadStage {
    Actor,
    Generation,
    Execute,
    Receipt,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum ForageRouteError {
    #[error("{source}")]
    Database {
        stage: ForageReadStage,
        actor: CharacterId,
        #[source]
        source: SpacetimeError,
    },
    #[error("Character not found")]
    MissingActor(CharacterId),
    #[error("{0}")]
    Vicinity(#[from] VicinityError),
    #[error("Terrain data is unavailable")]
    TerrainUnavailable,
    #[error("{0}")]
    Terrain(#[from] TerrainForageError),
    #[error("Foraging is unavailable on open water")]
    OpenWater,
    #[error("{0}")]
    Coordinate(#[from] CoordinateAdmissionError),
    #[error("{0}")]
    Attestation(#[from] serde_json::Error),
    #[error("Foraging completed but its result is not visible yet")]
    ReceiptUnavailable,
    #[error("Invalid forage receipt for {actor}: {source}")]
    Receipt {
        actor: CharacterId,
        #[source]
        source: ForageReceiptError,
    },
}

impl ForageRouteError {
    pub(super) fn database(
        stage: ForageReadStage,
        actor: CharacterId,
        source: SpacetimeError,
    ) -> Self {
        Self::Database {
            stage,
            actor,
            source,
        }
    }

    pub(super) fn feedback(&self) -> ForageFeedback {
        ForageFeedback::Unavailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn receipt_outcome_keeps_actor_and_decode_cause_without_feedback_leakage() {
        let source =
            adventuresim_core::foraging::ForagePublicLegalOutcome::try_from("lawful").unwrap_err();
        let error = ForageRouteError::Receipt {
            actor: 7.into(),
            source: source.into(),
        };
        let ForageRouteError::Receipt { actor, .. } = &error else {
            panic!("receipt outcome must retain its selected actor");
        };
        assert_eq!(*actor, CharacterId::from(7));
        let receipt = error
            .source()
            .unwrap()
            .downcast_ref::<ForageReceiptError>()
            .unwrap();
        assert!(
            receipt
                .source()
                .unwrap()
                .is::<adventuresim_core::foraging::ForageLegalOutcomeError>()
        );
        assert!(error.to_string().contains("lawful"));
        assert_eq!(error.feedback(), ForageFeedback::Unavailable);
        assert!(!error.feedback().message().contains("lawful"));
    }

    #[test]
    fn terrain_failure_preserves_nested_provider_cause_without_feedback_leakage() {
        let error = ForageRouteError::from(TerrainForageError::Sample(
            adventuresim_terrain::Error::Io(std::io::Error::other("private terrain source")),
        ));
        let sampling = error
            .source()
            .unwrap()
            .downcast_ref::<TerrainForageError>()
            .unwrap();
        let terrain = sampling
            .source()
            .unwrap()
            .downcast_ref::<adventuresim_terrain::Error>()
            .unwrap();
        assert!(terrain.source().unwrap().is::<std::io::Error>());
        assert_eq!(error.feedback(), ForageFeedback::Unavailable);
        assert!(
            !error
                .feedback()
                .message()
                .contains("private terrain source")
        );
    }

    #[test]
    fn receipt_read_keeps_selected_character_and_read_stage() {
        let decode = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let error = ForageRouteError::database(
            ForageReadStage::Receipt,
            7.into(),
            SpacetimeError::QueryResponseDecode(decode),
        );
        let ForageRouteError::Database { stage, actor, .. } = &error else {
            panic!("receipt read must retain its operation");
        };
        assert_eq!(*stage, ForageReadStage::Receipt);
        assert_eq!(*actor, CharacterId::from(7));
        assert!(error.source().unwrap().is::<SpacetimeError>());
        assert_eq!(error.feedback(), ForageFeedback::Unavailable);
    }
}
