//! Route handlers
mod coordinates;
use coordinates::{wgs84_e7, wgs84_latitude_longitude_degrees};

pub mod challenges;
pub mod characters;
mod clock;
mod data;
pub mod developer_quests;
pub mod dialogue;
pub mod evidence;
pub mod foraging;
pub mod home;
mod inventory_forms;
pub mod investigation;
pub mod local_chat;
pub mod missions;
pub mod parties;
mod party_actions;
mod party_readiness;
pub mod quests;
mod return_url;
mod scene_assets;
mod scene_equipment;
mod vicinity;
use clock::current_time;
pub mod settlements;
pub(crate) mod travel;
mod weapon_icons;
use crate::live::LiveState;
use crate::session::{Session, SessionCodec};
use crate::spacetimedb::{
    CharacterTime, CharacterView, PartyJourneyRouteView, PartyView, SettlementView, SpacetimeClient,
};
use axum::{
    Router,
    extract::{Request, State},
    http::{Method, StatusCode, Uri, header},
    middleware::{self, Next},
    response::{IntoResponse, Json, Redirect, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};

/// Application state shared across routes
#[derive(Clone)]
pub struct AppState {
    pub db: SpacetimeClient,
    pub live: LiveState,
    pub strategic_map: Option<std::sync::Arc<crate::strategic_map::StrategicMap>>,
    pub terrain: Option<std::sync::Arc<travel::TerrainPlanner>>,
    pub session_codec: std::sync::Arc<SessionCodec>,
}

#[cfg(test)]
use party_actions::terrain_profile::terrain_mental_check;
pub(crate) use party_actions::{
    PartyAction, PartyActionError, approve_party_action, character_case_site_id,
    execute_or_request_party_action, party_terrain_profile,
};

pub(crate) enum PartyActionOutcome {
    Executed,
    Requested,
}

/// A validated ordinary-conversation duration. Parsing at the HTTP boundary
/// prevents invalid minute counts from entering route logic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SocialDuration(u16);

impl SocialDuration {
    pub(crate) const fn minutes(self) -> u64 {
        self.0 as u64
    }
}

impl TryFrom<u64> for SocialDuration {
    type Error = &'static str;

    fn try_from(minutes: u64) -> Result<Self, Self::Error> {
        if (15..=8 * 60).contains(&minutes) && minutes.is_multiple_of(15) {
            Ok(Self(minutes as u16))
        } else {
            Err("choose 15 minutes to 8 hours in 15-minute increments")
        }
    }
}

impl<'de> Deserialize<'de> for SocialDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::try_from(u64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Opaque idempotency key accepted from the browser after validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SocialActionId(String);

impl SocialActionId {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SocialActionId {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.is_empty()
            && value.len() <= 96
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            Ok(Self(value))
        } else {
            Err("invalid conversation action ID")
        }
    }
}

impl<'de> Deserialize<'de> for SocialActionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::try_from(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

pub(crate) fn redirect_to_local(return_to: &str, fallback: &str) -> Redirect {
    match return_url::LocalReturnUrl::try_from(return_to) {
        Ok(destination) => destination.redirect(),
        Err(_) => Redirect::to(fallback),
    }
}

#[cfg(test)]
mod return_url_tests {
    use super::{return_url::LocalReturnUrl, terrain_mental_check};

    #[test]
    fn return_urls_are_local_paths_with_optional_query_and_fragment() {
        assert_eq!(
            LocalReturnUrl::try_from(
                "/locations/settlement/riverdale?destination=quest-1&target_surplus=1.5#plan"
            )
            .unwrap()
            .to_string(),
            "/locations/settlement/riverdale?destination=quest-1&target_surplus=1.5#plan"
        );
        assert!(LocalReturnUrl::try_from("https://example.com/steal").is_err());
        assert!(LocalReturnUrl::try_from("//example.com/steal").is_err());
        assert!(LocalReturnUrl::try_from("/\\example.com/steal").is_err());
        assert!(LocalReturnUrl::try_from("merchants").is_err());
        assert!(LocalReturnUrl::try_from("/safe\nLocation: /unsafe").is_err());
    }

    #[test]
    fn party_purchase_funding_uses_shared_coin_before_personal_coin() {
        use adventuresim_core::strategic_economy::split_party_purchase_payment;

        assert_eq!(split_party_purchase_payment(8, 20, 15), Some((8, 7)));
        assert_eq!(split_party_purchase_payment(20, 8, 15), Some((15, 0)));
        assert_eq!(split_party_purchase_payment(4, 5, 10), None);
    }

    #[test]
    fn terrain_mental_check_applies_authoritative_head_health() {
        let healthy = terrain_mental_check(2.0, 2.0, 1.0);
        let injured = terrain_mental_check(2.0, 2.0, 0.5);
        let destroyed = terrain_mental_check(2.0, 2.0, 0.0);
        assert_eq!(healthy, 2.0);
        assert_eq!(injured, 1.0);
        assert_eq!(destroyed, 0.0);
    }
}

pub(crate) fn persisted_route_position(
    route: &PartyJourneyRouteView,
    minute: u64,
) -> Option<(f64, f64)> {
    let coordinate = |point: &crate::spacetimedb::JourneyRoutePoint| {
        wgs84_latitude_longitude_degrees(point.latitude_e_7, point.longitude_e_7)
            .expect("persisted journey route coordinates must be valid WGS84")
    };
    let distance = |from: (f64, f64), to: (f64, f64)| {
        let earth_radius_m = 6_371_000.0_f64;
        let lat1 = from.0.to_radians();
        let lat2 = to.0.to_radians();
        let delta_lat = (to.0 - from.0).to_radians();
        let delta_lon = (to.1 - from.1).to_radians();
        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
        (earth_radius_m * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())).round() as u64
    };
    let lengths = route
        .points
        .windows(2)
        .map(|pair| distance(coordinate(&pair[0]), coordinate(&pair[1])))
        .collect::<Vec<_>>();
    let total = lengths.iter().sum::<u64>();
    if total == 0 || route.minutes == 0 {
        return route.points.first().map(coordinate);
    }
    let target = total.saturating_mul(minute.min(route.minutes)) / route.minutes;
    let mut traversed = 0_u64;
    for (index, length) in lengths.into_iter().enumerate() {
        if traversed.saturating_add(length) >= target {
            let from = coordinate(&route.points[index]);
            let to = coordinate(&route.points[index + 1]);
            let fraction = if length == 0 {
                0.0
            } else {
                target.saturating_sub(traversed) as f64 / length as f64
            };
            return Some((
                from.0 + (to.0 - from.0) * fraction,
                from.1 + (to.1 - from.1) * fraction,
            ));
        }
        traversed = traversed.saturating_add(length);
    }
    route.points.last().map(coordinate)
}

/// Build the complete router
pub fn build_router(state: AppState) -> Router {
    let middleware_state = state.clone();
    Router::new()
        .route(
            crate::strategic_map::DATA_LICENSE_PATH,
            get(crate::strategic_map::data_license),
        )
        .route(
            "/map/tiles/{theme}/{zoom}/{x}/{tile}",
            get(crate::strategic_map::world_tile),
        )
        .merge(characters::routes().layer(middleware::from_fn(require_same_origin_mutation)))
        .merge(home::routes())
        .merge(
            Router::new()
                .merge(investigation::routes())
                .merge(challenges::routes())
                .merge(dialogue::routes())
                .merge(developer_quests::routes())
                .merge(evidence::routes())
                .merge(foraging::routes())
                .merge(local_chat::routes())
                .merge(settlements::routes())
                .merge(parties::routes())
                .merge(quests::routes())
                .merge(missions::routes())
                .merge(weapon_icons::routes())
                .merge(scene_equipment::routes())
                .merge(scene_assets::routes())
                .merge(crate::live::routes())
                .route("/time", get(current_time))
                .layer(middleware::from_fn(require_same_origin_mutation))
                .layer(middleware::from_fn_with_state(
                    middleware_state,
                    require_active_character,
                )),
        )
        .with_state(state)
}

/// Strategic screens have no anonymous mode. Character creation and selection
/// remain public entry screens; every other route requires a selected character.
async fn require_active_character(session: Session, request: Request, next: Next) -> Response {
    if session.character_id_u64().is_none() {
        return Redirect::to("/characters").into_response();
    }
    next.run(request).await
}

/// The opaque browser session is bearer authority, so every browser mutation
/// in the onboarding or active-character route groups must originate from this
/// exact web origin. SameSite cookies alone do not stop a different service on
/// the same site (for example, another localhost port) from submitting a form.
///
/// Non-mutating internal strategic navigation remains unaffected. There are no
/// non-browser mutation endpoints in this protected router; any future one
/// must receive a separately authenticated route rather than bypass this
/// browser-origin boundary.
async fn require_same_origin_mutation(request: Request, next: Next) -> Response {
    if is_browser_mutation(request.method()) && !has_same_origin(&request) {
        return (
            StatusCode::FORBIDDEN,
            "Cross-origin strategic mutation rejected",
        )
            .into_response();
    }
    next.run(request).await
}

fn is_browser_mutation(method: &Method) -> bool {
    method == Method::POST
        || method == Method::PUT
        || method == Method::PATCH
        || method == Method::DELETE
}

fn has_same_origin(request: &Request) -> bool {
    let Some(origin) = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .filter(|value| *value != "null")
    else {
        return false;
    };
    let Ok(origin) = origin.parse::<Uri>() else {
        return false;
    };
    if origin.path() != "/" || origin.query().is_some() {
        return false;
    }
    let Some(origin_scheme) = origin.scheme_str() else {
        return false;
    };
    if !matches!(origin_scheme, "http" | "https") {
        return false;
    }
    let Some(origin_authority) = origin.authority().map(|value| value.as_str()) else {
        return false;
    };
    let Some(host) = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let request_scheme = request
        .uri()
        .scheme_str()
        .or_else(|| {
            request
                .headers()
                .get("x-forwarded-proto")
                .and_then(|value| value.to_str().ok())
                .filter(|value| !value.contains(','))
        })
        .unwrap_or("http");
    origin_scheme.eq_ignore_ascii_case(request_scheme)
        && origin_authority.eq_ignore_ascii_case(host)
}

#[cfg(test)]
mod onboarding_route_tests {
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, header},
    };

    use super::has_same_origin;

    #[test]
    fn home_route_is_merged_before_the_active_character_guard() {
        let source = include_str!("mod.rs");
        let home = source.find(".merge(home::routes())").unwrap();
        let protected = source.find(".merge(dialogue::routes())").unwrap();
        let guard = source
            .find(".layer(middleware::from_fn_with_state(")
            .unwrap();
        assert!(home < protected && protected < guard);
    }

    fn mutation(
        origin: Option<&str>,
        host: Option<&str>,
        forwarded_proto: Option<&str>,
    ) -> Request {
        let mut builder = Request::builder()
            .method(Method::POST)
            .uri("/locations/settlement/lubeck/places/residences/rent/cheap");
        if let Some(origin) = origin {
            builder = builder.header(header::ORIGIN, origin);
        }
        if let Some(host) = host {
            builder = builder.header(header::HOST, host);
        }
        if let Some(proto) = forwarded_proto {
            builder = builder.header("x-forwarded-proto", proto);
        }
        builder.body(Body::empty()).unwrap()
    }

    #[test]
    fn browser_mutations_require_an_exact_same_origin() {
        assert!(has_same_origin(&mutation(
            Some("http://127.0.0.1:8080"),
            Some("127.0.0.1:8080"),
            None,
        )));
        assert!(has_same_origin(&mutation(
            Some("https://game.example.test"),
            Some("game.example.test"),
            Some("https"),
        )));
        assert!(!has_same_origin(&mutation(
            Some("http://localhost:9000"),
            Some("localhost:8080"),
            None,
        )));
        assert!(!has_same_origin(&mutation(
            Some("null"),
            Some("127.0.0.1:8080"),
            None,
        )));
        assert!(!has_same_origin(&mutation(
            None,
            Some("127.0.0.1:8080"),
            None,
        )));
        assert!(!has_same_origin(&mutation(
            Some("https://game.example.test"),
            Some("game.example.test"),
            Some("http"),
        )));
    }
}
