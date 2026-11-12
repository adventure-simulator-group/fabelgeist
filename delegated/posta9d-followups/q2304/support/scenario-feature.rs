//! Case-sensitive admission grammar for open animation-review labels.
//!
//! Authored identities use their explicit catalogue policy. Other labels remain
//! valid and acquire each matching feature in the established admission order.

use super::ScenarioId;
use std::collections::BTreeSet;

const OPEN_LABEL_RULES: &[ScenarioFeatureRule] = &[
    ScenarioFeatureRule {
        feature: ScenarioFeature::Quickstep,
        pattern: ScenarioLabelPattern::Prefix("quickstep-"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Downed,
        pattern: ScenarioLabelPattern::Prefix("downed-"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Dive,
        pattern: ScenarioLabelPattern::Prefix("dive-"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::GetUp,
        pattern: ScenarioLabelPattern::Suffix("-get-up"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::ProneRoll,
        pattern: ScenarioLabelPattern::Prefix("prone-roll-"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::JumpCharge,
        pattern: ScenarioLabelPattern::Exact(ScenarioId::JumpChargeAnticipation),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::AttackLive,
        pattern: ScenarioLabelPattern::Prefix("attack-live-"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::DiveForward,
        pattern: ScenarioLabelPattern::Prefix("dive-forward"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::DiveBackward,
        pattern: ScenarioLabelPattern::Prefix("dive-backward"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::DiveLeft,
        pattern: ScenarioLabelPattern::Prefix("dive-left"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::DiveRight,
        pattern: ScenarioLabelPattern::Prefix("dive-right"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::StationaryAttack,
        pattern: ScenarioLabelPattern::Prefix("attack-live-stationary"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::HighSpeed,
        pattern: ScenarioLabelPattern::Substring("high-speed"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::YawOnly,
        pattern: ScenarioLabelPattern::Substring("yaw-only"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::TerrainPrefix,
        pattern: ScenarioLabelPattern::Prefix("terrain-"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::TerrainNamed,
        pattern: ScenarioLabelPattern::Substring("terrain"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::FlatGrid,
        pattern: ScenarioLabelPattern::Prefix("flat-grid-"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::RaisedGuard,
        pattern: ScenarioLabelPattern::Prefix("raised-guard"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::RaisedGuardHyphen,
        pattern: ScenarioLabelPattern::Prefix("raised-guard-"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::RaisedTapStop,
        pattern: ScenarioLabelPattern::Prefix("raised-guard-tap-stop-"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Run,
        pattern: ScenarioLabelPattern::Substring("run"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::TerrainSteadyRun,
        pattern: ScenarioLabelPattern::Prefix("terrain-steady-run"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::RaisedTapStopAny,
        pattern: ScenarioLabelPattern::Prefix("raised-guard-tap-stop"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Stop,
        pattern: ScenarioLabelPattern::Substring("stop"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Turn,
        pattern: ScenarioLabelPattern::Substring("turn"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Toggle,
        pattern: ScenarioLabelPattern::Substring("toggle"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Restart,
        pattern: ScenarioLabelPattern::Substring("restart"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Chatter,
        pattern: ScenarioLabelPattern::Substring("chatter"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Release,
        pattern: ScenarioLabelPattern::Substring("release"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::TapStop,
        pattern: ScenarioLabelPattern::Substring("tap-stop"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Reversal,
        pattern: ScenarioLabelPattern::Substring("reversal"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Accelerate,
        pattern: ScenarioLabelPattern::Substring("accelerate"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Transition,
        pattern: ScenarioLabelPattern::Substring("transition"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::EnterExit,
        pattern: ScenarioLabelPattern::Substring("enter-exit"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::HardStop,
        pattern: ScenarioLabelPattern::Substring("hard-stop"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Ramp,
        pattern: ScenarioLabelPattern::Substring("ramp"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::Impact,
        pattern: ScenarioLabelPattern::Suffix("-impact"),
    },
    ScenarioFeatureRule {
        feature: ScenarioFeature::AimedImpact,
        pattern: ScenarioLabelPattern::Suffix("-aimed-impact"),
    },
];

struct ScenarioFeatureRule {
    feature: ScenarioFeature,
    pattern: ScenarioLabelPattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ScenarioFeature {
    Quickstep,
    Downed,
    Dive,
    GetUp,
    ProneRoll,
    JumpCharge,
    TerrainPrefix,
    TerrainNamed,
    FlatGrid,
    AttackLive,
    RaisedGuard,
    RaisedGuardHyphen,
    RaisedTapStop,
    DiveForward,
    DiveBackward,
    DiveLeft,
    DiveRight,
    Run,
    Stop,
    Turn,
    Toggle,
    Restart,
    Chatter,
    TerrainSteadyRun,
    Release,
    TapStop,
    Reversal,
    Accelerate,
    Transition,
    EnterExit,
    HardStop,
    Ramp,
    Impact,
    AimedImpact,
    RaisedTapStopAny,
    StationaryAttack,
    HighSpeed,
    YawOnly,
}

#[derive(Clone, Copy)]
enum ScenarioLabelPattern {
    Prefix(&'static str),
    Substring(&'static str),
    Suffix(&'static str),
    Exact(ScenarioId),
}

impl ScenarioFeature {
    pub(super) fn from_open_label(label: &str) -> BTreeSet<Self> {
        OPEN_LABEL_RULES
            .iter()
            .filter(|rule| rule.pattern.matches(label))
            .map(|rule| rule.feature)
            .collect()
    }
}

impl ScenarioLabelPattern {
    fn matches(self, label: &str) -> bool {
        match self {
            Self::Prefix(prefix) => label.starts_with(prefix),
            Self::Substring(fragment) => label.contains(fragment),
            Self::Suffix(suffix) => label.ends_with(suffix),
            Self::Exact(identity) => label == <&'static str>::from(identity),
        }
    }
}
