//! Observer-safe witness feedback at the SDK/browser boundary.

use super::*;
use adventuresim_core::social::ClaimChallengeApproach;

#[derive(Serialize)]
pub(super) struct ClaimView {
    challenge_token: String,
    charm_response: Option<String>,
    command_response: Option<String>,
    bluff_response: Option<String>,
    assessment_direction: String,
    assessment_strength: f32,
    resolution: Option<adventuresim_core::social::WitnessClaimResolution>,
}
pub(super) fn claim_view(
    event_sequence: u32,
    claim_order: u32,
    displayed_text: &str,
    claims: &[BackendDialogueWitnessClaim],
) -> Option<ClaimView> {
    let claim = claims.iter().find(|claim| {
        claim.event_sequence == event_sequence
            && claim.claim_order == claim_order
            && claim.displayed_text == displayed_text
    })?;
    Some(ClaimView {
        challenge_token: claim.challenge_token.clone(),
        charm_response: claim.charm_response.clone(),
        command_response: claim.command_response.clone(),
        bluff_response: claim.bluff_response.clone(),
        assessment_direction: claim.assessment_direction.clone(),
        assessment_strength: claim.assessment_strength.clamp(0.0, 1.0),
        resolution: claim.resolution.as_ref().map(|resolution| {
            use adventuresim_core::social::{WitnessClaimOutcome, WitnessClaimResolution};
            WitnessClaimResolution {
                outcome: match resolution.outcome {
                    adventuresim_stdb_client::WitnessClaimOutcome::UsefulAnswer => {
                        WitnessClaimOutcome::UsefulAnswer
                    }
                    adventuresim_stdb_client::WitnessClaimOutcome::DidNotYield => {
                        WitnessClaimOutcome::DidNotYield
                    }
                },
                affinity_delta: resolution.affinity_delta,
            }
        }),
    })
}

#[derive(Deserialize)]
pub(super) struct WitnessApproachRequest {
    session_id: String,
    challenge_token: String,
    approach: String,
    action_id: String,
    expected_revision: u64,
}

pub(super) async fn witness_approach(
    State(state): State<AppState>,
    session: Session,
    Json(request): Json<WitnessApproachRequest>,
) -> Result<Json<ConversationView>, StatusCode> {
    let character_id = session.character_id_u64().ok_or(StatusCode::UNAUTHORIZED)?;
    let approach = request
        .approach
        .parse::<ClaimChallengeApproach>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    state
        .db
        .call(
            "approach_dialogue_witness",
            &[
                json!(character_id),
                json!(&request.session_id),
                json!(&request.challenge_token),
                json!(approach.stable_id()),
                json!(request.action_id),
                json!(request.expected_revision),
            ],
        )
        .await
        .map_err(|_| StatusCode::CONFLICT)?;
    Ok(Json(
        build_view(&state, character_id, &request.session_id).await?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_feedback_preserves_optional_resolution_and_realized_delta() {
        let mut claim = super::super::tests::claim("heard", 0);
        let unresolved =
            serde_json::to_value(claim_view(4, 0, "heard", &[claim.clone()]).unwrap()).unwrap();
        assert!(unresolved["resolution"].is_null());
        assert!(unresolved.get("resolved").is_none());
        for (outcome, label) in [
            (
                adventuresim_stdb_client::WitnessClaimOutcome::UsefulAnswer,
                "useful_answer",
            ),
            (
                adventuresim_stdb_client::WitnessClaimOutcome::DidNotYield,
                "did_not_yield",
            ),
        ] {
            claim.resolution = Some(adventuresim_stdb_client::WitnessClaimResolution {
                outcome,
                affinity_delta: -0.5,
            });
            let value =
                serde_json::to_value(claim_view(4, 0, "heard", &[claim.clone()]).unwrap()).unwrap();
            assert_eq!(
                value["resolution"],
                json!({ "outcome": label, "affinity_delta": -0.5 })
            );
        }
    }
}

#[cfg(test)]
#[path = "claims/http_tests.rs"]
mod http_tests;
