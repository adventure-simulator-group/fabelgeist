//! Exercise the actual route and session extractor against a read-only DB spy.
use super::*;
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::ws::WebSocketUpgrade,
    http::{HeaderMap, Request},
    routing::{get, post},
};
use base64::Engine;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

#[derive(Default)]
struct DatabaseSpy {
    owner: String,
    granted: bool,
    selected: bool,
    pin_granted: bool,
    raiding_allowed: bool,
    requests: Vec<String>,
}

type Spy = Arc<Mutex<DatabaseSpy>>;

fn rows(row: Value) -> Value {
    rows_many([row])
}

fn rows_many(rows: impl IntoIterator<Item = Value>) -> Value {
    let rows = rows.into_iter().collect::<Vec<_>>();
    let fields = rows.first().unwrap().as_object().unwrap();
    let names = fields.keys().cloned().collect::<Vec<_>>();
    json!([{"schema": {"elements": names.iter().map(|name| json!({"name": {"some": name}, "algebraic_type": {}})).collect::<Vec<_>>()},
        "rows": rows.into_iter().map(|row| {
            let fields = row.as_object().unwrap();
            names.iter().map(|name| fields[name].clone()).collect::<Vec<_>>()
        }).collect::<Vec<_>>() }])
}

fn case_site_pin(owner_character_id: u64, raiding_allowed: bool) -> Value {
    json!({"owner_character_id":owner_character_id, "case_id":"observer:alias", "case_site_id":{"value":"site:clearing"},
        "origin_settlement_id":"town", "name":"Clearing", "description":"", "scene_key":"outdoors",
        "longitude_e_7":0, "latitude_e_7":0, "coordinates_are_geographic":false, "distance_m":1000,
        "raiding_allowed":raiding_allowed, "knowledge_stage":{"ExactBelieved":[]}, "tracked":false,
        "display_title":"Clearing", "generated_case":false, "case_resolved":false,
        "combat_available":false, "opposition_count":{"none":[]}, "opposition_combat_power":{"none":[]}})
}

fn empty_rows() -> Value {
    json!([])
}

async fn query(State(spy): State<Spy>, body: String) -> Json<Value> {
    let mut spy = spy.lock().unwrap();
    spy.requests.push(body.clone());
    let row = if body.contains("backend_browser_character_access") {
        if !spy.granted {
            return Json(json!([]));
        }
        json!({"owner_key": spy.owner, "character_id": 7, "selected": spy.selected})
    } else if body.contains("backend_characters") {
        json!({"id":7, "scan_id":7, "name":"Ada", "xp":0, "level":1,
            "current_settlement_id":{"none":[]}, "party_id":{"none":[]},
            "server":["0x0"], "in_server":false, "temporary":false, "age_years":30,
            "alive":true, "party_treatment_decision":{"Allowed":[]}})
    } else if body.contains("backend_character_case_site_locations") {
        json!({"character_id":7, "case_site_id":{"value":"site:clearing"}})
    } else if body.contains("backend_case_site_pins") {
        if !body.contains("owner_character_id = 7") {
            return Json(rows_many([
                case_site_pin(7, spy.raiding_allowed),
                case_site_pin(8, false),
            ]));
        }
        if !spy.pin_granted {
            return Json(empty_rows());
        }
        case_site_pin(7, spy.raiding_allowed)
    } else {
        panic!("unexpected read or mutation: {body}");
    };
    Json(rows(row))
}

async fn socket(ws: WebSocketUpgrade, headers: HeaderMap) -> Response {
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

struct Harness {
    app: Router,
    spy: Spy,
    token: String,
    server: tokio::task::JoinHandle<()>,
}
impl Harness {
    async fn new() -> Self {
        let codec = Arc::new(
            crate::session::SessionCodec::from_base64url(
                &base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([7_u8; 32]),
                false,
            )
            .unwrap(),
        );
        let issued = codec.issue().unwrap();
        let spy = Arc::new(Mutex::new(DatabaseSpy {
            owner: issued.owner_key,
            granted: true,
            selected: true,
            pin_granted: true,
            raiding_allowed: true,
            ..Default::default()
        }));
        let mock = Router::new()
            .route("/v1/database/test/sql", post(query))
            .route("/v1/database/test/subscribe", get(socket))
            .fallback(
                |State(spy): State<Spy>, request: Request<Body>| async move {
                    spy.lock().unwrap().requests.push(format!(
                        "{} {}",
                        request.method(),
                        request.uri()
                    ));
                    StatusCode::NOT_FOUND
                },
            )
            .with_state(spy.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, mock).await.unwrap();
        });
        let state = AppState {
            db: crate::spacetimedb::SpacetimeClient::new(&address, "test").unwrap(),
            live: crate::live::LiveState::connect(&address, "test", None).unwrap(),
            strategic_map: None,
            terrain: None,
            session_codec: codec,
        };
        let app = Router::new()
            .route(
                "/locations/{kind}/{id}/party/{character_id}/schedule/preview",
                post(preview_training_schedule),
            )
            .with_state(state);
        Self {
            app,
            spy,
            token: issued.token,
            server,
        }
    }
    async fn request(
        &self,
        character: u64,
        site: &str,
        token: Option<&str>,
        allocation: &str,
    ) -> (StatusCode, Value) {
        let mut request = Request::post(format!(
            "/locations/case-site/{site}/party/{character}/schedule/preview"
        ))
        .header("content-type", "application/x-www-form-urlencoded");
        if let Some(token) = token {
            request = request.header("cookie", format!("adventuresim_session={token}"));
        }
        let response = self
            .app
            .clone()
            .oneshot(request.body(Body::from(allocation.to_owned())).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), 65536).await.unwrap();
        (
            status,
            serde_json::from_slice(&body).unwrap_or_else(|_| json!(String::from_utf8_lossy(&body))),
        )
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        self.server.abort();
    }
}

const ALLOCATION: &str = "reading_minutes=60&socializing_minutes=30&combat_training_minutes=30&raiding_minutes=120&thievery_minutes=90";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn denied_previews_never_read_character_context() {
    let h = Harness::new().await;
    for token in [None, Some("forged")] {
        assert_eq!(
            h.request(7, "site:clearing", token, ALLOCATION).await.0,
            StatusCode::FORBIDDEN
        );
    }
    assert!(h.spy.lock().unwrap().requests.is_empty());
    assert_eq!(
        h.request(8, "site:clearing", Some(&h.token), ALLOCATION)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    h.spy.lock().unwrap().selected = false;
    assert_eq!(
        h.request(7, "site:clearing", Some(&h.token), ALLOCATION)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    h.spy.lock().unwrap().granted = false;
    assert_eq!(
        h.request(7, "site:clearing", Some(&h.token), ALLOCATION)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert!(
        h.spy
            .lock()
            .unwrap()
            .requests
            .iter()
            .all(|query| query.contains("backend_browser_character_access"))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invalid_allocation_and_location_stop_before_reading_location_context() {
    let h = Harness::new().await;
    assert_eq!(
        h.request(7, "site:clearing", Some(&h.token), "labor_minutes=1")
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    assert!(
        h.spy
            .lock()
            .unwrap()
            .requests
            .iter()
            .all(|query| query.contains("backend_browser_character_access"))
    );
    assert_eq!(
        h.request(7, "site:elsewhere", Some(&h.token), ALLOCATION)
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert!(
        h.spy
            .lock()
            .unwrap()
            .requests
            .iter()
            .all(|query| !query.contains("backend_case_site_pins"))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn proposed_allocation_is_read_only_and_uses_current_server_eligibility() {
    use adventuresim_core::strategic_schedule::{DailySchedule, ValidatedSchedule};
    let h = Harness::new().await;
    let proposed = DailySchedule {
        reading_minutes: 60,
        socializing_minutes: 30,
        combat_training_minutes: 30,
        raiding: 120,
        thievery: 90,
        ..Default::default()
    };
    for (allowed, policy) in [
        (true, ActivityLocation::NamedOutdoorLocation),
        (false, ActivityLocation::IneligibleNamedLocation),
    ] {
        h.spy.lock().unwrap().raiding_allowed = allowed;
        let (status, preview) = h
            .request(7, "site:clearing", Some(&h.token), ALLOCATION)
            .await;
        assert_eq!(status, StatusCode::OK, "{preview}");
        let execution = ValidatedSchedule::try_from(proposed)
            .unwrap()
            .effective_at(policy, 7);
        assert_eq!(
            preview["effective"],
            serde_json::to_value(execution).unwrap()
        );
        assert_eq!(
            preview["leisure_minutes"],
            1440 - execution.allocated_minutes()
        );
    }
    let (status, preview) = h
        .request(
            7,
            "site:clearing",
            Some(&h.token),
            "thievery_minutes=120&reading_minutes=60",
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert_eq!(preview["effective"]["thievery"], 0);
    assert_eq!(preview["effective"]["reading_minutes"], 60);
    assert_eq!(preview["leisure_minutes"], 1380);
    assert!(
        h.spy
            .lock()
            .unwrap()
            .requests
            .iter()
            .all(|query| query.starts_with("SELECT "))
    );
    assert!(
        !h.spy
            .lock()
            .unwrap()
            .requests
            .iter()
            .any(|query| query.contains("training_schedule"))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shared_case_site_lookup_is_owner_scoped_and_requires_the_owners_pin() {
    let h = Harness::new().await;
    assert_eq!(
        h.request(7, "site:clearing", Some(&h.token), ALLOCATION)
            .await
            .0,
        StatusCode::OK
    );
    assert!(
        h.spy
            .lock()
            .unwrap()
            .requests
            .iter()
            .filter(|query| query.contains("backend_case_site_pins"))
            .all(|query| query.contains("owner_character_id = 7"))
    );
    h.spy.lock().unwrap().pin_granted = false;
    assert_eq!(
        h.request(7, "site:clearing", Some(&h.token), ALLOCATION)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}
