use super::vicinity::vicinity;
use crate::spacetimedb::SqlQuery;
mod destination;
mod error;
mod feedback;
mod form;
mod notice;
mod receipt;
mod request;
use adventuresim_core::{foraging::ForageAttemptGeneration, identity::CharacterId};
use destination::ForageDialogDestination;
use error::{ForageReadStage, ForageRouteError};
use form::{FORAGE_FORM_MAX_BYTES, ForageForm};
use notice::ForageFeedback;
use receipt::{ForageReceipt, ForageReceiptError};
use request::ForageReceiptReference;

use adventuresim_world_schema::calendar::StrategicMinute;
use axum::{
    Router,
    extract::{DefaultBodyLimit, RawForm, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::post,
};
use maud::{Markup, html};
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeSet;

use super::{AppState, wgs84_e7};
use crate::{
    session::Session,
    spacetimedb::{
        BackendForageAttemptState, BackendForageReceipt, BackendOrganizationMembership,
        CharacterTime, CharacterView, OrganizationMembershipStatus, OrganizationPresentation,
        SpacetimeError,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/forage",
        post(perform).layer(DefaultBodyLimit::max(FORAGE_FORM_MAX_BYTES)),
    )
}

#[derive(Serialize)]
struct WireForageEnvironmentAttestation<'a> {
    package_digest: &'a str,
    // SpacetimeDB codegen splits the numeric suffix in these wire names.
    latitude_e_7: i32,
    longitude_e_7: i32,
    context_kind: &'a str,
    context_id: &'a str,
    plains: u16,
    forest: u16,
    hills: u16,
    wetlands: u16,
    river_or_wet_ground: bool,
    sea_or_coast: bool,
    cultivated: bool,
}

fn source_privilege(
    source: adventuresim_core::foraging::ForageSource,
) -> Option<adventuresim_core::organization::Privilege> {
    use adventuresim_core::foraging::ForageSource;
    use adventuresim_core::organization::Privilege;
    Some(match source {
        ForageSource::HighGame => Privilege::ForageHighGame,
        ForageSource::LowGame => Privilege::ForageLowGame,
        ForageSource::Fish => Privilege::ForageFish,
        ForageSource::Plants => Privilege::ForagePlants,
        ForageSource::HarmfulBeasts => return None,
    })
}

async fn advisory_privileges(
    state: &AppState,
    character_id: CharacterId,
) -> BTreeSet<adventuresim_core::organization::Privilege> {
    let presentation = state
        .db
        .query_one_sats::<OrganizationPresentation>(
            crate::spacetimedb::organization_presentation_by_character_id(character_id),
        )
        .await
        .ok()
        .flatten();
    let memberships = state
        .db
        .query_sats::<BackendOrganizationMembership>(SqlQuery::from(format!(
            "SELECT * FROM backend_organization_memberships WHERE character_id = {character_id}"
        )))
        .await
        .unwrap_or_default();
    let minute = state
        .db
        .query_one_sats::<CharacterTime>(crate::spacetimedb::character_time_by_character_id(
            character_id,
        ))
        .await
        .ok()
        .flatten()
        .map(|row| StrategicMinute::new(row.minutes.minutes));
    advisory_privileges_for(
        presentation
            .as_ref()
            .map(|presentation| presentation.organization_id.as_str()),
        &memberships,
        minute,
    )
}

fn advisory_privileges_for(
    presented_organization_id: Option<&str>,
    memberships: &[BackendOrganizationMembership],
    minute: Option<StrategicMinute>,
) -> BTreeSet<adventuresim_core::organization::Privilege> {
    let Some((presented_organization_id, minute)) = presented_organization_id.zip(minute) else {
        return BTreeSet::new();
    };
    let Some(definition) = adventuresim_core::organization::organization(presented_organization_id)
    else {
        return BTreeSet::new();
    };
    let Some(membership) = memberships.iter().find(|membership| {
        membership.organization_id == presented_organization_id
            && membership.status == OrganizationMembershipStatus::Active
            && minute <= StrategicMinute::new(membership.dues_paid_through_minute.minutes)
    }) else {
        return BTreeSet::new();
    };
    [
        adventuresim_core::organization::Privilege::ForageHighGame,
        adventuresim_core::organization::Privilege::ForageLowGame,
        adventuresim_core::organization::Privilege::ForageFish,
        adventuresim_core::organization::Privilege::ForagePlants,
    ]
    .into_iter()
    .filter(|privilege| definition.has_privilege_at_role(&membership.role_id, *privilege))
    .collect()
}

fn source_rows(
    environment: adventuresim_core::foraging::ForageEnvironment,
    privileges: &BTreeSet<adventuresim_core::organization::Privilege>,
) -> Markup {
    html! {
        @for source in adventuresim_core::foraging::ForageSource::ALL {
            @let available = adventuresim_core::foraging::source_available(source, environment);
            @let licensed = source_privilege(source)
                .is_none_or(|privilege| privileges.contains(&privilege));
            @let tooltip = (!licensed).then_some(
                "Your presented profession does not grant the hunting license required for this source. Selecting it is poaching."
            );
            label class=(format!(
                    "forage-source-row{}{}",
                    if !licensed { " forage-source-unlicensed" } else { "" },
                    if !available { " forage-source-unavailable" } else { "" }
                ))
                tabindex=[(!licensed && available).then_some("0")]
                data-strategic-tooltip=[tooltip] {
                input type="checkbox" name="source" value=(source.id()) disabled[!available];
                span class="forage-source-copy" {
                    strong { (source.name()) }
                    span { (source.description()) }
                }
                @if !available {
                    span class="forage-source-status" { "Unavailable here" }
                } @else if !licensed {
                    span class="forage-source-status" { "Unlicensed" }
                } @else if !source.requires_license() {
                    span class="forage-source-status" { "No license required" }
                } @else {
                    span class="forage-source-status" { "Licensed" }
                }
            }
        }
    }
}

async fn character(
    state: &AppState,
    id: CharacterId,
) -> std::result::Result<CharacterView, ForageRouteError> {
    state
        .db
        .query_one_sats_into::<adventuresim_stdb_client::Character, CharacterView>(
            crate::spacetimedb::character_by_id(id),
        )
        .await
        .map_err(|source: SpacetimeError| -> ForageRouteError {
            ForageRouteError::database(ForageReadStage::Actor, id, source)
        })?
        .ok_or(ForageRouteError::MissingActor(id))
}

async fn environment(
    state: &AppState,
    character: &CharacterView,
) -> std::result::Result<
    (
        adventuresim_core::foraging::ForageEnvironment,
        serde_json::Value,
    ),
    ForageRouteError,
> {
    let location = vicinity(state, character).await?;
    let terrain = state
        .terrain
        .as_deref()
        .ok_or(ForageRouteError::TerrainUnavailable)?;
    let (cell, river_or_wet, coastal) =
        terrain.forage_environment(location.latitude, location.longitude)?;
    if cell.surface == adventuresim_terrain::Surface::Water && !cell.crossing {
        return Err(ForageRouteError::OpenWater);
    }
    let weights = cell.terrain_weights();
    let mixture = adventuresim_core::foraging::LocalTerrainMixture {
        plains: weights.plains + weights.urban,
        forest: weights.forest,
        hills: weights.hills,
        wetlands: weights.wetlands,
    };
    let environment = adventuresim_core::foraging::ForageEnvironment {
        terrain: mixture,
        river_or_wet_ground: river_or_wet,
        sea_or_coast: coastal,
        cultivated: cell.cultivated,
        settlement: location.settlement,
        license_violation: false,
    };
    let (latitude_e7, longitude_e7) = wgs84_e7(location.latitude, location.longitude)?;
    let attestation = serde_json::to_value(WireForageEnvironmentAttestation {
        package_digest: terrain.digest(),
        latitude_e_7: latitude_e7,
        longitude_e_7: longitude_e7,
        context_kind: &location.kind,
        context_id: &location.id,
        plains: mixture.plains,
        forest: mixture.forest,
        hills: mixture.hills,
        wetlands: mixture.wetlands,
        river_or_wet_ground: environment.river_or_wet_ground,
        sea_or_coast: environment.sea_or_coast,
        cultivated: environment.cultivated,
    })?;
    Ok((environment, attestation))
}

pub(crate) async fn activity_dialog(
    state: &AppState,
    character: &CharacterView,
    return_to: &str,
    receipt_id: Option<&str>,
    error_code: Option<&str>,
) -> Markup {
    let receipt_reference = receipt_id
        .map(ForageReceiptReference::try_from)
        .and_then(Result::ok);
    let receipt = if let Some(request_id) = receipt_reference {
        state
            .db
            .query_one_sats::<BackendForageReceipt>(request_id.query(character.id.into()))
            .await
            .ok()
            .flatten()
            .map(|native: BackendForageReceipt| -> std::result::Result<ForageReceipt, ForageReceiptError> {
                ForageReceipt::admit(native, character.id.into(), &request_id)
            })
    } else {
        None
    };
    let error_message = error_code
        .and_then(ForageFeedback::from_http)
        .map(ForageFeedback::message);
    let outcome = environment(state, character).await;
    let (environment, unavailable) = match outcome {
        Ok((environment, _)) => (Some(environment), None),
        Err(error) => (None, Some(error.feedback().message())),
    };
    let privileges = advisory_privileges(state, character.id.into()).await;
    let illegal =
        environment.is_some_and(|environment| environment.settlement || environment.cultivated);
    let available_source_rows = environment
        .as_ref()
        .map(|environment| source_rows(*environment, &privileges));
    html! {
        div class="character-action-overlay" data-character-action-dialog
            data-initial-focus=(if receipt.is_some() { ".modal-actions .btn" } else { "#forage-targets input:not(:disabled)" }) {
            a class="character-action-backdrop" href=(return_to) aria-label="Close foraging dialog" {}
            section class="character-action-dialog forage-dialog" role="dialog" aria-modal="true"
                aria-labelledby="forage-title" aria-describedby="forage-description" tabindex="-1" {
                header class="character-action-dialog-header" {
                    h2 id="forage-title" { "Forage nearby" }
                    a class="character-action-dialog-close" href=(return_to) aria-label="Close foraging dialog" { "×" }
                }
                p id="forage-description" { "Search only the character's immediate vicinity. Selected sources share the search time." }
                @if let Some(receipt) = receipt.as_ref() {
                    @match receipt {
                        Ok(receipt) => { (receipt.render()) }
                        Err(_error) => {
                            p role="alert" class="badge badge-danger" {
                                (ForageFeedback::Unavailable.message())
                            }
                        }
                    }
                    div class="modal-actions" {
                        a class="btn btn-primary character-action-dialog-close" href=(return_to) { "Return" }
                    }
                } @else if let Some(reason) = unavailable {
                    (feedback::unavailable(reason, return_to))
                } @else {
                    @if let Some(message) = error_message {
                        p role="alert" class="badge badge-danger" { (message) }
                    }
                    @if illegal {
                        p role="alert" class="badge badge-warning" { "Foraging here is illegal. One Stealth check is made when the search completes; failure adds local Infamy." }
                    }
                    form method="post" action="/forage" {
                        input type="hidden" name="return_to" value=(return_to);
                        p id="forage-source-help" class="text-muted small-copy" { "Choose food sources. Selected categories share one search-time budget." }
                        fieldset id="forage-targets" aria-describedby="forage-source-help" {
                            legend { "Food sources" }
                            @if let Some(rows) = available_source_rows {
                                (rows)
                            }
                        }
                        label for="forage-hours" { "Search plan" }
                        input id="forage-hours" name="hours" type="range" min="1" max="24" value="4"
                            oninput="this.nextElementSibling.value=this.value + ' hours'";
                        output { "4 hours" }
                        div class="modal-actions" {
                            button class="btn btn-primary" type="submit" { "Begin search" }
                            a class="btn btn-secondary character-action-dialog-close" href=(return_to) { "Cancel" }
                        }
                    }
                }
            }
        }
    }
}

async fn perform(
    State(state): State<AppState>,
    session: Session,
    RawForm(body): RawForm,
) -> Response {
    let Ok(form) = ForageForm::try_from(&body[..]) else {
        return StatusCode::UNPROCESSABLE_ENTITY.into_response();
    };
    let return_to = form.return_destination();
    let Some(character_id) = session.character_id_u64().map(CharacterId::from) else {
        return Redirect::to("/characters").into_response();
    };
    let result = async {
        let character = character(&state, character_id).await?;
        let (environment, attestation) = environment(&state, &character).await?;
        let minutes = form.duration();
        let request_id = ForageReceiptReference::issue(character_id);
        let attempt_generation = match state
            .db
            .query_one_sats::<BackendForageAttemptState>(
                crate::spacetimedb::forage_attempt_state_by_character_id(character_id),
            )
            .await
            .map_err(|source: SpacetimeError| -> ForageRouteError {
                ForageRouteError::database(ForageReadStage::Generation, character_id, source)
            })? {
            Some(row) => ForageAttemptGeneration::from(row.next_generation),
            None => ForageAttemptGeneration::INITIAL,
        };
        // The browser submits only selected categories and duration. This
        // session-scoped endpoint hydrates the opaque request, generation, and
        // private terrain attestation before entering the plan gateway.
        state
            .db
            .call(
                "forage_current_vicinity",
                &[
                    json!(character_id),
                    json!(&request_id),
                    json!(form.sources()),
                    json!(minutes),
                    json!(attempt_generation),
                    attestation,
                ],
            )
            .await
            .map_err(|source: SpacetimeError| -> ForageRouteError {
                ForageRouteError::database(ForageReadStage::Execute, character_id, source)
            })?;
        let attempt = state
            .db
            .query_one_sats::<BackendForageReceipt>(request_id.query(character_id))
            .await
            .map_err(|source: SpacetimeError| -> ForageRouteError {
                ForageRouteError::database(ForageReadStage::Receipt, character_id, source)
            })?
            .ok_or(ForageRouteError::ReceiptUnavailable)?;
        let receipt = ForageReceipt::admit(attempt, character_id, &request_id).map_err(
            |source: ForageReceiptError| -> ForageRouteError {
                ForageRouteError::Receipt {
                    actor: character_id,
                    source,
                }
            },
        )?;
        Ok::<_, ForageRouteError>((environment, receipt, request_id))
    }
    .await;
    match result {
        Ok((_environment, _receipt, request_id)) => {
            ForageDialogDestination::receipt(return_to, &request_id)
                .redirect()
                .into_response()
        }
        Err(error) => ForageDialogDestination::failure(return_to, error.feedback())
            .redirect()
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mixed_environment() -> adventuresim_core::foraging::ForageEnvironment {
        adventuresim_core::foraging::ForageEnvironment {
            terrain: adventuresim_core::foraging::LocalTerrainMixture {
                plains: 400,
                forest: 400,
                hills: 200,
                wetlands: 0,
            },
            river_or_wet_ground: true,
            sea_or_coast: false,
            cultivated: false,
            settlement: false,
            license_violation: false,
        }
    }

    #[test]
    fn source_markup_is_ordered_accessible_and_keeps_poaching_enabled() {
        let markup = source_rows(mixed_environment(), &BTreeSet::new()).into_string();
        let positions = ["High Game", "Low Game", "Fish", "Harmful Beasts", "Plants"]
            .map(|label| markup.find(label).unwrap());
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(markup.matches("type=\"checkbox\"").count(), 5);
        assert_eq!(markup.matches("Unlicensed").count(), 4);
        assert!(markup.contains("No license required"));
        assert!(markup.contains("forage-source-unlicensed"));
        assert!(markup.contains("data-strategic-tooltip="));
        assert!(!markup.contains("title="));
        assert!(!markup.contains("value=\"high_game\" disabled"));
    }

    #[test]
    fn unavailable_source_remains_visible_and_disabled() {
        let mut environment = mixed_environment();
        environment.river_or_wet_ground = false;
        let markup = source_rows(environment, &BTreeSet::new()).into_string();
        assert!(markup.contains("value=\"fish\" disabled"));
        assert!(markup.contains("Unavailable here"));
    }

    fn ranger_membership(
        role_id: &str,
        status: OrganizationMembershipStatus,
        paid_through: u64,
    ) -> BackendOrganizationMembership {
        BackendOrganizationMembership {
            id: 1,
            character_id: 7,
            organization_id: "lodge_hart_king".into(),
            role_id: role_id.into(),
            joined_minute: adventuresim_stdb_client::StrategicMinute { minutes: 0 },
            dues_paid_through_minute: adventuresim_stdb_client::StrategicMinute {
                minutes: paid_through,
            },
            status,
            apprenticeship_minutes_accrued: 0,
            practice_minutes_accrued: 0,
        }
    }

    #[test]
    fn advisory_licenses_require_matching_current_presentation_and_role() {
        use adventuresim_core::organization::Privilege;
        let warden = ranger_membership("warden", OrganizationMembershipStatus::Active, 100);
        let common = advisory_privileges_for(
            Some("lodge_hart_king"),
            std::slice::from_ref(&warden),
            Some(StrategicMinute::new(100)),
        );
        assert!(common.contains(&Privilege::ForageLowGame));
        assert!(common.contains(&Privilege::ForageFish));
        assert!(common.contains(&Privilege::ForagePlants));
        assert!(!common.contains(&Privilege::ForageHighGame));

        let master = ranger_membership("master", OrganizationMembershipStatus::Active, 100);
        assert!(
            advisory_privileges_for(
                Some("lodge_hart_king"),
                &[master],
                Some(StrategicMinute::new(100))
            )
            .contains(&Privilege::ForageHighGame)
        );
        assert!(
            advisory_privileges_for(
                None,
                std::slice::from_ref(&warden),
                Some(StrategicMinute::new(100))
            )
            .is_empty()
        );
        assert!(
            advisory_privileges_for(
                Some("hunt_pale_lantern"),
                std::slice::from_ref(&warden),
                Some(StrategicMinute::new(100))
            )
            .is_empty()
        );
        let lapsed = ranger_membership("master", OrganizationMembershipStatus::Active, 99);
        assert!(
            advisory_privileges_for(
                Some("lodge_hart_king"),
                &[lapsed],
                Some(StrategicMinute::new(100))
            )
            .is_empty()
        );
        let suspended = ranger_membership("master", OrganizationMembershipStatus::Suspended, 100);
        assert!(
            advisory_privileges_for(
                Some("lodge_hart_king"),
                &[suspended],
                Some(StrategicMinute::new(100))
            )
            .is_empty()
        );
    }

    #[test]
    fn forage_attestation_uses_generated_spacetime_wire_field_names() {
        let encoded = serde_json::to_value(WireForageEnvironmentAttestation {
            package_digest: "digest",
            latitude_e_7: 517_500_000,
            longitude_e_7: 97_500_000,
            context_kind: "settlement",
            context_id: "dev-scenario-foraging",
            plains: 0,
            forest: 1_000,
            hills: 0,
            wetlands: 0,
            river_or_wet_ground: false,
            sea_or_coast: false,
            cultivated: false,
        })
        .unwrap();
        assert_eq!(
            encoded,
            json!({
                "package_digest": "digest",
                "latitude_e_7": 517_500_000,
                "longitude_e_7": 97_500_000,
                "context_kind": "settlement",
                "context_id": "dev-scenario-foraging",
                "plains": 0,
                "forest": 1_000,
                "hills": 0,
                "wetlands": 0,
                "river_or_wet_ground": false,
                "sea_or_coast": false,
                "cultivated": false,
            })
        );
        let generated = include_str!(
            "../../../adventuresim-stdb-client/src/forage_environment_attestation_type.rs"
        );
        assert!(generated.contains("pub latitude_e_7: i32"));
        assert!(generated.contains("pub longitude_e_7: i32"));
    }

    #[test]
    fn receipt_lookup_is_exact_for_character_and_opaque_request() {
        let request = "a".repeat(64);
        assert_eq!(
            ForageReceiptReference::try_from(request.as_str())
                .unwrap()
                .query(17.into())
                .to_string(),
            format!(
                "SELECT * FROM backend_forage_receipts WHERE character_id = 17 AND request_id = '{request}'"
            )
        );
    }

    #[test]
    fn request_ids_are_unique_and_opaque() {
        let first = ForageReceiptReference::issue(17.into());
        let second = ForageReceiptReference::issue(17.into());
        assert_eq!(first.to_string().len(), 64);
        assert_ne!(first, second);
    }

    #[test]
    fn result_redirects_reopen_authoritative_receipts_in_the_integrated_dialog() {
        let request = "a".repeat(64);
        let reference = ForageReceiptReference::try_from(request.as_str()).unwrap();
        assert!(ForageReceiptReference::try_from("../client-feedback").is_err());
        assert_eq!(
            ForageDialogDestination::receipt(
                crate::routes::return_url::LocalReturnUrl::try_from("/locations/camp").unwrap(),
                &reference
            )
            .to_string(),
            format!(
                "{}?forage=true&forage_receipt={}",
                crate::location_urls::patterns::CAMP.url([]),
                crate::location_urls::encode_component(&request.to_string())
            )
        );
        assert_eq!(
            ForageDialogDestination::receipt(
                crate::routes::return_url::LocalReturnUrl::try_from(
                    "/locations/settlement/lubeck/party/17?building=public-square"
                )
                .unwrap(),
                &reference
            )
            .to_string(),
            format!(
                "{}?building=public-square&forage=true&forage_receipt={}",
                crate::location_urls::patterns::PARTY_PERSONAL.url([
                    &("settlement"),
                    &("lubeck"),
                    &("17")
                ]),
                crate::location_urls::encode_component(&request.to_string())
            )
        );
    }

    #[test]
    fn failures_reopen_the_integrated_dialog_with_allowlisted_feedback() {
        assert_eq!(
            ForageRouteError::database(
                ForageReadStage::Execute,
                7.into(),
                crate::spacetimedb::SpacetimeError::Remote(
                    crate::spacetimedb::RemoteDatabaseFailure::from_response(
                        crate::spacetimedb::DatabaseOperation::Reducer,
                        reqwest::StatusCode::CONFLICT,
                        Ok("invalid target item".into())
                    )
                )
            )
            .feedback()
            .to_string(),
            "unavailable"
        );
        assert_eq!(
            ForageRouteError::TerrainUnavailable.feedback().to_string(),
            "unavailable"
        );
        assert_eq!(
            ForageRouteError::database(
                ForageReadStage::Execute,
                7.into(),
                crate::spacetimedb::SpacetimeError::Remote(
                    crate::spacetimedb::RemoteDatabaseFailure::from_response(
                        crate::spacetimedb::DatabaseOperation::Reducer,
                        reqwest::StatusCode::CONFLICT,
                        Ok("database exploded".into())
                    )
                )
            )
            .feedback()
            .to_string(),
            "unavailable"
        );
        assert_eq!(ForageFeedback::from_http("../raw-error"), None);
        assert_eq!(
            ForageDialogDestination::failure(
                crate::routes::return_url::LocalReturnUrl::try_from("/locations/camp").unwrap(),
                ForageFeedback::from_http("../raw-error").unwrap_or(ForageFeedback::Unavailable)
            )
            .to_string(),
            "/locations/camp?forage=true&forage_error=unavailable"
        );
        assert_eq!(
            ForageDialogDestination::failure(
                crate::routes::return_url::LocalReturnUrl::try_from(
                    "/locations/settlement/lubeck/party/17?building=public-square"
                )
                .unwrap(),
                ForageFeedback::Targets
            )
            .to_string(),
            "/locations/settlement/lubeck/party/17?building=public-square&forage=true&forage_error=targets"
        );
    }
}
