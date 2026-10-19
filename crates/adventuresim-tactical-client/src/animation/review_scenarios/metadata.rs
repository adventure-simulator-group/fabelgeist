//! One open scenario-label admission boundary and retained metadata.

use super::*;
use std::cmp::Ordering;

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

#[derive(Debug, Clone, Copy)]
pub(crate) enum Repeatability {
    Repeating,
    Transition,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SolverUse {
    Procedural,
    Authored,
}

impl<'a> ScenarioMetadata<'a> {
    pub(crate) fn from_label(label: &'a str) -> Self {
        let identity = label.parse::<ScenarioId>().ok();
        let features = match identity {
            Some(identity) => identity.features(),
            None => unknown_features(label),
        };
        scenario_metadata(Cow::Borrowed(label), identity, features)
    }
    pub(crate) fn authored(identity: ScenarioId) -> ScenarioMetadata<'static> {
        scenario_metadata(
            Cow::Borrowed(identity.into()),
            Some(identity),
            identity.features(),
        )
    }
    pub(crate) fn label(&self) -> &str {
        &self.label
    }
    pub(crate) fn identity(&self) -> Option<ScenarioId> {
        self.identity
    }
    pub(crate) fn terrain(&self) -> ScenarioTerrain {
        self.terrain
    }
    pub(crate) fn body(&self) -> Option<BodyState> {
        self.execution.body
    }
    pub(crate) fn required_motion(&self) -> Option<&MotionId> {
        self.execution.motion.as_ref()
    }
    pub(crate) fn dive_direction(&self) -> Option<DiveDirection> {
        match self.execution.transition {
            Some((_, PostureTransitionKind::DiveToDowned { direction, .. })) => Some(direction),
            _ => None,
        }
    }
    pub(crate) fn transition(&self) -> Option<(BodyState, PostureTransitionKind)> {
        self.execution.transition
    }
    pub(crate) fn kind(&self) -> ScenarioKind {
        self.kind
    }
    pub(crate) fn repeatable(&self) -> bool {
        matches!(self.repeatable, Repeatability::Repeating)
    }
    pub(crate) fn procedural_solver(&self) -> bool {
        matches!(self.procedural_solver, SolverUse::Procedural)
    }
    pub(crate) fn has(&self, feature: ScenarioFeature) -> bool {
        self.features.contains(&feature)
    }
    pub(crate) fn is(&self, identity: ScenarioId) -> bool {
        self.identity == Some(identity)
    }
    pub(crate) fn into_owned(self) -> ScenarioMetadata<'static> {
        ScenarioMetadata {
            label: Cow::Owned(self.label.into_owned()),
            identity: self.identity,
            features: self.features,
            terrain: self.terrain,
            kind: self.kind,
            repeatable: self.repeatable,
            procedural_solver: self.procedural_solver,
            execution: self.execution,
        }
    }
}
impl PartialEq for ScenarioMetadata<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}
impl Eq for ScenarioMetadata<'_> {}
impl PartialOrd for ScenarioMetadata<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ScenarioMetadata<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.label.cmp(&other.label)
    }
}
impl Serialize for ScenarioMetadata<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.label())
    }
}
impl From<&str> for ScenarioMetadata<'static> {
    fn from(label: &str) -> Self {
        ScenarioMetadata::from_label(label).into_owned()
    }
}
impl From<String> for ScenarioMetadata<'static> {
    fn from(label: String) -> Self {
        ScenarioMetadata::from_label(&label).into_owned()
    }
}

fn unknown_features(label: &str) -> BTreeSet<ScenarioFeature> {
    let mut features = BTreeSet::new();
    admit_motion_features(label, &mut features);
    admit_terrain_features(label, &mut features);
    admit_transition_features(label, &mut features);
    features
}

fn admit_motion_features(label: &str, features: &mut BTreeSet<ScenarioFeature>) {
    let admitted = label.starts_with("quickstep-");
    if admitted {
        features.insert(ScenarioFeature::Quickstep);
    }
    let admitted = label.starts_with("downed-");
    if admitted {
        features.insert(ScenarioFeature::Downed);
    }
    let admitted = label.starts_with("dive-");
    if admitted {
        features.insert(ScenarioFeature::Dive);
    }
    let admitted = label.ends_with("-get-up");
    if admitted {
        features.insert(ScenarioFeature::GetUp);
    }
    let admitted = label.starts_with("prone-roll-");
    if admitted {
        features.insert(ScenarioFeature::ProneRoll);
    }
    let admitted = label == "jump-charge-anticipation";
    if admitted {
        features.insert(ScenarioFeature::JumpCharge);
    }
    let admitted = label.starts_with("attack-live-");
    if admitted {
        features.insert(ScenarioFeature::AttackLive);
    }
    let admitted = label.starts_with("dive-forward");
    if admitted {
        features.insert(ScenarioFeature::DiveForward);
    }
    let admitted = label.starts_with("dive-backward");
    if admitted {
        features.insert(ScenarioFeature::DiveBackward);
    }
    let admitted = label.starts_with("dive-left");
    if admitted {
        features.insert(ScenarioFeature::DiveLeft);
    }
    let admitted = label.starts_with("dive-right");
    if admitted {
        features.insert(ScenarioFeature::DiveRight);
    }
    let admitted = label.starts_with("attack-live-stationary");
    if admitted {
        features.insert(ScenarioFeature::StationaryAttack);
    }
    let admitted = label.contains("high-speed");
    if admitted {
        features.insert(ScenarioFeature::HighSpeed);
    }
    let admitted = label.contains("yaw-only");
    if admitted {
        features.insert(ScenarioFeature::YawOnly);
    }
}

fn admit_terrain_features(label: &str, features: &mut BTreeSet<ScenarioFeature>) {
    let admitted = label.starts_with("terrain-");
    if admitted {
        features.insert(ScenarioFeature::TerrainPrefix);
    }
    let admitted = label.contains("terrain");
    if admitted {
        features.insert(ScenarioFeature::TerrainNamed);
    }
    let admitted = label.starts_with("flat-grid-");
    if admitted {
        features.insert(ScenarioFeature::FlatGrid);
    }
    let admitted = label.starts_with("raised-guard");
    if admitted {
        features.insert(ScenarioFeature::RaisedGuard);
    }
    let admitted = label.starts_with("raised-guard-");
    if admitted {
        features.insert(ScenarioFeature::RaisedGuardHyphen);
    }
    let admitted = label.starts_with("raised-guard-tap-stop-");
    if admitted {
        features.insert(ScenarioFeature::RaisedTapStop);
    }
    let admitted = label.contains("run");
    if admitted {
        features.insert(ScenarioFeature::Run);
    }
    let admitted = label.starts_with("terrain-steady-run");
    if admitted {
        features.insert(ScenarioFeature::TerrainSteadyRun);
    }
    let admitted = label.starts_with("raised-guard-tap-stop");
    if admitted {
        features.insert(ScenarioFeature::RaisedTapStopAny);
    }
}

fn admit_transition_features(label: &str, features: &mut BTreeSet<ScenarioFeature>) {
    let admitted = label.contains("stop");
    if admitted {
        features.insert(ScenarioFeature::Stop);
    }
    let admitted = label.contains("turn");
    if admitted {
        features.insert(ScenarioFeature::Turn);
    }
    let admitted = label.contains("toggle");
    if admitted {
        features.insert(ScenarioFeature::Toggle);
    }
    let admitted = label.contains("restart");
    if admitted {
        features.insert(ScenarioFeature::Restart);
    }
    let admitted = label.contains("chatter");
    if admitted {
        features.insert(ScenarioFeature::Chatter);
    }
    let admitted = label.contains("release");
    if admitted {
        features.insert(ScenarioFeature::Release);
    }
    let admitted = label.contains("tap-stop");
    if admitted {
        features.insert(ScenarioFeature::TapStop);
    }
    let admitted = label.contains("reversal");
    if admitted {
        features.insert(ScenarioFeature::Reversal);
    }
    let admitted = label.contains("accelerate");
    if admitted {
        features.insert(ScenarioFeature::Accelerate);
    }
    let admitted = label.contains("transition");
    if admitted {
        features.insert(ScenarioFeature::Transition);
    }
    let admitted = label.contains("enter-exit");
    if admitted {
        features.insert(ScenarioFeature::EnterExit);
    }
    let admitted = label.contains("hard-stop");
    if admitted {
        features.insert(ScenarioFeature::HardStop);
    }
    let admitted = label.contains("ramp");
    if admitted {
        features.insert(ScenarioFeature::Ramp);
    }
    let admitted = label.ends_with("-impact");
    if admitted {
        features.insert(ScenarioFeature::Impact);
    }
    let admitted = label.ends_with("-aimed-impact");
    if admitted {
        features.insert(ScenarioFeature::AimedImpact);
    }
}
