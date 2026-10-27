//! Noisy testimony perception and strategic responses to atomic claims.

use super::{SocialActionKind, SocialAttempt, SocialTopic, resolve_social_attempt};
use crate::personality::{Mirth, Transparency};
use crate::skill::Skill;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimAssessmentDirection {
    Unknown,
    LikelyFalse,
    LikelyTrue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClaimAssessment {
    pub direction: ClaimAssessmentDirection,
    /// Bounded presentation strength. This is a noisy demeanor perception, not
    /// confidence in factual accuracy.
    pub strength: f32,
}

/// Produce a fallible observer perception for one atomic claim.
///
/// Noise deliberately dominates weak checks, so any demeanor can produce any
/// colored direction. Better Insight strengthens a non-ambiguous private
/// demeanor signal without ever making it certain.
pub fn assess_testimony_claim(
    demeanor_truth_signal: f32,
    insight_check: f32,
    roll: f32,
) -> ClaimAssessment {
    let insight = insight_check.clamp(0.0, 5.0);
    let noise = (roll.clamp(0.0, 1.0) * 2.0 - 1.0) * 0.75;
    let signal = demeanor_truth_signal.clamp(-1.0, 1.0) * (0.1 + insight * 0.06) + noise;
    let absolute = signal.abs();
    let direction = if absolute < 0.18 {
        ClaimAssessmentDirection::Unknown
    } else if signal < 0.0 {
        ClaimAssessmentDirection::LikelyFalse
    } else {
        ClaimAssessmentDirection::LikelyTrue
    };
    ClaimAssessment {
        direction,
        strength: ((absolute - 0.18).max(0.0) / 0.82).clamp(0.0, 1.0),
    }
}

/// Owns witness request keys and skill selection. Witness leverage, truth, and
/// affinity policy remain distinct from ordinary companion social actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimChallengeApproach {
    Charm,
    Command,
    Bluff,
}

impl ClaimChallengeApproach {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Charm => "charm",
            Self::Command => "command",
            Self::Bluff => "bluff",
        }
    }

    pub const fn skill(self) -> Skill {
        self.social_action().skill(false)
    }

    const fn social_action(self) -> SocialActionKind {
        match self {
            Self::Charm => SocialActionKind::LightenMood,
            Self::Command => SocialActionKind::Rally,
            Self::Bluff => SocialActionKind::Reframe,
        }
    }

    pub const fn leverage(self) -> f32 {
        match self {
            Self::Charm => 0.0,
            Self::Command => 0.45,
            Self::Bluff => 0.9,
        }
    }

    pub const fn failure_affinity_loss(self) -> f32 {
        match self {
            Self::Charm => -0.8,
            Self::Command => -1.4,
            Self::Bluff => -2.5,
        }
    }
}

/// The request key does not name a witness challenge approach.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseClaimChallengeApproachError;

impl fmt::Display for ParseClaimChallengeApproachError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Unknown witness approach")
    }
}

impl std::error::Error for ParseClaimChallengeApproachError {}

impl FromStr for ClaimChallengeApproach {
    type Err = ParseClaimChallengeApproachError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(value))
            .map_err(|_| ParseClaimChallengeApproachError)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ClaimChallengeInput {
    pub approach: ClaimChallengeApproach,
    pub claim_is_factually_accurate: bool,
    pub skill_check: f32,
    pub affinity: f32,
    pub familiarity_hours: f32,
    /// Current settled NPC morale in the ordinary -100..=100 strategic range.
    /// It contributes at most +/-0.12 chance before the common clamp.
    pub current_morale: f32,
    pub target_transparency: Transparency,
    pub target_mirth: Mirth,
    pub roll: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClaimChallengeOutcome {
    pub succeeded: bool,
    pub morale_delta: f32,
    pub affinity_delta: f32,
}

/// Resolve a response to one atomic claim. Truthful claims always use the same
/// safe failed-challenge result as an insufficient check against an untrue
/// claim, so callers cannot infer why a response failed.
pub fn resolve_claim_challenge(input: ClaimChallengeInput) -> ClaimChallengeOutcome {
    let personality_fit = match (
        input.approach,
        input.target_transparency,
        input.target_mirth,
    ) {
        (ClaimChallengeApproach::Charm, _, Mirth::Merry) => 0.45,
        (ClaimChallengeApproach::Charm, _, Mirth::Grave) => -0.45,
        (ClaimChallengeApproach::Command, Transparency::Open, _) => -0.3,
        (ClaimChallengeApproach::Command, Transparency::Guarded, _) => 0.25,
        (ClaimChallengeApproach::Bluff, Transparency::Open, _) => 0.25,
        (ClaimChallengeApproach::Bluff, Transparency::Guarded, _) => -0.35,
        _ => 0.0,
    };
    let sensitivity = match input.target_transparency {
        Transparency::Open => 0.2,
        Transparency::Neutral => 0.5,
        Transparency::Guarded => 0.85,
    };
    let morale_fit = input.current_morale.clamp(-100.0, 100.0) / 100.0 * 1.5;
    let outcome = resolve_social_attempt(SocialAttempt {
        action: input.approach.social_action(),
        topic: SocialTopic::Defeat,
        skill_check: input.skill_check + personality_fit + morale_fit + input.approach.leverage(),
        affinity: input.affinity,
        familiarity_hours: input.familiarity_hours,
        diagnosis_correct: None,
        sensitivity,
        roll: input.roll,
    });
    let succeeded = !input.claim_is_factually_accurate && outcome.succeeded;
    let affinity_delta = if succeeded {
        if input.approach == ClaimChallengeApproach::Command {
            -0.4
        } else {
            0.0
        }
    } else {
        input.approach.failure_affinity_loss()
    };
    ClaimChallengeOutcome {
        succeeded,
        morale_delta: if succeeded {
            outcome.morale_delta
        } else {
            outcome.morale_delta.min(0.0)
        },
        affinity_delta,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_keys_select_canonical_executing_skills() {
        for (key, approach, skill) in [
            ("charm", ClaimChallengeApproach::Charm, Skill::Charm),
            ("command", ClaimChallengeApproach::Command, Skill::Command),
            ("bluff", ClaimChallengeApproach::Bluff, Skill::Deception),
        ] {
            assert_eq!(key.parse::<ClaimChallengeApproach>(), Ok(approach));
            assert_eq!(approach.stable_id(), key);
            assert_eq!(approach.skill(), skill);
            assert_eq!(
                serde_json::to_string(&approach).unwrap(),
                format!("\"{key}\"")
            );
        }
    }

    #[test]
    fn malformed_and_ordinary_social_action_keys_are_not_witness_approaches() {
        for key in [
            "",
            "Charm",
            "charm ",
            "deception",
            "lighten_mood",
            "unknown",
        ] {
            assert_eq!(
                key.parse::<ClaimChallengeApproach>(),
                Err(ParseClaimChallengeApproachError)
            );
        }
    }
}
