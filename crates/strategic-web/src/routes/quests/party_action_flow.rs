//! Quest-journal party intent and its player-safe outcome presentation.
use super::super::{
    AppState, PartyAction, PartyActionError, PartyActionOutcome, execute_or_request_party_action,
};
use crate::location_urls::patterns as paths;
use axum::response::Redirect;

pub(super) async fn accept_quest_for_character(
    state: &AppState,
    character_id: adventuresim_core::identity::CharacterId,
    quest_id: &str,
) -> std::result::Result<PartyActionOutcome, PartyActionError> {
    execute_or_request_party_action(
        state,
        character_id,
        PartyAction::AcceptContract {
            contract_id: quest_id.into(),
        },
    )
    .await
}

pub(super) fn safe_case_site_travel_error(_error: &PartyActionError) -> &'static str {
    "The exact destination or the party's travel readiness changed. Review the journal before trying again."
}

pub(super) fn autoresolve_redirect(
    case_site_id: Option<&str>,
    outcome: std::result::Result<PartyActionOutcome, PartyActionError>,
) -> Redirect {
    match outcome {
        Ok(PartyActionOutcome::Executed) | Err(_) => case_site_id.map_or_else(
            || Redirect::to("/"),
            |id| Redirect::to(&paths::QUEST_LOCATION_ENEMY.url([&id])),
        ),
        Ok(PartyActionOutcome::Requested) => Redirect::to("/?party-requested=autoresolve"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::header::LOCATION, response::IntoResponse};
    fn redirect_location(redirect: Redirect) -> String {
        redirect
            .into_response()
            .headers()
            .get(LOCATION)
            .expect("redirect has a location")
            .to_str()
            .expect("redirect location is valid text")
            .to_owned()
    }

    #[test]
    fn autoresolve_stays_on_the_enemy_lifecycle_except_while_requesting_approval() {
        let enemy = "/locations/case-site/case-site-1/enemy";
        assert_eq!(
            redirect_location(autoresolve_redirect(
                Some("case-site-1"),
                Ok(PartyActionOutcome::Executed),
            )),
            enemy,
        );
        assert_eq!(
            redirect_location(autoresolve_redirect(
                Some("case-site-1"),
                Err(PartyActionError::MissingActor(7.into()))
            )),
            enemy,
        );
        assert_eq!(
            redirect_location(autoresolve_redirect(
                Some("case-site-1"),
                Ok(PartyActionOutcome::Requested),
            )),
            "/?party-requested=autoresolve",
        );
    }

    #[test]
    fn case_site_travel_errors_are_safe_and_actionable() {
        let private = PartyActionError::reducer(
            7.into(),
            crate::spacetimedb::SpacetimeError::Remote(
                crate::spacetimedb::RemoteDatabaseFailure::from_response(
                    crate::spacetimedb::DatabaseOperation::Reducer,
                    reqwest::StatusCode::CONFLICT,
                    Ok("private canonical site mismatch: site:secret".into()),
                ),
            ),
        );
        assert_eq!(
            safe_case_site_travel_error(&PartyActionError::from(
                crate::routes::party_readiness::PartyReadinessError::Incapacitated(7.into())
            )),
            "The exact destination or the party's travel readiness changed. Review the journal before trying again."
        );
        assert_eq!(
            safe_case_site_travel_error(&private),
            "The exact destination or the party's travel readiness changed. Review the journal before trying again."
        );
        assert!(!safe_case_site_travel_error(&private).contains("site:secret"));
    }
}
