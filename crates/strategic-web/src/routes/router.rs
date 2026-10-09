//! Compose public and authenticated strategic routes with browser guards.
use super::{
    AppState, challenges, characters, clock::current_time, developer_quests, dialogue, evidence,
    foraging, home, investigation, local_chat, map_city, map_environment, missions, parties,
    quests, scene_assets, scene_equipment, settlements, weapon_icons,
};
use crate::session::Session;
use axum::{
    Router,
    extract::Request,
    http::{Method, StatusCode, Uri, header},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};

/// An Origin header identifies an authority, with no document path.
const ORIGIN_ROOT_PATH: &str = "/";

/// Build the complete router
pub fn build_router(state: AppState) -> Router {
    let middleware_state = state.clone();
    Router::new()
        .route(
            crate::strategic_map::DATA_LICENSE_PATH,
            get(crate::strategic_map::data_license),
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
                .merge(map_environment::routes())
                .merge(map_city::routes())
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
    if origin.path() != ORIGIN_ROOT_PATH || origin.query().is_some() {
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
        let source = include_str!("router.rs");
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
