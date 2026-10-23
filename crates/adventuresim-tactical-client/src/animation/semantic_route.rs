//! Direct deterministic semantic routing for pose-buffer playback.
//!
//! Authoritative presentation state is converted directly into the semantic
//! pose samples consumed by the pose buffer.

use adventuresim_core::linear_speed::NonnegativeSpeed;
use adventuresim_tactical_core::animation::{
    AnimationPackId, AnimationValueResult, CyclePhase, UnitInterval,
};
use std::collections::BTreeMap;

use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

use super::{AnimatedActors, AnimationRuntime, PresentedSkeleton, catalog};

/// Read-only coordinates presented to the semantic pose router. Every field
/// comes from client presentation state or its pure semantic evaluation.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub(crate) struct SemanticRouteInputs {
    pub speed: NonnegativeSpeed,
    pub direction: Vec2,
    pub gait_phase: CyclePhase,
    pub action: SkeletonAction,
    #[serde(rename = "airborne", serialize_with = "serialize_airborne")]
    pub ground_contact: GroundContact,
    pub target_height: UnitInterval,
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
            ground_contact: GroundContact::from(&skeleton.state),
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
                // The chain synchronizes Commands before evaluating skeletons.
                commands.entity(entity).remove::<SemanticRouteTrace>();
                continue;
            }
        };
        *telemetry.counts.entry(trace.path).or_default() += 1;
        commands.entity(entity).insert(trace);
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

fn serialize_airborne<S: serde::Serializer>(
    contact: &GroundContact,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_bool(*contact == GroundContact::Airborne)
}
