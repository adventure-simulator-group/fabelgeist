use super::{SettlementPlace, patterns, require_canonical_location_path, with_query};
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{Path, Query},
    http::{Method, Request, StatusCode},
    middleware,
    response::Redirect,
    routing::{get, post},
};
use std::collections::HashMap;
use tower::ServiceExt;

#[test]
fn production_location_routers_merge_without_conflicting_parameters() {
    let _router = crate::routes::settlements::routes()
        .merge(crate::routes::quests::routes())
        .merge(crate::routes::dialogue::routes())
        .merge(crate::routes::evidence::routes());
}

fn boundary_router() -> Router {
    Router::new()
        .route(
            patterns::SETTLEMENT.pattern(),
            get(
                |Path(id): Path<String>, Query(query): Query<HashMap<String, String>>| async move {
                    format!(
                        "{id}|{}",
                        query.get("destination").map_or("", String::as_str)
                    )
                },
            ),
        )
        .route(
            patterns::SETTLEMENT_FIREPLACE.pattern(),
            get(|Path((id, place)): Path<(String, String)>| async move { format!("{id}|{place}") }),
        )
        .route(
            patterns::REST.pattern(),
            post(|Path((id, place)): Path<(String, String)>| async move {
                Redirect::to(&patterns::SETTLEMENT_PLACE.url([&id, &place]))
            }),
        )
        .layer(middleware::from_fn(require_canonical_location_path))
}

async fn request(method: Method, uri: &str) -> axum::response::Response {
    boundary_router()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response")
}

#[tokio::test]
async fn encoded_location_and_destination_survive_axum_extraction() {
    let id = "site:town/雪 #?%";
    let destination = "site:nearby/?#";
    let url = with_query(&patterns::SETTLEMENT.url([&id]), "destination", destination);
    let response = request(Method::GET, &url).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024).await.expect("body");
    assert_eq!(body.as_ref(), format!("{id}|{destination}").as_bytes());
}

#[tokio::test]
async fn place_identity_is_in_the_path_and_actions_return_to_the_same_place() {
    let response = request(
        Method::GET,
        "/locations/settlement/willowmere/places/inn/fireplace?building=forge",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        to_bytes(response.into_body(), 1024)
            .await
            .expect("body")
            .as_ref(),
        b"willowmere|inn"
    );
    let response = request(
        Method::POST,
        "/locations/settlement/willowmere/places/inn/rest",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers()["location"],
        "/locations/settlement/willowmere/places/inn"
    );
    assert_eq!(
        request(
            Method::GET,
            "/locations/settlement/willowmere/places/inn/rest"
        )
        .await
        .status(),
        StatusCode::METHOD_NOT_ALLOWED
    );
}

#[tokio::test]
async fn obsolete_routes_and_place_aliases_are_rejected() {
    for path in [
        "/settlements/willowmere/inn",
        "/locations/settlement/willowmere/map",
        "/locations/settlement/willowmere/fireplace?building=inn",
        "/locations/settlement/willowmere/places/weapons/fireplace",
        "/locations/settlement/willowmere/places/overview/fireplace",
        "/locations/settlement/willowmere/places/unknown/fireplace",
    ] {
        assert_eq!(
            request(Method::GET, path).await.status(),
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
}

#[test]
fn chapter_urls_require_the_exact_local_chapter() {
    let catalog = adventuresim_core::organization::catalog();
    let chapter = catalog
        .organizations
        .iter()
        .flat_map(|organization| &organization.chapters)
        .next()
        .expect("catalog chapter");
    assert!(matches!(
        SettlementPlace::parse(&chapter.settlement_id, &chapter.location_id),
        Some(SettlementPlace::Chapter(_))
    ));
    assert!(SettlementPlace::parse("not-the-chapter-settlement", &chapter.location_id).is_none());
    assert!(SettlementPlace::parse(&chapter.settlement_id, "../inn").is_none());
}

#[test]
fn building_queries_preserve_existing_queries_and_fragments() {
    assert_eq!(
        with_query(
            "/locations/settlement/t/party/7?medical=surgery#limb",
            "building",
            "inn"
        ),
        "/locations/settlement/t/party/7?medical=surgery&building=inn#limb"
    );
}
