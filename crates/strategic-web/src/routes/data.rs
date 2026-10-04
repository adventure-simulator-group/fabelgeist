//! Shared typed reads used by more than one strategic feature.

mod chronology;

use chronology::PersonalFrontier;
pub(crate) use chronology::{FrontierAlignment, MutableCharacterAccess, ObservedLife};

use super::AppState;
use crate::spacetimedb::{
    BackendCharacterCaseSiteLocation, CaseSiteId, CharacterDeath, CharacterTime, CharacterView,
    Result, SpacetimeError,
};
use adventuresim_core::identity::CharacterId;
use adventuresim_world_schema::calendar::StrategicMinute;

async fn personal_frontier(
    state: &AppState,
    character_id: CharacterId,
) -> Result<PersonalFrontier> {
    state
        .db
        .query_one_sats::<CharacterTime>(crate::spacetimedb::character_time_by_character_id(
            character_id,
        ))
        .await
        .map(|time| {
            PersonalFrontier::new(
                character_id,
                time.map(|time| StrategicMinute::new(time.minutes.minutes)),
            )
        })
}

/// Mutable Character columns have no effective-dated history yet. They are
/// safe to use cross-character only when the subject has not advanced beyond
/// the observer. Callers that need co-location or another shared-current fact
/// should additionally require equal frontiers.
pub(crate) async fn mutable_character_access(
    state: &AppState,
    character_id: CharacterId,
    observer_character_id: CharacterId,
) -> Result<MutableCharacterAccess> {
    if character_id == observer_character_id {
        return Ok(MutableCharacterAccess::Available);
    }
    let (subject, observer) = tokio::join!(
        personal_frontier(state, character_id),
        personal_frontier(state, observer_character_id),
    );
    Ok(subject?.mutable_access_for(&observer?))
}

pub(crate) async fn frontier_alignment(
    state: &AppState,
    first_character_id: CharacterId,
    second_character_id: CharacterId,
) -> Result<FrontierAlignment> {
    let (first, second) = tokio::join!(
        personal_frontier(state, first_character_id),
        personal_frontier(state, second_character_id),
    );
    Ok(first?.alignment_with(&second?))
}

fn prefer_complete_cache<T>(cache: Option<Option<T>>, fallback: Option<T>) -> Option<T> {
    cache.unwrap_or(fallback)
}

pub(crate) async fn character(
    state: &AppState,
    character_id: CharacterId,
) -> Result<Option<CharacterView>> {
    let case_site_sql =
        crate::spacetimedb::character_case_site_location_by_character_id(character_id);
    // Character is a public mutable projection and is served from the SDK
    // cache once its explicit subscription is complete. The case-site view is
    // intentionally kept on HTTP SQL: owner-scoped/private projections are
    // never treated as an authorization boundary by the shared cache.
    let cached_character = state.live.cached_character(character_id);
    let mut character = match cached_character {
        Some(character) => Ok(prefer_complete_cache(Some(character), None)),
        None => {
            state
                .db
                .query_one_sats_into::<adventuresim_stdb_client::Character, CharacterView>(
                    crate::spacetimedb::character_by_id(character_id),
                )
                .await
        }
    }?;
    let case_site = state
        .db
        .query_one_sats::<BackendCharacterCaseSiteLocation>(case_site_sql)
        .await?;
    if let Some(character) = character.as_mut() {
        character.current_case_site_id = case_site
            .map(|location| CaseSiteId::try_new(location.case_site_id.value))
            .transpose()
            .map_err(SpacetimeError::CaseSiteIdentity)?;
    }
    Ok(character)
}

/// Reconstruct mutable life state at the selected observer's authoritative
/// personal minute. The trusted gateway can read the broad current Character
/// projection, but must not disclose a death from another character's future.
pub(crate) async fn project_alive_as_observed(
    state: &AppState,
    observer_character_id: CharacterId,
    characters: &mut [CharacterView],
) -> Result<()> {
    let observer_time = match state
        .db
        .query_one_sats::<CharacterTime>(crate::spacetimedb::character_time_by_character_id(
            observer_character_id,
        ))
        .await
    {
        Ok(time) => time,
        Err(error) => {
            tracing::warn!(%error, observer_character_id = u64::from(observer_character_id), "could not read observer chronology");
            for character in characters.iter_mut().filter(|character| !character.alive) {
                character.alive = true;
            }
            return Ok(());
        }
    };
    let Some(observer_minute) =
        observer_time.map(|time| StrategicMinute::new(time.minutes.minutes))
    else {
        // Without an observer frontier the gateway cannot safely decide that a
        // broad current death is already knowable. Preserve availability and
        // leave authoritative reducers to reject actions when appropriate.
        for character in characters.iter_mut().filter(|character| !character.alive) {
            character.alive = true;
        }
        return Ok(());
    };
    for character in characters.iter_mut().filter(|character| !character.alive) {
        let death = match state
            .db
            .query_one_sats::<CharacterDeath>(crate::spacetimedb::character_death_by_character_id(
                character.id.into(),
            ))
            .await
        {
            Ok(death) => death,
            Err(error) => {
                tracing::warn!(%error, character_id = character.id, observer_character_id = u64::from(observer_character_id), "could not read death chronology");
                character.alive = true;
                continue;
            }
        };
        character.alive = ObservedLife::at_date(
            Some(observer_minute),
            death.map(|death| StrategicMinute::new(death.strategic_minute.minutes)),
        ) == ObservedLife::Alive;
    }
    Ok(())
}

pub(crate) async fn character_as_observed(
    state: &AppState,
    character_id: CharacterId,
    observer_character_id: CharacterId,
) -> Result<Option<CharacterView>> {
    if mutable_character_access(state, character_id, observer_character_id).await?
        != MutableCharacterAccess::Available
    {
        // We cannot reconstruct location, party, age, wealth, or progression
        // at the observer's earlier date. Fail closed instead of returning a
        // row containing mutable facts from the subject's future.
        return Ok(None);
    }
    let mut character = character(state, character_id).await?;
    if let Some(character) = character.as_mut() {
        project_alive_as_observed(
            state,
            observer_character_id,
            std::slice::from_mut(character),
        )
        .await?;
    }
    Ok(character)
}

pub(crate) async fn observed_life(
    state: &AppState,
    character_id: CharacterId,
    observer_character_id: CharacterId,
) -> Result<ObservedLife> {
    // Life state has its own effective-dated history, so callers that need
    // only this fact must not inherit the fail-closed policy for unrelated
    // mutable Character fields. This keeps asynchronous NPC dialogue
    // available without exposing the NPC's future location, wealth, or party.
    let mut character = character(state, character_id).await?;
    let Some(character) = character.as_mut() else {
        return Ok(ObservedLife::MissingCharacter);
    };
    project_alive_as_observed(
        state,
        observer_character_id,
        std::slice::from_mut(character),
    )
    .await?;
    Ok(ObservedLife::from_character(character))
}

pub(crate) fn new_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

#[cfg(test)]
mod tests {
    use super::prefer_complete_cache;
    use crate::spacetimedb::CharacterView;
    use adventuresim_core::identity::CharacterId;

    fn character(id: CharacterId) -> CharacterView {
        CharacterView {
            id: u64::from(id),
            name: format!("character-{id}"),
            xp: 0,
            level: 1,
            current_settlement_id: None,
            current_case_site_id: None,
            party_id: None,
            age_years: 20,
            alive: true,
            temporary: false,
            social_notification_count: 0,
            automatic_social_chat_enabled: false,
        }
    }

    #[test]
    fn complete_cache_hit_wins_and_cache_miss_falls_back() {
        assert_eq!(
            prefer_complete_cache(
                Some(Some(character(CharacterId::from(7)))),
                Some(character(CharacterId::from(8))),
            )
            .unwrap()
            .id,
            7
        );
        assert!(prefer_complete_cache(Some(None), Some(character(CharacterId::from(8)))).is_none());
        assert_eq!(
            prefer_complete_cache(None, Some(character(CharacterId::from(8))))
                .unwrap()
                .id,
            8
        );
    }

    #[test]
    fn character_loader_uses_authoritative_case_site_projection() {
        let source = include_str!("data.rs");
        let loader = source
            .split("pub(crate) async fn character(")
            .nth(1)
            .unwrap()
            .split("pub(crate) async fn project_alive_as_observed")
            .next()
            .unwrap();
        assert!(loader.contains("character_case_site_location_by_character_id"));
        assert!(loader.contains("character.current_case_site_id"));
        assert!(loader.contains("location.case_site_id.value"));
        assert!(!loader.contains("current_case_site_id.unwrap"));
    }

    #[test]
    fn observed_character_projection_uses_observer_time_and_private_death_view() {
        let source = include_str!("data.rs");
        let projection = source
            .split("pub(crate) async fn project_alive_as_observed")
            .nth(1)
            .unwrap()
            .split("pub(crate) async fn character_as_observed")
            .next()
            .unwrap();
        assert!(projection.contains("character_time_by_character_id"));
        assert!(projection.contains("character_death_by_character_id"));
        assert!(projection.contains("ObservedLife::at_date"));
        assert!(projection.contains("death.strategic_minute.minutes"));
        assert!(projection.contains("let Some(observer_minute)"));
        assert!(projection.contains("character.alive = true"));
        assert!(!projection.contains("character_death WHERE"));
    }

    #[test]
    fn cross_character_loader_fails_closed_before_reading_future_mutable_state() {
        let source = include_str!("data.rs");
        let loader = source
            .split("pub(crate) async fn character_as_observed")
            .nth(1)
            .unwrap()
            .split("pub(crate) async fn observed_life")
            .next()
            .unwrap();
        let chronology = loader.find("mutable_character_access").unwrap();
        let mutable_read = loader.find("character(state, character_id)").unwrap();
        assert!(chronology < mutable_read);
        assert!(loader.contains("return Ok(None)"));
    }

    #[test]
    fn life_only_projection_does_not_require_mutable_frontier_alignment() {
        let source = include_str!("data.rs");
        let loader = source
            .split("pub(crate) async fn observed_life")
            .nth(1)
            .unwrap()
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert!(loader.contains("project_alive_as_observed"));
        assert!(!loader.contains("character_as_observed"));
        assert!(!loader.contains("mutable_character_access"));
    }
}
