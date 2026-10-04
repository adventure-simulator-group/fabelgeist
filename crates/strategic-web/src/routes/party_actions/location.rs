//! Admission of a character's persisted exact-site identity.
use super::super::AppState;
use crate::spacetimedb::{
    self as db, BackendCharacterCaseSiteLocation, CaseSiteId, SpacetimeError,
};
use adventuresim_core::{identity::CharacterId, strategic_place::PlaceIdentityError};

#[derive(Debug, thiserror::Error)]
pub(crate) enum CaseSiteObservationError {
    #[error("{source}")]
    Query {
        character: CharacterId,
        #[source]
        source: SpacetimeError,
    },
    #[error("{source}")]
    Identity {
        character: CharacterId,
        #[source]
        source: PlaceIdentityError,
    },
}

pub(crate) async fn character_case_site_id(
    state: &AppState,
    character_id: CharacterId,
) -> std::result::Result<Option<CaseSiteId>, CaseSiteObservationError> {
    state
        .db
        .query_one_sats::<BackendCharacterCaseSiteLocation>(
            db::character_case_site_location_by_character_id(character_id),
        )
        .await
        .map_err(|source: SpacetimeError| -> CaseSiteObservationError {
            CaseSiteObservationError::Query {
                character: character_id,
                source,
            }
        })?
        .map(
            |location: BackendCharacterCaseSiteLocation| -> std::result::Result<CaseSiteId, PlaceIdentityError> {
                CaseSiteId::try_new(location.case_site_id.value)
            },
        )
        .transpose()
        .map_err(|source: PlaceIdentityError| -> CaseSiteObservationError {
            CaseSiteObservationError::Identity {
                character: character_id,
                source,
            }
        })
}
