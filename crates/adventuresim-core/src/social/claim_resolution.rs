//! Captured observer feedback for a resolved witness claim.

use serde::{Deserialize, Serialize};

use super::ClaimChallengeOutcome;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum WitnessClaimOutcome {
    UsefulAnswer,
    DidNotYield,
}

/// Immutable resolution captured after applying the relationship change.
/// Its presence is the claim's sole resolved-state authority. The affinity
/// delta records the realized change, including relationship-bound clamping;
/// it does not follow later changes to the relationship.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub struct WitnessClaimResolution {
    pub outcome: WitnessClaimOutcome,
    pub affinity_delta: f32,
}

impl WitnessClaimResolution {
    pub fn from_challenge(challenge: ClaimChallengeOutcome, realized_affinity_delta: f32) -> Self {
        Self {
            outcome: if challenge.succeeded {
                WitnessClaimOutcome::UsefulAnswer
            } else {
                WitnessClaimOutcome::DidNotYield
            },
            affinity_delta: realized_affinity_delta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feedback_captures_realized_delta_and_serializes_without_state_sentinels() {
        for (succeeded, label) in [(true, "useful_answer"), (false, "did_not_yield")] {
            let resolution = WitnessClaimResolution::from_challenge(
                ClaimChallengeOutcome {
                    succeeded,
                    morale_delta: 1.0,
                    affinity_delta: -0.4,
                },
                -0.1,
            );
            let value = serde_json::to_value(Some(resolution)).unwrap();
            assert_eq!(
                value,
                serde_json::json!({ "outcome": label, "affinity_delta": -0.1_f32 })
            );
            assert_eq!(
                serde_json::from_value::<Option<WitnessClaimResolution>>(value).unwrap(),
                Some(resolution)
            );
        }
        assert_eq!(
            serde_json::to_value(None::<WitnessClaimResolution>).unwrap(),
            serde_json::Value::Null
        );
    }
}
