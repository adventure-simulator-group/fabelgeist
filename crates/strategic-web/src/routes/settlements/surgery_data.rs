//! Required surgical reads retain their dataset and underlying query failure.

use adventuresim_core::identity::CharacterId;
use axum::response::Html;
use spacetimedb_sats::de::DeserializeOwned;

use crate::{
    routes::AppState,
    spacetimedb::{SpacetimeError, SqlQuery},
};

#[derive(Clone, Copy, Debug)]
pub(super) enum SurgeryDataset {
    PatientInjuries,
    SurgeonInjuries,
    RetainedProjectiles,
    Inventory,
}

impl std::fmt::Display for SurgeryDataset {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::PatientInjuries => "patient injuries",
            Self::SurgeonInjuries => "surgeon injuries",
            Self::RetainedProjectiles => "retained projectiles",
            Self::Inventory => "surgeon inventory",
        })
    }
}

impl SurgeryDataset {
    fn query(self, character_id: CharacterId) -> SqlQuery {
        match self {
            Self::PatientInjuries | Self::SurgeonInjuries => SqlQuery::from(format!(
                "SELECT * FROM limb_injury WHERE character_id = {character_id}"
            )),
            Self::RetainedProjectiles => SqlQuery::from(format!(
                "SELECT * FROM retained_projectile WHERE character_id = {character_id}"
            )),
            Self::Inventory => SqlQuery::from(format!(
                "SELECT * FROM inventory_item WHERE character_id = {character_id}"
            )),
        }
    }

    pub(super) async fn load<T: DeserializeOwned>(
        self,
        state: &AppState,
        character_id: CharacterId,
    ) -> Result<Vec<T>, SurgeryDataFailure> {
        state.db.query_sats(self.query(character_id)).await.map_err(
            |source: SpacetimeError| -> SurgeryDataFailure {
                SurgeryDataFailure {
                    dataset: self,
                    source,
                }
            },
        )
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to load {dataset}: {source}")]
pub(super) struct SurgeryDataFailure {
    dataset: SurgeryDataset,
    #[source]
    source: SpacetimeError,
}

impl SurgeryDataFailure {
    pub(super) fn response(self) -> Html<String> {
        tracing::error!(error = %self.source, data_kind = %self.dataset, "failed to load surgery data");
        Html("<h1>Strategic medical data is unavailable</h1>".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_datasets_keep_the_exact_character_selector_and_full_identity_width() {
        let character = CharacterId::from(u64::MAX);
        for (dataset, expected) in [
            (
                SurgeryDataset::PatientInjuries,
                "SELECT * FROM limb_injury WHERE character_id = 18446744073709551615",
            ),
            (
                SurgeryDataset::SurgeonInjuries,
                "SELECT * FROM limb_injury WHERE character_id = 18446744073709551615",
            ),
            (
                SurgeryDataset::RetainedProjectiles,
                "SELECT * FROM retained_projectile WHERE character_id = 18446744073709551615",
            ),
            (
                SurgeryDataset::Inventory,
                "SELECT * FROM inventory_item WHERE character_id = 18446744073709551615",
            ),
        ] {
            assert_eq!(dataset.query(character).to_string(), expected);
        }
    }

    #[test]
    fn presentation_keeps_the_medical_notice_and_native_cause_until_logging() {
        use std::error::Error;
        let cause = serde_json::from_str::<serde_json::Value>("[").unwrap_err();
        let error = SurgeryDataFailure {
            dataset: SurgeryDataset::PatientInjuries,
            source: SpacetimeError::QueryResponseDecode(cause),
        };
        assert!(
            error
                .source()
                .unwrap()
                .source()
                .unwrap()
                .is::<serde_json::Error>()
        );
        assert!(
            error
                .to_string()
                .starts_with("failed to load patient injuries:")
        );
        assert_eq!(
            error.response().0,
            "<h1>Strategic medical data is unavailable</h1>"
        );
    }
}
