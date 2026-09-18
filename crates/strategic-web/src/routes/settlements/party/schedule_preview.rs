//! Read-only previews calculated from current server eligibility.
//!
//! A result reflects server state at calculation time. It matches execution
//! only for the same allocation, character seed, and eligibility context;
//! execution authorizes and validates against its own current state.
use super::*;
use crate::schedule::SchedulePreview;
use adventuresim_core::activity::ActivityLocation;

pub(in crate::routes::settlements) async fn preview_training_schedule(
    State(state): State<AppState>,
    Path((kind, id, character_id)): Path<(String, String, u64)>,
    session: Session,
    Form(form): Form<TrainingScheduleForm>,
) -> Response {
    // Session extraction verified the signature and read current character grants.
    // Authentication alone is insufficient: match both the grant and selection.
    if !may_preview(
        session.character_id_u64(),
        &session.character_ids(),
        character_id,
    ) {
        return (
            StatusCode::FORBIDDEN,
            "Select this character before inspecting their schedule",
        )
            .into_response();
    }
    let mut schedule = match form.into_schedule() {
        Ok(schedule) => schedule,
        Err(error) => return (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    };
    let (character, location) = match current_location(&state, &kind, &id, character_id).await {
        Ok(context) => context,
        Err(error) => return error.into_response(),
    };
    let policy = match location_policy(&state, &location, character_id).await {
        Ok(policy) => policy,
        Err(error) => return error.into_response(),
    };
    if let Err(error) = resolve_organizations(&state, &character, &mut schedule).await {
        return error.into_response();
    }
    match SchedulePreview::calculate(&schedule, policy, character_id) {
        Ok(preview) => (
            [(axum::http::header::CACHE_CONTROL, "no-store")],
            Json(preview),
        )
            .into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    }
}

fn may_preview(selected: Option<u64>, granted: &[u64], requested: u64) -> bool {
    selected == Some(requested) && granted.contains(&requested)
}

async fn current_location(
    state: &AppState,
    kind: &str,
    id: &str,
    character_id: u64,
) -> Result<(CharacterView, LocationView), PreviewError> {
    let mut character = state
        .db
        .query_one_sats_into::<adventuresim_stdb_client::Character, CharacterView>(
            &crate::spacetimedb::character_by_id(character_id),
        )
        .await
        .map_err(|_| unavailable())?
        .ok_or(PreviewError::CharacterNotFound)?;
    let occupancy = state
        .db
        .query_one_sats::<crate::spacetimedb::BackendCharacterCaseSiteLocation>(
            &crate::spacetimedb::character_case_site_location_by_character_id(character_id),
        )
        .await
        .map_err(|_| unavailable())?;
    character.current_case_site_id = occupancy
        .map(|row| crate::spacetimedb::CaseSiteId::try_new(row.case_site_id.value))
        .transpose()
        .map_err(|_| unavailable())?;
    if !character.alive {
        return Err(PreviewError::InactiveCharacter);
    }
    let kind_value = kind
        .parse::<LocationKind>()
        .map_err(|_| PreviewError::LocationNotFound)?;
    let at_requested_location = match kind_value {
        LocationKind::Settlement => character.current_settlement_id.as_deref() == Some(id),
        LocationKind::CaseSite => {
            character.current_settlement_id.is_none()
                && character.current_case_site_id.as_deref() == Some(id)
        }
    };
    if !at_requested_location {
        return Err(PreviewError::LocationChanged);
    }
    let location = resolve_preview_location(state, kind_value, kind, id, character_id).await?;
    if !character_is_at_location(&character, &location)
        || (location.kind == LocationKind::CaseSite && character.current_settlement_id.is_some())
    {
        return Err(PreviewError::LocationChanged);
    }
    Ok((character, location))
}

async fn resolve_preview_location(
    state: &AppState,
    kind: LocationKind,
    kind_source: &str,
    id: &str,
    character_id: u64,
) -> Result<LocationView, PreviewError> {
    if kind == LocationKind::Settlement {
        return match resolve_location(state, kind_source, id).await {
            LocationLookup::Found(location) => Ok(location),
            LocationLookup::NotFound => Err(PreviewError::LocationNotFound),
            LocationLookup::Unavailable => Err(unavailable()),
        };
    }
    let site = state
        .db
        .query_one_sats::<BackendCaseSitePin>(
            &crate::spacetimedb::case_site_pin_by_case_site_id_and_owner(id, character_id),
        )
        .await
        .map_err(|_| unavailable())?
        .ok_or(PreviewError::LocationNotFound)?;
    Ok(LocationView {
        kind,
        id: id.to_owned(),
        name: site.display_title,
        religion_id: None,
        category: None,
        economy: None,
        active_building: None,
    })
}

async fn location_policy(
    state: &AppState,
    location: &LocationView,
    character_id: u64,
) -> Result<ActivityLocation, PreviewError> {
    match location.kind {
        LocationKind::Settlement => Ok(ActivityLocation::Settlement {
            has_inn: location.economy.as_ref().is_some_and(|economy| {
                economy.has_service(adventuresim_world_schema::SettlementService::Inn)
            }),
        }),
        LocationKind::CaseSite => {
            let site = state
                .db
                .query_one_sats::<BackendCaseSitePin>(
                    &crate::spacetimedb::case_site_pin_by_case_site_id_and_owner(
                        &location.id,
                        character_id,
                    ),
                )
                .await
                .map_err(|_| unavailable())?
                .ok_or_else(unavailable)?;
            Ok(if site.raiding_allowed {
                ActivityLocation::NamedOutdoorLocation
            } else {
                ActivityLocation::IneligibleNamedLocation
            })
        }
    }
}

async fn resolve_organizations(
    state: &AppState,
    character: &CharacterView,
    schedule: &mut ScheduleAllocation,
) -> Result<(), PreviewError> {
    if schedule.apprenticeship_minutes == 0 && schedule.profession_practice_minutes == 0 {
        return Ok(());
    }
    let character_id = character.id;
    let memberships: Vec<crate::spacetimedb::BackendOrganizationMembership> = state
        .db
        .query_sats(&format!(
            "SELECT * FROM backend_organization_memberships WHERE character_id = {character_id}"
        ))
        .await
        .map_err(|_| unavailable())?;
    let minute = state
        .db
        .query_one_sats::<CharacterTime>(&crate::spacetimedb::character_time_by_character_id(
            character_id,
        ))
        .await
        .map_err(|_| unavailable())?
        .ok_or_else(unavailable)?
        .minutes;
    let eligible = |id: Option<&str>, practice: bool| {
        id.is_some_and(|id| {
            let Some(definition) = adventuresim_core::organization::organization(id) else {
                return false;
            };
            character
                .current_settlement_id
                .as_deref()
                .is_some_and(|settlement| definition.has_chapter(settlement))
                && memberships.iter().any(|row| {
                    row.organization_id == id
                        && row.status
                            == adventuresim_stdb_client::OrganizationMembershipStatus::Active
                        && minute <= row.dues_paid_through_minute
                        && (!practice
                            || definition
                                .role(&row.role_id)
                                .is_some_and(|role| role.practice_allowed))
                })
        })
    };
    // Execution also releases lapsed organization allocations to leisure before
    // location redistribution; it samples eligibility again at execution time.
    if !eligible(schedule.apprenticeship_organization_id.as_deref(), false) {
        schedule.apprenticeship_minutes = 0;
    }
    if !eligible(schedule.practice_organization_id.as_deref(), true) {
        schedule.profession_practice_minutes = 0;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum PreviewError {
    Unavailable,
    CharacterNotFound,
    InactiveCharacter,
    LocationNotFound,
    LocationChanged,
}

impl IntoResponse for PreviewError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Schedule context is unavailable",
            ),
            Self::CharacterNotFound => (StatusCode::NOT_FOUND, "Character not found"),
            Self::InactiveCharacter => (
                StatusCode::CONFLICT,
                "Character can no longer follow a schedule",
            ),
            Self::LocationNotFound => (StatusCode::NOT_FOUND, "Location not found"),
            Self::LocationChanged => (
                StatusCode::CONFLICT,
                "Character is no longer at this location",
            ),
        };
        (status, message).into_response()
    }
}

fn unavailable() -> PreviewError {
    PreviewError::Unavailable
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preview_requires_selected_character_and_verified_grant() {
        assert!(may_preview(Some(7), &[7, 8], 7));
        assert!(!may_preview(Some(7), &[7, 8], 8));
        assert!(!may_preview(Some(7), &[8], 7));
        assert!(!may_preview(None, &[7], 7));
    }
}

#[cfg(test)]
#[path = "schedule_preview/http_tests.rs"]
mod http_tests;
