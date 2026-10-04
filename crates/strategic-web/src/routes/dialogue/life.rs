//! Resident availability keeps chronology failure separate from a known death.

use super::super::{AppState, data};
use crate::spacetimedb::SpacetimeError;
use adventuresim_core::identity::CharacterId;
use data::ObservedLife;

pub(super) struct ResidentObservation {
    resident: CharacterId,
    observer: CharacterId,
}

impl ResidentObservation {
    pub(super) fn new(resident: CharacterId, observer: CharacterId) -> Self {
        Self { resident, observer }
    }

    pub(super) async fn life_at(self, state: &AppState) -> ObservedLife {
        let outcome = data::observed_life(state, self.resident, self.observer).await;
        self.admit(outcome)
    }

    fn admit(self, outcome: std::result::Result<ObservedLife, SpacetimeError>) -> ObservedLife {
        match outcome {
            Ok(life) => life,
            Err(error) => {
                tracing::warn!(%error, resident_character_id = u64::from(self.resident), observer_character_id = u64::from(self.observer), "could not project resident life state");
                ObservedLife::Alive
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spacetimedb::{DatabaseOperation, RemoteDatabaseFailure};

    #[test]
    fn known_death_and_absent_character_remain_unavailable() {
        for life in [
            ObservedLife::Alive,
            ObservedLife::Dead,
            ObservedLife::MissingCharacter,
        ] {
            let observation = ResidentObservation::new(CharacterId::from(8), CharacterId::from(7));
            assert_eq!(observation.admit(Ok(life)), life);
        }
    }

    #[test]
    fn query_failure_preserves_resident_availability() {
        let observation = ResidentObservation::new(CharacterId::from(8), CharacterId::from(7));
        let failure = RemoteDatabaseFailure::from_response(
            DatabaseOperation::Query,
            reqwest::StatusCode::SERVICE_UNAVAILABLE,
            Ok("chronology unavailable".into()),
        );
        assert_eq!(
            observation.admit(Err(SpacetimeError::Remote(failure))),
            ObservedLife::Alive
        );
    }
}
