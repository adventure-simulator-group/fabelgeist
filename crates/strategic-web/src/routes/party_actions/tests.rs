use super::*;
use axum::{
    Router,
    extract::{Json, State},
    http::{StatusCode, Uri},
    routing::post,
};
use std::sync::{Arc, Mutex};

struct CapturedReducer {
    uri: Uri,
    arguments: Value,
}

async fn receive_reducer(
    State(capture): State<Arc<Mutex<Option<CapturedReducer>>>>,
    uri: Uri,
    Json(arguments): Json<Value>,
) -> StatusCode {
    *capture.lock().unwrap() = Some(CapturedReducer { uri, arguments });
    StatusCode::OK
}

async fn capture_action(action: PartyAction, actor: CharacterId) -> CapturedReducer {
    let capture = Arc::new(Mutex::new(None));
    let router = Router::new()
        .route("/v1/database/test/call/{reducer}", post(receive_reducer))
        .with_state(capture.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await
            .unwrap();
    });
    let client = SpacetimeClient::new(format!("http://{address}"), "test").unwrap();
    action.execute(actor, &client).await.unwrap();
    stop.send(()).unwrap();
    server.await.unwrap();
    capture.lock().unwrap().take().unwrap()
}

#[tokio::test]
async fn approval_rebinds_actor_from_the_typed_variant() {
    let action = PartyAction::TravelToCaseSite {
        case_site_id: "case-site-7".into(),
    };
    let captured = capture_action(action, CharacterId::from(42)).await;
    assert_eq!(
        captured.uri.path(),
        "/v1/database/test/call/travel_to_case_site"
    );
    assert_eq!(captured.arguments, json!([42, { "value": "case-site-7" }]));
}

#[test]
fn action_payload_round_trips() {
    let action = PartyAction::CancelMission {
        mission_id: "mission-3".into(),
    };
    let encoded = serde_json::to_string(&action).unwrap();
    assert_eq!(
        serde_json::from_str::<PartyAction>(&encoded).unwrap(),
        action
    );
}

#[tokio::test]
async fn recruitment_role_payload_keeps_the_id_and_embeds_weapon_precision_in_requirements() {
    let requirements = RoleRequirements {
        weapon_precision: adventuresim_core::capability::WEAPON_PRECISION_SWORD,
        ..Default::default()
    };
    let edit = PartyAction::UpdateRecruitmentRole {
        role_id: 17,
        name: "Scout".into(),
        quantity: 1,
        requirements,
    };
    let delete = PartyAction::DeleteRecruitmentRole { role_id: 17 };
    assert_eq!(edit.kind(), PartyActionKind::UpdateRole);
    assert_eq!(delete.kind(), PartyActionKind::DeleteRole);
    assert!(
        serde_json::to_string(&edit)
            .unwrap()
            .contains("\"role_id\":17")
    );
    assert!(
        serde_json::to_string(&delete)
            .unwrap()
            .contains("\"role_id\":17")
    );
    let captured = capture_action(edit, CharacterId::from(42)).await;
    assert_eq!(
        captured.uri.path(),
        "/v1/database/test/call/update_recruitment_role"
    );
    assert_eq!(captured.arguments.as_array().unwrap().len(), 5);
    assert_eq!(captured.arguments[0], 42);
    assert_eq!(captured.arguments[1], 17);
    assert_eq!(
        captured.arguments[4]["weapon_precision"],
        adventuresim_core::capability::WEAPON_PRECISION_SWORD
    );
}

#[tokio::test]
async fn removal_keeps_full_width_character_identity_at_the_reducer_boundary() {
    let action = PartyAction::RemovePartyMember {
        character_id: CharacterId::from(u64::MAX),
    };
    let encoded = serde_json::to_value(&action).unwrap();
    assert_eq!(encoded["character_id"], json!(u64::MAX));
    assert_eq!(
        serde_json::from_value::<PartyAction>(encoded).unwrap(),
        action
    );
    let captured = capture_action(action, CharacterId::from(u64::MAX - 1)).await;
    assert_eq!(captured.arguments, json!([u64::MAX - 1, u64::MAX]));
    for invalid in [json!(-1), json!("7"), json!(1.5)] {
        assert!(
            serde_json::from_value::<PartyAction>(
                json!({"action":"remove_party_member", "character_id":invalid})
            )
            .is_err()
        );
    }
}

#[path = "readiness_tests.rs"]
mod readiness_tests;
