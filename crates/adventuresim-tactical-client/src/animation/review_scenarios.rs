//! Admitted animation-review policy shared by capture, metrics and jitter.

use super::{MotionId, TerrainIkMode};
use adventuresim_tactical_core::prelude::*;
pub(crate) use authored::ScenarioId;
pub(crate) use metadata::{Repeatability, ScenarioFeature, SolverUse};
use serde::Serialize;
use std::{borrow::Cow, collections::BTreeSet};

mod authored;
mod authored_policy;
mod metadata;

#[derive(Debug, Clone)]
pub(crate) struct ScenarioMetadata<'a> {
    label: Cow<'a, str>,
    identity: Option<ScenarioId>,
    features: BTreeSet<ScenarioFeature>,
    terrain: ScenarioTerrain,
    kind: ScenarioKind,
    repeatable: Repeatability,
    procedural_solver: SolverUse,
    execution: ScenarioExecution,
}

#[derive(Debug, Clone, Default)]
struct ScenarioExecution {
    body: Option<BodyState>,
    motion: Option<MotionId>,
    transition: Option<(BodyState, PostureTransitionKind)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScenarioKind {
    Ordinary,
    RaisedGuard,
    Terrain,
    Transition,
    Landing,
    Attack,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ScenarioTerrain {
    Absent,
    Authored,
    Named,
    RaisedStop { startup: TerrainIkMode },
    Toggle,
}

impl ScenarioTerrain {
    pub(crate) fn admit(name: &ScenarioMetadata<'_>, kind: ScenarioKind) -> Self {
        let toggle = name.is(ScenarioId::TerrainToggleMidStride);
        let raised_stop = name.has(ScenarioFeature::RaisedTapStop);
        let named = name.has(ScenarioFeature::TerrainNamed);
        if toggle {
            Self::Toggle
        } else if raised_stop {
            Self::RaisedStop {
                startup: if named {
                    TerrainIkMode::Enabled
                } else {
                    TerrainIkMode::Disabled
                },
            }
        } else if kind == ScenarioKind::Terrain {
            Self::Authored
        } else if named {
            Self::Named
        } else {
            Self::Absent
        }
    }

    pub(crate) fn startup(self) -> TerrainIkMode {
        match self {
            Self::Absent => TerrainIkMode::Disabled,
            Self::RaisedStop { startup } => startup,
            Self::Authored | Self::Named | Self::Toggle => TerrainIkMode::Enabled,
        }
    }
}

pub(crate) fn uses_authored_combat_locomotion(name: &ScenarioMetadata<'_>) -> bool {
    name.has(ScenarioFeature::RaisedGuard)
        && !is_guard_stop_transition(name)
        && !matches!(
            name.identity,
            Some(ScenarioId::RaisedGuardStationaryTurn) | Some(ScenarioId::RaisedGuardTransition)
        )
}

pub(crate) fn is_guard_stop_transition(name: &ScenarioMetadata<'_>) -> bool {
    name.has(ScenarioFeature::RaisedGuard)
        && (name.has(ScenarioFeature::Release) || name.has(ScenarioFeature::TapStop))
}

pub(crate) fn is_quickstep_scenario(name: &ScenarioMetadata<'_>) -> bool {
    name.has(ScenarioFeature::Quickstep)
}

fn scenario_metadata(
    label: Cow<'_, str>,
    identity: Option<ScenarioId>,
    features: BTreeSet<ScenarioFeature>,
) -> ScenarioMetadata<'_> {
    let name = ScenarioMetadata {
        label,
        identity,
        features,
        terrain: ScenarioTerrain::Absent,
        kind: ScenarioKind::Ordinary,
        repeatable: Repeatability::Repeating,
        procedural_solver: SolverUse::Authored,
        execution: ScenarioExecution::default(),
    };
    let classified = |kind, repeatable, procedural_solver| ScenarioMetadata {
        label: name.label.clone(),
        identity: name.identity,
        features: name.features.clone(),
        terrain: ScenarioTerrain::admit(&name, kind),
        kind,
        execution: ScenarioExecution {
            body: downed_body_for_scenario(&name),
            motion: required_motion_for_scenario(&name),
            transition: transition_for_scenario(&name),
        },
        repeatable: if repeatable {
            Repeatability::Repeating
        } else {
            Repeatability::Transition
        },
        procedural_solver: if procedural_solver {
            SolverUse::Procedural
        } else {
            SolverUse::Authored
        },
    };
    if is_quickstep_scenario(&name)
        || name.has(ScenarioFeature::Downed)
        || name.has(ScenarioFeature::Dive)
        || name.has(ScenarioFeature::GetUp)
        || name.has(ScenarioFeature::ProneRoll)
        || name.is(ScenarioId::JumpChargeAnticipation)
    {
        classified(ScenarioKind::Transition, false, false)
    } else if name.is(ScenarioId::TerrainToggleMidStride) {
        classified(
            ScenarioKind::Terrain,
            false, // contract is bounded transition continuity, not steady plants.
            false,
        )
    } else if matches!(
        name.identity,
        Some(ScenarioId::FlatGridWalkNoIk) | Some(ScenarioId::FlatGridSprintNoIk)
    ) {
        classified(ScenarioKind::Ordinary, true, false)
    } else if name.is(ScenarioId::CrossSlopeWalk)
        || name.has(ScenarioFeature::TerrainPrefix)
        || name.has(ScenarioFeature::FlatGrid)
    {
        classified(
            ScenarioKind::Terrain,
            !name.has(ScenarioFeature::Stop)
                && !name.has(ScenarioFeature::Turn)
                && !name.has(ScenarioFeature::Toggle)
                && !name.has(ScenarioFeature::Restart)
                && !name.has(ScenarioFeature::Chatter)
                && !name.has(ScenarioFeature::TerrainSteadyRun),
            true,
        )
    } else if name.has(ScenarioFeature::AttackLive) {
        classified(ScenarioKind::Attack, false, true)
    } else if name.has(ScenarioFeature::RaisedGuard) {
        classified(
            ScenarioKind::RaisedGuard,
            !name.has(ScenarioFeature::Release)
                && !name.has(ScenarioFeature::Reversal)
                && !name.has(ScenarioFeature::Accelerate)
                && !name.is(ScenarioId::RaisedGuardStationaryTurn)
                && !name.is(ScenarioId::RaisedGuardTransition), // procedural plant/step solver.
            !uses_authored_combat_locomotion(&name),
        )
    } else if name.is(ScenarioId::AirborneLanding) {
        classified(ScenarioKind::Landing, false, false)
    } else if name.has(ScenarioFeature::Transition)
        || name.has(ScenarioFeature::EnterExit)
        || name.has(ScenarioFeature::HardStop)
        || name.has(ScenarioFeature::Ramp)
        || name.has(ScenarioFeature::Turn)
    {
        classified(ScenarioKind::Transition, false, false)
    } else {
        classified(ScenarioKind::Ordinary, true, false)
    }
}

fn downed_body_for_scenario(scenario: &ScenarioMetadata<'_>) -> Option<BodyState> {
    match scenario.identity {
        Some(ScenarioId::DownedProneCrawl) | Some(ScenarioId::DownedProneLookAt) => {
            Some(BodyState::Prone)
        }
        Some(ScenarioId::DownedSupineScamper) => Some(BodyState::Supine),
        Some(ScenarioId::FullRagdoll) => Some(BodyState::Ragdolled),
        _ => None,
    }
}

fn required_motion_for_scenario(
    scenario: &ScenarioMetadata<'_>,
) -> Option<crate::animation::MotionId> {
    if scenario.has(ScenarioFeature::Dive) {
        return Some(crate::animation::MotionId::DIVE);
    }
    match scenario.identity {
        Some(ScenarioId::DownedProneCrawl) => Some(crate::animation::MotionId::PRONE_CRAWL),
        Some(ScenarioId::DownedSupineScamper) => Some(crate::animation::MotionId::SUPINE_SCAMPER),
        Some(ScenarioId::DownedProneLookAt) => Some(crate::animation::MotionId::PRONE_IDLE),
        Some(ScenarioId::ProneGetUp) => Some(crate::animation::MotionId::PRONE_TRANSITION),
        Some(ScenarioId::SupineGetUp) => Some(crate::animation::MotionId::SUPINE_TRANSITION),
        Some(ScenarioId::ProneRollLeft) => Some(crate::animation::MotionId::PRONE_SUPINE_ROLL_LEFT),
        Some(ScenarioId::ProneRollRight) => {
            Some(crate::animation::MotionId::PRONE_SUPINE_ROLL_RIGHT)
        }
        _ => None,
    }
}

fn transition_for_scenario(
    scenario: &ScenarioMetadata<'_>,
) -> Option<(BodyState, PostureTransitionKind)> {
    let upright = BodyState::Grounded(GroundedPosture::Upright);
    let dive_direction = if scenario.has(ScenarioFeature::DiveForward) {
        Some(DiveDirection::Forward)
    } else if scenario.has(ScenarioFeature::DiveBackward) {
        Some(DiveDirection::Backward)
    } else if scenario.has(ScenarioFeature::DiveLeft) {
        Some(DiveDirection::Left)
    } else if scenario.has(ScenarioFeature::DiveRight) {
        Some(DiveDirection::Right)
    } else {
        None
    };
    if let Some(direction) = dive_direction {
        return Some((
            upright,
            PostureTransitionKind::DiveToDowned {
                direction,
                trajectory: DiveTrajectory::Airborne,
            },
        ));
    }
    match scenario.identity {
        Some(ScenarioId::ProneGetUp) => {
            Some((BodyState::Prone, PostureTransitionKind::ProneToUpright))
        }
        Some(ScenarioId::SupineGetUp) => {
            Some((BodyState::Supine, PostureTransitionKind::SupineToUpright))
        }
        Some(ScenarioId::ProneRollLeft) => Some((
            BodyState::Prone,
            PostureTransitionKind::ProneToSupine {
                direction: RollDirection::Left,
            },
        )),
        Some(ScenarioId::ProneRollRight) => Some((
            BodyState::Prone,
            PostureTransitionKind::ProneToSupine {
                direction: RollDirection::Right,
            },
        )),
        _ => None,
    }
}
