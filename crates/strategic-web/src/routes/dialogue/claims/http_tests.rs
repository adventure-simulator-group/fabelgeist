//! HTTP request admission uses the shared witness vocabulary before dispatch.
use super::*;
use axum::{
    Router,
    body::Body,
    http::Request,
    routing::{get, post},
};
use base64::Engine;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

type Calls = Arc<Mutex<Vec<serde_json::Value>>>;

async fn reject_dispatch(
    State(calls): State<Calls>,
    Json(args): Json<serde_json::Value>,
) -> StatusCode {
    calls.lock().unwrap().push(args);
    StatusCode::CONFLICT
}

async fn socket(
    ws: axum::extract::ws::WebSocketUpgrade,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let protocols = headers
        .get("sec-websocket-protocol")
        .unwrap()
        .to_str()
        .unwrap()
        .split(',')
        .map(str::trim)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    ws.protocols(protocols)
        .on_upgrade(|mut socket| async move { while socket.recv().await.is_some() {} })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn witness_http_admits_canonical_keys_and_rejects_unknown_keys_before_dispatch() {
    let codec = Arc::new(
        crate::session::SessionCodec::from_base64url(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([9_u8; 32]),
            false,
        )
        .unwrap(),
    );
    let issued = codec.issue().unwrap();
    let owner = issued.owner_key.clone();
    let calls: Calls = Arc::default();
    let mock = Router::new()
        .route("/v1/database/test/subscribe", get(socket))
        .route(
            "/v1/database/test/sql",
            post(move |body: String| {
                let owner = owner.clone();
                async move {
                    assert!(body.contains("backend_browser_character_access"));
                    Json(json!([{"schema":{"elements":[
                    {"name":{"some":"owner_key"},"algebraic_type":{}},
                    {"name":{"some":"character_id"},"algebraic_type":{}},
                    {"name":{"some":"selected"},"algebraic_type":{}}]},
                    "rows":[[owner,7,true]]}]))
                }
            }),
        )
        .route(
            "/v1/database/test/call/approach_dialogue_witness",
            post(reject_dispatch),
        )
        .with_state(calls.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, mock).await.unwrap();
    });
    let state = AppState {
        db: crate::spacetimedb::SpacetimeClient::new(&address, "test").unwrap(),
        live: crate::live::LiveState::connect(&address, "test", None).unwrap(),
        terrain: None,
        regional_roads: std::sync::Arc::new(crate::routes::RegionalRoadSource::new(
            std::path::PathBuf::new(),
        )),
        session_codec: codec,
    };
    let app = Router::new()
        .route("/witness", post(witness_approach))
        .with_state(state);
    for (key, expected) in [
        ("charm", StatusCode::CONFLICT),
        ("command", StatusCode::CONFLICT),
        ("bluff", StatusCode::CONFLICT),
        ("Charm", StatusCode::BAD_REQUEST),
        ("deception", StatusCode::BAD_REQUEST),
        ("", StatusCode::BAD_REQUEST),
    ] {
        let request = Request::post("/witness")
            .header("content-type", "application/json")
            .header("cookie", format!("adventuresim_session={}", issued.token))
            .body(Body::from(
                json!({"session_id":"dialogue:fixture",
                "challenge_token":"claim:fixture", "approach":key,
                "action_id":"action", "expected_revision":3})
                .to_string(),
            ))
            .unwrap();
        assert_eq!(
            app.clone().oneshot(request).await.unwrap().status(),
            expected,
            "{key}"
        );
    }
    assert_eq!(
        *calls.lock().unwrap(),
        ["charm", "command", "bluff"].map(|key| json!([
            7,
            "dialogue:fixture",
            "claim:fixture",
            key,
            "action",
            3
        ]))
    );
    server.abort();
}
