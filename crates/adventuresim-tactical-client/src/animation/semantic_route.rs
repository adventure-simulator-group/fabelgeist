//! Direct deterministic semantic routing for pose-buffer playback.
//!
//! Authoritative presentation state is converted directly into the semantic
//! pose samples consumed by the pose buffer.

use adventuresim_tactical_core::animation::{
    AnimationPackId, AnimationValueError, AnimationValueResult, CyclePhase,
};
use std::collections::BTreeMap;

use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

use super::{AnimatedActors, AnimationRuntime, PresentedSkeleton, catalog};

/// Read-only coordinates presented to the semantic pose router. Every field
/// comes from client presentation state or its pure semantic evaluation.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub(crate) struct SemanticRouteInputs {
    pub speed: f32,
    pub direction: Vec2,
    pub gait_phase: CyclePhase,
    pub action: SkeletonAction,
    pub airborne: bool,
    pub target_height: f32,
    pub lead: LeadFoot,
    pub support: LeadFoot,
    pub contact_sequence: ContactSequence,
    pub pack: AnimationPackId,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct SemanticRouteTrace {
    pub inputs: SemanticRouteInputs,
    pub requested_path: SemanticRoutePath,
    pub path: SemanticRoutePath,
    pub evaluation: AnimationEvaluation,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct SemanticRouteTelemetry {
    counts: BTreeMap<SemanticRoutePath, u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SemanticRoutePath {
    GeneralPose,
    OrdinaryLocomotion,
    RaisedGuardAttack,
}

impl SemanticRouteInputs {
    pub(crate) fn from_presented(
        skeleton: &PresentedSkeleton,
        evaluation: &AnimationEvaluation,
    ) -> Self {
        Self {
            speed: evaluation.movement_speed,
            direction: skeleton.animation_local_velocity().xz().normalize_or_zero(),
            gait_phase: evaluation.gait_phase,
            action: skeleton.action_kind(),
            airborne: !skeleton.is_grounded(),
            target_height: evaluation.attack_target_height,
            lead: skeleton.lead_foot,
            support: skeleton.contact_foot,
            contact_sequence: skeleton.contact_sequence,
            pack: skeleton.animation_pack.clone(),
        }
    }
}

impl SemanticRoutePath {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::GeneralPose => "general_pose",
            Self::OrdinaryLocomotion => "ordinary_locomotion",
            Self::RaisedGuardAttack => "raised_guard_attack",
        }
    }
}

pub(super) fn route_semantic_pose(
    skeleton: &PresentedSkeleton,
) -> AnimationValueResult<SemanticRouteTrace> {
    let evaluation = AnimationEvaluation::from_skeleton(skeleton)?;
    let inputs = SemanticRouteInputs::from_presented(skeleton, &evaluation);
    let path = requested_path(skeleton);
    Ok(SemanticRouteTrace {
        inputs,
        requested_path: path,
        path,
        evaluation,
    })
}

pub(super) fn evaluate_semantic_route_paths(
    mut commands: Commands,
    mut telemetry: ResMut<SemanticRouteTelemetry>,
    runtime: Res<AnimationRuntime>,
    players: Query<(Entity, &PresentedSkeleton, Option<&InventoryItems>), AnimatedActors>,
    items: Query<&ItemProperties, With<WeaponItem>>,
    equip_slots: Query<&EquipSlot>,
) {
    for (entity, skeleton, inventory) in &players {
        let mut resolved = skeleton.clone();
        resolved.state.animation_pack =
            super::equipped_animation_pack(inventory, &items, &equip_slots).to_owned();
        resolved.state.attack_animations = runtime
            .library
            .attack_animations(&resolved.state.animation_pack);
        let trace = match route_semantic_pose(&resolved) {
            Ok(trace) => trace,
            Err(error) => {
                warn!(%error, ?entity, "Rejected semantic animation evaluation");
                continue;
            }
        };
        *telemetry.counts.entry(trace.path).or_default() += 1;
        commands.entity(entity).insert(trace);
    }
}

/// IK ownership follows the direct semantic locomotion samples.
/// The curve shapes are animation metadata: walk keeps one supported foot,
/// run has genuine intervals with neither foot loaded, and idle loads both.
pub(super) fn semantic_foot_ik_weights(
    evaluation: &AnimationEvaluation,
) -> AnimationValueResult<Vec2> {
    let samples = if !evaluation.lower_body.is_empty() {
        &evaluation.lower_body
    } else if evaluation.action.is_empty() {
        &evaluation.base
    } else {
        return Ok(Vec2::ZERO);
    };
    let mut result = Vec2::ZERO;
    let mut total = BlendWeight::ZERO;
    for sample in samples {
        let mut weights = match (sample.pose, sample.sampling) {
            (SemanticPose::IdleRelaxed, _) => Vec2::ONE,
            (SemanticPose::CombatStance, _) => Vec2::ONE,
            (
                SemanticPose::StrafeCycle | SemanticPose::SkipCycle,
                PoseSampling::Cycle { phase },
            ) => combat_cycle_ik_weights(phase),
            (
                SemanticPose::QuickstepForwardTakeoff
                | SemanticPose::QuickstepRightTakeoff
                | SemanticPose::QuickstepLeftTakeoff
                | SemanticPose::QuickstepBackTakeoff,
                PoseSampling::Timeline { .. },
            ) => Vec2::ZERO,
            (SemanticPose::WalkContact, PoseSampling::Cycle { phase }) => {
                let (left, right) = gait_support_weights(walk_locomotion_profile(), phase);
                Vec2::new(left, right)
            }
            (SemanticPose::RunContact, PoseSampling::Cycle { phase }) => {
                // Simulation support spans the complete stance interval. IK
                // is animation metadata and should only become strong near
                // the authored contact itself, after the foot has descended.
                let contact_profile = LocomotionProfile {
                    support_phase_radius: 0.11,
                    ..run_locomotion_profile()
                };
                let (left, right) = gait_support_weights(contact_profile, phase);
                Vec2::new(left, right)
            }
            _ => Vec2::ZERO,
        };
        if sample.mirror_lower_body == ClipMirror::Mirrored {
            weights = Vec2::new(weights.y, weights.x);
        }
        let weight = sample.weight;
        let next = result + weights * weight.factor();
        if !next.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: adventuresim_tactical_core::animation::AnimationQuantity::BlendWeight,
            });
        }
        total = total.checked_add(weight)?;
        result = next;
    }
    let normalized = if total.is_absent() {
        Vec2::ZERO
    } else {
        result / total.factor()
    };
    if !normalized.is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: adventuresim_tactical_core::animation::AnimationQuantity::BlendWeight,
        });
    }
    Ok(normalized.clamp(Vec2::ZERO, Vec2::ONE))
}

/// Select the single authored combat contact. Frames 0/12 are swing midpoints
/// and frames 6/18 switch contact. The pose buffer interpolates continuously
/// into each terrain-conformed target; support identity itself must not overlap
/// because an outgoing foot is free as soon as the incoming foot contacts.
pub(super) fn combat_cycle_ik_weights(phase: CyclePhase) -> Vec2 {
    let phase = phase.wrapped().cycles();
    if (catalog::COMBAT_FIRST_CONTACT_PHASE..catalog::COMBAT_SECOND_CONTACT_PHASE).contains(&phase)
    {
        Vec2::X
    } else {
        Vec2::Y
    }
}

fn requested_path(skeleton: &PresentedSkeleton) -> SemanticRoutePath {
    if skeleton.action_kind() == SkeletonAction::Attack
        || skeleton.weapon_guard() == WeaponGuardState::Raised
    {
        SemanticRoutePath::RaisedGuardAttack
    } else if skeleton.is_grounded() && skeleton.posture() == Posture::Upright {
        SemanticRoutePath::OrdinaryLocomotion
    } else {
        SemanticRoutePath::GeneralPose
    }
}
