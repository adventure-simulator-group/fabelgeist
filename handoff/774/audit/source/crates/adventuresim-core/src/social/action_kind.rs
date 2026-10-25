//! Social action admission vocabulary and skill selection.
use super::SocialTopic;
use crate::skill::Skill;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Owns social request keys, topic eligibility, and the executing skill.
/// Persisted cooldowns and receipts capture reducer keys; labels derive from Skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SocialActionKind {
    Reflect,
    Listen,
    Commiserate,
    Pray,
    Reassure,
    LightenMood,
    #[serde(rename = "command")]
    Rally,
    #[serde(rename = "deception")]
    Reframe,
    Flirt,
}

impl SocialActionKind {
    pub const fn reducer_value(self) -> &'static str {
        match self {
            Self::Reflect => "reflect",
            Self::Listen => "listen",
            Self::Commiserate => "commiserate",
            Self::Pray => "pray",
            Self::Reassure => "reassure",
            Self::LightenMood => "lighten_mood",
            Self::Rally => "command",
            Self::Reframe => "deception",
            Self::Flirt => "flirt",
        }
    }

    pub const fn skill(self, shares_concern: bool) -> Skill {
        match self {
            Self::Reflect | Self::Listen => Skill::Insight,
            Self::Commiserate if shares_concern => Skill::Insight,
            Self::Commiserate | Self::Reframe => Skill::Deception,
            Self::Pray => Skill::Religion,
            Self::Reassure => Skill::Physiology,
            Self::LightenMood | Self::Flirt => Skill::Charm,
            Self::Rally => Skill::Command,
        }
    }

    pub const fn available_for(self, topic: SocialTopic) -> bool {
        match self {
            Self::Reflect | Self::Listen | Self::Commiserate => true,
            Self::Pray => !matches!(topic, SocialTopic::Filth),
            Self::Reassure => matches!(
                topic,
                SocialTopic::Injury | SocialTopic::Fatigue | SocialTopic::Hunger
            ),
            Self::LightenMood => !matches!(topic, SocialTopic::Faith),
            Self::Rally => matches!(
                topic,
                SocialTopic::Defeat | SocialTopic::Fatigue | SocialTopic::Faith
            ),
            Self::Reframe => matches!(
                topic,
                SocialTopic::Defeat | SocialTopic::Injury | SocialTopic::Faith
            ),
            Self::Flirt => matches!(topic, SocialTopic::Defeat | SocialTopic::Injury),
        }
    }

    pub const fn description(self, topic: SocialTopic, shares_concern: bool) -> &'static str {
        match (self, topic, shares_concern) {
            (Self::Reflect, _, _) => "Reflect on why this affects you",
            (Self::Listen, SocialTopic::Defeat, _) => "Ask how they feel about the defeat",
            (Self::Listen, SocialTopic::Injury, _) => "Ask how the injury is affecting them",
            (Self::Listen, SocialTopic::Fatigue, _) => "Ask how exhaustion is wearing on them",
            (Self::Listen, SocialTopic::Hunger, _) => "Ask how hunger is affecting them",
            (Self::Listen, SocialTopic::Faith, _) => "Ask what is troubling their conscience",
            (Self::Listen, SocialTopic::Filth, _) => "Ask why the grime is bothering them",
            (Self::Commiserate, SocialTopic::Defeat, true) => "Commiserate about the defeat",
            (Self::Commiserate, SocialTopic::Injury, true) => "Commiserate about being injured",
            (Self::Commiserate, SocialTopic::Fatigue, true) => "Commiserate about the exhaustion",
            (Self::Commiserate, SocialTopic::Hunger, true) => "Commiserate about going hungry",
            (Self::Commiserate, SocialTopic::Faith, true) => "Commiserate about the moral setback",
            (Self::Commiserate, SocialTopic::Filth, true) => "Commiserate about being filthy",
            (Self::Commiserate, SocialTopic::Defeat, false) => "Feign sympathy about the defeat",
            (Self::Commiserate, SocialTopic::Injury, false) => "Feign sympathy about the injury",
            (Self::Commiserate, SocialTopic::Fatigue, false) => {
                "Feign sympathy about the exhaustion"
            }
            (Self::Commiserate, SocialTopic::Hunger, false) => "Feign sympathy about going hungry",
            (Self::Commiserate, SocialTopic::Faith, false) => {
                "Feign sympathy about the moral setback"
            }
            (Self::Commiserate, SocialTopic::Filth, false) => "Feign sympathy about being filthy",
            (Self::Pray, _, _) => "Offer a prayer in their tradition",
            (Self::Reassure, SocialTopic::Injury, _) => {
                "Sit with them and speak calmly about what can be plainly observed"
            }
            (Self::Reassure, SocialTopic::Fatigue, _) => {
                "Attend to their weariness and acknowledge what they are feeling"
            }
            (Self::Reassure, SocialTopic::Hunger, _) => {
                "Stay with them and acknowledge the bodily distress of hunger"
            }
            (Self::LightenMood, SocialTopic::Defeat, _) => "Joke about bouncing back from defeat",
            (Self::LightenMood, SocialTopic::Injury, _) => "Joke to distract them from the pain",
            (Self::LightenMood, SocialTopic::Fatigue, _) => "Joke to help keep them awake",
            (Self::LightenMood, SocialTopic::Hunger, _) => "Joke about the empty provisions",
            (Self::LightenMood, SocialTopic::Filth, _) => "Joke about the mess they are in",
            (Self::Rally, SocialTopic::Defeat, _) => "Rally them after the defeat",
            (Self::Rally, SocialTopic::Fatigue, _) => "Urge them to keep going despite exhaustion",
            (Self::Rally, SocialTopic::Faith, _) => "Call on them to stand by their convictions",
            (Self::Reframe, SocialTopic::Defeat, _) => "Cast the defeat as a lesson",
            (Self::Reframe, SocialTopic::Injury, _) => {
                "Claim the injury is less serious than it looks"
            }
            (Self::Reframe, SocialTopic::Faith, _) => {
                "Offer a reassuring interpretation of the moral setback"
            }
            (Self::Flirt, SocialTopic::Defeat, _) => {
                "Tell them they remain impressive despite the defeat"
            }
            (Self::Flirt, SocialTopic::Injury, _) => "Tell them the scar makes them look striking",
            // Callers must check `available_for`; this keeps malformed clients
            // from eliciting invented topic-specific copy.
            _ => "This approach does not fit the concern",
        }
    }

    pub const fn risk(self) -> f32 {
        match self {
            Self::Reflect => 0.05,
            Self::Listen => 0.05,
            Self::Commiserate => 0.2,
            Self::Pray => 0.45,
            Self::Reassure => 0.12,
            Self::LightenMood => 0.45,
            Self::Rally => 0.55,
            Self::Reframe => 0.65,
            Self::Flirt => 0.85,
        }
    }
}

/// The supplied key is outside the canonical SocialActionKind vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseSocialActionKindError;

impl fmt::Display for ParseSocialActionKindError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Unknown social action")
    }
}

impl std::error::Error for ParseSocialActionKindError {}

impl FromStr for SocialActionKind {
    type Err = ParseSocialActionKindError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(value))
            .map_err(|_| ParseSocialActionKindError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_keys_select_the_executing_skill_and_serialize_consistently() {
        for (key, action, unshared, shared) in [
            (
                "reflect",
                SocialActionKind::Reflect,
                Skill::Insight,
                Skill::Insight,
            ),
            (
                "listen",
                SocialActionKind::Listen,
                Skill::Insight,
                Skill::Insight,
            ),
            (
                "commiserate",
                SocialActionKind::Commiserate,
                Skill::Deception,
                Skill::Insight,
            ),
            (
                "pray",
                SocialActionKind::Pray,
                Skill::Religion,
                Skill::Religion,
            ),
            (
                "reassure",
                SocialActionKind::Reassure,
                Skill::Physiology,
                Skill::Physiology,
            ),
            (
                "lighten_mood",
                SocialActionKind::LightenMood,
                Skill::Charm,
                Skill::Charm,
            ),
            (
                "command",
                SocialActionKind::Rally,
                Skill::Command,
                Skill::Command,
            ),
            (
                "deception",
                SocialActionKind::Reframe,
                Skill::Deception,
                Skill::Deception,
            ),
            ("flirt", SocialActionKind::Flirt, Skill::Charm, Skill::Charm),
        ] {
            assert_eq!(key.parse::<SocialActionKind>(), Ok(action));
            assert_eq!(action.reducer_value(), key);
            assert_eq!(action.skill(false), unshared);
            assert_eq!(action.skill(true), shared);
            assert_eq!(
                serde_json::to_string(&action).unwrap(),
                format!("\"{key}\"")
            );
        }
    }

    #[test]
    fn unknown_actions_and_ineligible_topics_fail_closed() {
        for key in ["", "rally", "reframe", "Command", " listen", "unknown"] {
            assert_eq!(
                key.parse::<SocialActionKind>(),
                Err(ParseSocialActionKindError)
            );
        }
        assert!(
            !"pray"
                .parse::<SocialActionKind>()
                .unwrap()
                .available_for(SocialTopic::Filth)
        );
        assert!(
            !"reassure"
                .parse::<SocialActionKind>()
                .unwrap()
                .available_for(SocialTopic::Faith)
        );
        assert!(
            "commiserate"
                .parse::<SocialActionKind>()
                .unwrap()
                .available_for(SocialTopic::Filth)
        );
    }
}
