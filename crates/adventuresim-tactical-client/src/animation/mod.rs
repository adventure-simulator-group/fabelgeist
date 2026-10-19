#![expect(
    unused_imports,
    reason = "the gameplay client, animation viewer, and reflection linker consume different parts of this shared facade"
)]

use adventuresim_tactical_core::animation::AttackCurve;
use adventuresim_tactical_core::animation::{AnimationElapsed, AnimationValueError};
use adventuresim_tactical_core::animation::{
    AnimationPackId, AnimationSampleSpan, AnimationSampleTick, BlendWeight, ClipMirror, CycleDelta,
    CyclePhase, SkeletonFamily,
};
use adventuresim_tactical_core::prelude::*;
#[cfg(not(target_family = "wasm"))]
use adventuresim_tactical_netcode::message::PlayerInputRequest;
use adventuresim_tactical_netcode::message::SuccessfulAttackResponse;
use authored_cadence::{
    AuthoredLocomotionStrides, AuthoredPhaseCurve, AuthoredStrideMeasurement, CalibrationMotion,
};
use bevy::{
    animation::AnimationTargetId,
    asset::{AssetId, LoadState},
    gltf::Gltf,
    prelude::*,
};
use bind_pose::{capture_authored_bind_transforms, restore_authored_bind_pose};
pub(crate) use capture_ids::{capture_animation_target_id, capture_entity_id};
pub use catalog::AnimationPackCatalog;
use catalog::*;
pub(crate) use catalog_id::MotionId;
use catalog_id::{AnimationAssetDirectory, AnimationAssetPath, AuthoredAnimationFrame, MotionKey};
use continuation::{ContinuationSpan, append_continuation_span};
use diagnostics::log_animation_diagnostics;
#[cfg(not(target_family = "wasm"))]
pub(crate) use diagnostics::{
    AnimationDiagnosticLog, DiagnosticInputStatus, RenderScheduleTelemetry,
};
use loading::*;
use playback_contributions::PlaybackContributions;
use playback_state::{
    BaseRigLoad, InertialState, OrdinaryLocomotion, PoseOwnership, PoseVisibility,
    TimelineDirection,
};
pub(crate) use presentation::{
    plugin::{AnimatedActors, StrategicModel},
    *,
};
pub(crate) use procedural::{
    ArmIkState, BodyAngle, BoneRole, HandIkTarget, HandSide, HeldWeaponConstraint, HumanoidBone,
    HumanoidIkTargets, HumanoidRig, LegIkDiagnostics, LegIkState, LocomotionBodyResponseState,
    LocomotionHeightState, MhrBone, ProceduralAnimationClock, RaisedFootworkState,
    authored_bind_global, cache_humanoid_rigs, locomotion_support_weights,
    measured_ankle_sole_offset_metres, sole_contact_tolerance_metres,
};
use resolver::PoseSampleResolver;
use rig_identity::AnimationBodyRegion;
pub(crate) use rig_identity::AuthoredRigJoint;
use sample_values::{AnimationValueResult, ClipDuration, ClipTime, finite_sample_value};
#[cfg(test)]
use semantic_route::combat_cycle_ik_weights;
use semantic_route::semantic_foot_ik_weights;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use weighted::{WeightedClip, append_weighted_anchor, append_weighted_clip};

mod authored_cadence;
mod bind_pose;
pub(crate) mod bouncy_bones;
mod capture_ids;
pub(crate) mod catalog;
mod catalog_id;
mod clip_revision;
mod continuation;
#[cfg(not(target_family = "wasm"))]
mod diagnostic_record;
mod diagnostics;
pub(crate) mod fallback;
mod identity_morphs;
pub(crate) mod jitter;
mod loading;
mod playback_contributions;
mod playback_state;
pub(crate) mod pose_buffer;
mod presentation;
mod procedural;
mod ragdoll;
mod resolver;
pub(crate) mod review_scenarios;
mod rig_identity;
mod sample_values;
pub(crate) mod semantic_route;
pub(crate) mod skeletal_proportions;
mod weighted;

pub(crate) const WORKSPACE_ASSET_SOURCE: &str = "workspace";

const BIPED_BASE_GLB: AnimationAssetPath =
    AnimationAssetPath::from_static("animations/biped/unarmed/base.glb");

const BIPED_GRIP_HILT_GLB: AnimationAssetPath =
    AnimationAssetPath::from_static("animations/biped/grip_hilt.glb");

const BIPED_GRIP_POLEARM_GLB: AnimationAssetPath =
    AnimationAssetPath::from_static("animations/biped/grip_polearm.glb");

#[derive(Debug, Clone)]
struct LoadedClip {
    handle: Handle<AnimationClip>,
    duration_seconds: ClipDuration,
    layer: ClipLayer,
}

#[derive(Resource, Default)]
pub(crate) struct AnimationRuntime {
    base_rig: BaseRigLoad,
    requested_motions: BTreeMap<MotionKey, MotionLoad>,
    unavailable_motions: BTreeSet<MotionKey>,
    clips: BTreeMap<MotionKey, LoadedClip>,
    requested_grips: BTreeMap<WeaponGrip, Handle<Gltf>>,
    processed_grips: BTreeSet<WeaponGrip>,
    unavailable_grips: BTreeSet<WeaponGrip>,
    grips: BTreeMap<WeaponGrip, LoadedClip>,
    library: AnimationPackLibrary,
    canonical_targets: HashSet<AnimationTargetId>,
}

#[derive(Component, Debug)]
pub(super) struct AnimationPlayback {
    plan: PlaybackContributions,
    whole_body_mirror: BlendWeight,
    pub(super) foot_ik_weights: Vec2,
    weapon_guard: WeaponGuardState,
    ordinary_locomotion: OrdinaryLocomotion,
    cadence: PoseSamplingCadence,
}

#[derive(Debug, Clone)]
struct ExtrapolatedSpan {
    start: LoadedClip,
    start_time_seconds: ClipTime,
    end: LoadedClip,
    end_time_seconds: ClipTime,
    coordinate: f32,
    weight: BlendWeight,
    mirrored_weight: BlendWeight,
}

#[derive(Debug, Clone)]
struct PlaybackPose {
    plan: PlaybackContributions,
    whole_body_mirror: BlendWeight,
    foot_ik_weights: Vec2,
    cadence: PoseSamplingCadence,
}

#[derive(Component)]
pub struct FallbackAnimationRig(pub Entity);

#[derive(Component)]
pub(super) struct AnimationRigScene(pub Entity);

#[derive(Component)]
struct AnimationRigAttached;

#[derive(Component)]
struct RigAnimationTargetsBound;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct AuthoredBindTransform {
    pub(crate) owner: Entity,
    pub(super) local: Transform,
}

#[derive(Clone)]
struct ResolvedAnchor<'a> {
    clip: &'a LoadedClip,
    anchor: &'a PoseAnchor,
    pack_id: &'a AnimationPackId,
    semantic: SemanticPose,
    mirrored: ClipMirror,
    timeline_direction: TimelineDirection,
}

/// Runtime mode for terrain height, slope, and pelvis conformity. This is on
/// by default; debug builds expose F8 to compare against authored FK.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerrainIkMode {
    #[default]
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ClipLayer {
    Whole,
    Upper,
    Lower,
    /// Combat upper-body motion owns the pelvis rotation while the paired
    /// locomotion layer owns its translation.
    CombatUpper,
    CombatLower,
    /// Static equipment grip owns only the subtree below the right wrist.
    MainHand,
    /// Static equipment grip owns only the subtree below the left wrist.
    Offhand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WeaponGrip {
    Hilt,
    Polearm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PoseSamplingCadence {
    Buffered,
    Direct,
}

impl TerrainIkMode {
    pub(crate) fn is_enabled(self) -> bool {
        self == Self::Enabled
    }

    pub(crate) fn toggle(&mut self) {
        *self = match self {
            Self::Enabled => Self::Disabled,
            Self::Disabled => Self::Enabled,
        };
    }
}

impl LoadedClip {
    fn at_anchor(&self, frame: AuthoredAnimationFrame) -> Self {
        self.at_anchor_layer(frame, ClipLayer::Whole)
    }

    fn at_anchor_layer(&self, _frame: AuthoredAnimationFrame, layer: ClipLayer) -> Self {
        let mut clip = self.clone();
        clip.layer = layer;
        clip
    }
}

impl WeaponGrip {
    const ALL: [Self; 2] = [Self::Hilt, Self::Polearm];

    const fn path(self) -> AnimationAssetPath {
        match self {
            Self::Hilt => BIPED_GRIP_HILT_GLB,
            Self::Polearm => BIPED_GRIP_POLEARM_GLB,
        }
    }
}

impl AnimationRuntime {
    pub(crate) fn requested_base(&self) -> Option<&Handle<Gltf>> {
        self.base_rig.source()
    }

    pub(super) fn motion_is_processed(&self, motion: &MotionId) -> bool {
        self.clips.keys().any(|key| &key.motion == motion)
    }
}

fn animation_frames_per_second() -> f32 {
    runtime_animation_config().playback.frames_per_second
}

// Player transforms sit at the center of the 1.9 m server collider, while
// authored rigs use a floor-level origin. Keep visual feet on the collider's
// lower face so the first-person camera lands at the authored head.
fn player_visual_y_offset() -> adventuresim_building_generator::spatial_geometry::SignedLength {
    runtime_animation_config()
        .playback
        .player_visual_y_offset_metres
}

#[expect(
    clippy::type_complexity,
    reason = "the Bevy query couples each presented skeleton with its route, inventory, and playback state"
)]
fn evaluate_skeletons(
    mut commands: Commands,
    catalog: Res<AnimationPackCatalog>,
    runtime: Res<AnimationRuntime>,
    locomotion_strides: Res<AuthoredLocomotionStrides>,
    players: Query<
        (
            Entity,
            &PresentedSkeleton,
            &semantic_route::SemanticRouteTrace,
            Option<&InventoryItems>,
            Option<&mut AnimationPlayback>,
            Option<&pose_buffer::CharacterLocomotionStrides>,
        ),
        AnimatedActors,
    >,
    weapons: Query<(&WeaponItem, &ItemProperties)>,
    equip_slots: Query<&EquipSlot>,
) {
    'actor: for (entity, skeleton, route_trace, inventory, playback, character_strides) in players {
        // The preceding chained system directly routes authoritative
        // presentation state into deterministic semantic samples.
        let evaluation = route_trace.evaluation.clone();
        let samples = if evaluation.action.is_empty() {
            &evaluation.base
        } else {
            &evaluation.action
        };
        let sample_resolver = PoseSampleResolver {
            runtime: &runtime,
            catalog: &catalog,
            pack: &route_trace.inputs.pack,
            locomotion_strides: character_strides
                .map_or(&*locomotion_strides, |strides| &strides.measurements),
        };
        let mut target = match sample_resolver.resolve_body(skeleton, &evaluation) {
            Ok(target) => target,
            Err(error) => {
                error!(?entity, %error, "Rejected animation sample plan");
                continue;
            }
        };
        if let Some((weapon, properties)) = equipped_main_weapon(inventory, &weapons, &equip_slots)
            && let Some(clip) = runtime.grips.get(&weapon_grip(&weapon.skill_weights))
        {
            let uses_offhand =
                weapon_uses_offhand(&properties.id, offhand_is_empty(inventory, &equip_slots));
            let authored_pose_owns_hands = authored_pose_owns_hands(samples);
            for layer in weapon_grip_layers(authored_pose_owns_hands, uses_offhand)
                .into_iter()
                .flatten()
            {
                if let Err(error) = append_weighted_clip(
                    &mut target.clips,
                    &clip.at_anchor_layer(AuthoredAnimationFrame::ZERO, layer),
                    ClipMirror::Original,
                    ClipTime::ZERO,
                    BlendWeight::ONE,
                    None,
                ) {
                    error!(?entity, %error, "Rejected weapon grip contribution");
                    continue 'actor;
                }
            }
        }
        let target = match target.into_playback(&evaluation) {
            Ok(target) => target,
            Err(error) => {
                error!(?entity, %error, "Rejected animation IK contributions");
                continue;
            }
        };
        if let Some(mut playback) = playback {
            playback.apply_presented_pose(skeleton, target, evaluation.movement_speed);
        } else {
            commands
                .entity(entity)
                .insert(AnimationPlayback::from_presented_pose(
                    skeleton,
                    target,
                    evaluation.movement_speed,
                ));
        }
    }
}

fn combat_pelvis_is_component_blended(
    skeleton: &SkeletonState,
    evaluation: &AnimationEvaluation,
) -> bool {
    !evaluation.lower_body.is_empty()
        && skeleton.posture() == Posture::Upright
        && skeleton.weapon_guard() == WeaponGuardState::Raised
        && !skeleton.is_quickstep()
        && !skeleton.is_posture_transitioning()
}

fn weapon_grip(skill_weights: &[f32; 9]) -> WeaponGrip {
    if skill_weights[0] > 0.0 {
        WeaponGrip::Polearm
    } else {
        WeaponGrip::Hilt
    }
}

fn held_item(
    inventory: Option<&InventoryItems>,
    equip_slots: &Query<&EquipSlot>,
    slot: EquipSlot,
) -> Option<Entity> {
    inventory?.iter().find(|&entity| {
        equip_slots
            .get(entity)
            .is_ok_and(|equipped| *equipped == slot)
    })
}

fn equipped_main_weapon<'a>(
    inventory: Option<&InventoryItems>,
    weapons: &'a Query<(&WeaponItem, &ItemProperties)>,
    equip_slots: &Query<&EquipSlot>,
) -> Option<(&'a WeaponItem, &'a ItemProperties)> {
    held_item(inventory, equip_slots, EquipSlot::HoldingRight)
        .and_then(|entity| weapons.get(entity).ok())
}

fn offhand_is_empty(inventory: Option<&InventoryItems>, equip_slots: &Query<&EquipSlot>) -> bool {
    held_item(inventory, equip_slots, EquipSlot::HoldingLeft).is_none()
}

fn authored_pose_owns_hands(samples: &[PoseSample]) -> bool {
    samples
        .iter()
        .any(|sample| sample.pose.is_main_hand_attack())
}

fn weapon_uses_offhand(item_id: &str, offhand_is_empty: bool) -> bool {
    offhand_is_empty
        && item_catalog::weapon_handling(item_id) == Some(item_catalog::WeaponHandling::TwoHanded)
}

fn weapon_grip_layers(
    authored_pose_owns_hands: bool,
    uses_offhand: bool,
) -> [Option<ClipLayer>; 2] {
    match (authored_pose_owns_hands, uses_offhand) {
        (true, true) => [None, None],
        (false, true) => [Some(ClipLayer::MainHand), Some(ClipLayer::Offhand)],
        _ => [Some(ClipLayer::MainHand), None],
    }
}

fn equipped_animation_pack(
    inventory: Option<&InventoryItems>,
    items: &Query<&ItemProperties, With<WeaponItem>>,
    equip_slots: &Query<&EquipSlot>,
) -> AnimationPackId {
    let Some(properties) = held_item(inventory, equip_slots, EquipSlot::HoldingRight)
        .and_then(|entity| items.get(entity).ok())
    else {
        return AnimationPackId::HUMANOID_UNARMED;
    };
    animation_pack_for_weapon(
        &properties.id,
        weapon_uses_offhand(&properties.id, offhand_is_empty(inventory, equip_slots)),
    )
}

fn animation_pack_for_weapon(item_id: &str, uses_offhand: bool) -> AnimationPackId {
    if item_catalog::weapon_handling(item_id) == Some(item_catalog::WeaponHandling::TwoHanded)
        && !uses_offhand
    {
        return AnimationPackId::HUMANOID_UNARMED;
    }
    if let Some(pack) = item_catalog::weapon_animation_pack(item_id) {
        return AnimationPackId::from(pack);
    }
    match item_catalog::weapon_handling(item_id) {
        Some(item_catalog::WeaponHandling::TwoHanded) => AnimationPackId::HUMANOID_2H_CLOSE,
        _ => AnimationPackId::HUMANOID_UNARMED,
    }
}

fn ordinary_locomotion_candidate(skeleton: &SkeletonState) -> bool {
    !skeleton.is_posture_transitioning()
        && ((skeleton.is_grounded()
            && skeleton.action_kind() == SkeletonAction::None
            && skeleton.weapon_guard() == WeaponGuardState::Lowered)
            || skeleton.is_downed_turning())
}

fn apply_playback_pose(playback: &mut AnimationPlayback, pose: PlaybackPose) {
    playback.plan = pose.plan;
    playback.whole_body_mirror = pose.whole_body_mirror;
    playback.foot_ik_weights = pose.foot_ik_weights;
    playback.cadence = pose.cadence;
}

/// A server-confirmed hit kicks the struck bone's spring on the recipient.
fn on_successful_attack(
    event: On<SuccessfulAttackResponse>,
    mut kicks: MessageWriter<bouncy_bones::BoneKick>,
) {
    if event.impact_velocity_change.length_squared() > 0.0 {
        kicks.write(bouncy_bones::BoneKick {
            owner: event.impact_recipient,
            role: bouncy_bones::impact_bone_role(event.body_part),
            impulse: event.impact_velocity_change
                * runtime_animation_config().bouncy_bones.impact_impulse_scale,
        });
    }
}

fn resolve_anchor<'a>(
    runtime: &'a AnimationRuntime,
    catalog: &'a AnimationPackCatalog,
    requested_pack: &AnimationPackId,
    pose: SemanticPose,
) -> Option<ResolvedAnchor<'a>> {
    let ordinary = resolve_library_anchor(runtime, catalog, requested_pack, pose);
    if ordinary
        .as_ref()
        .is_some_and(|resolved| resolved.semantic == pose)
    {
        return ordinary;
    }

    // A directional quickstep may omit its opposite file. In that case its
    // takeoff resolves to the authored opposite contact and its contact to the
    // opposite takeoff, which samples that one clip backward. Exact authored
    // direction clips always win above.
    if let Some(reversed_pose) = reversed_quickstep_pose(pose)
        && let Some(reversed) =
            resolve_library_anchor(runtime, catalog, requested_pack, reversed_pose)
        && reversed.semantic == reversed_pose
    {
        return Some(ResolvedAnchor {
            timeline_direction: TimelineDirection::Reversed,
            ..reversed
        });
    }

    ordinary
}

fn resolve_library_anchor<'a>(
    runtime: &'a AnimationRuntime,
    catalog: &'a AnimationPackCatalog,
    requested_pack: &AnimationPackId,
    pose: SemanticPose,
) -> Option<ResolvedAnchor<'a>> {
    let ResolvedPose::Clip {
        pack_id,
        pose: resolved_pose,
        mirrored,
        ..
    } = runtime.library.resolve(requested_pack, pose)
    else {
        return None;
    };
    let pack = catalog.packs.get(pack_id)?;
    let anchor = pack.poses.get(&resolved_pose)?;
    pack.motions.get(&anchor.motion)?;
    let clip = runtime
        .clips
        .get(&MotionKey::new(pack_id, &anchor.motion))?;
    Some(ResolvedAnchor {
        clip,
        anchor,
        pack_id,
        semantic: resolved_pose,
        mirrored,
        timeline_direction: TimelineDirection::Forward,
    })
}

fn reversed_quickstep_pose(pose: SemanticPose) -> Option<SemanticPose> {
    use SemanticPose::*;
    Some(match pose {
        QuickstepForwardTakeoff => QuickstepBackTakeoff,
        QuickstepForwardContact => QuickstepBackContact,
        QuickstepBackTakeoff => QuickstepForwardTakeoff,
        QuickstepBackContact => QuickstepForwardContact,
        QuickstepRightTakeoff => QuickstepLeftTakeoff,
        QuickstepRightContact => QuickstepLeftContact,
        QuickstepLeftTakeoff => QuickstepRightTakeoff,
        QuickstepLeftContact => QuickstepRightContact,
        _ => return None,
    })
}

fn select_gait_endpoint_parity<'a>(
    runtime: &'a AnimationRuntime,
    resolved: ResolvedAnchor<'a>,
    mirrored: ClipMirror,
) -> Option<ResolvedAnchor<'a>> {
    if mirrored == ClipMirror::Original {
        return Some(resolved);
    }
    let motion = resolved.anchor.motion.mirrored();
    let clip = runtime
        .clips
        .get(&MotionKey::new(resolved.pack_id, &motion))?;
    Some(ResolvedAnchor { clip, ..resolved })
}

fn append_resolved_sample(
    weighted: &mut Vec<WeightedClip>,
    runtime: &AnimationRuntime,
    catalog: &AnimationPackCatalog,
    pack: &AnimationPackId,
    sample: PoseSample,
) -> AnimationValueResult<()> {
    let mut extrapolated_spans = Vec::new();
    let mut continuation_spans = Vec::new();
    let locomotion_strides = AuthoredLocomotionStrides::default();
    PoseSampleResolver {
        runtime,
        catalog,
        pack,
        locomotion_strides: &locomotion_strides,
    }
    .append_layer(
        weighted,
        &mut extrapolated_spans,
        &mut continuation_spans,
        sample,
        ClipLayer::Whole,
    )
}

fn update_rig_visibility(
    playbacks: Query<&AnimationPlayback>,
    mut fallbacks: Query<(&FallbackAnimationRig, &mut Visibility)>,
    mut authored: Query<(&AnimationRigScene, &mut Visibility), Without<FallbackAnimationRig>>,
) {
    let authored_owners = authored
        .iter_mut()
        .map(|(rig, _)| rig.0)
        .collect::<BTreeSet<_>>();
    for (owner, mut visibility) in &mut fallbacks {
        *visibility = if authored_owners.contains(&owner.0) {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    for (owner, mut visibility) in &mut authored {
        *visibility = if playbacks.get(owner.0).is_ok() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;

    const ABSENT_JUMP_MOTION_PREFIX: &str = "jump_";

    const LEFT_CONTACT_PHASE: f32 = 0.0;
    const LEFT_SUPPORT_SHOULDER_PHASE: f32 = 0.125;
    const LEFT_PASSING_PHASE: f32 = 0.25;
    const RIGHT_SUPPORT_SHOULDER_PHASE: f32 = 0.375;
    const RIGHT_CONTACT_PHASE: f32 = 0.5;
    const RIGHT_RELEASE_SHOULDER_PHASE: f32 = 0.625;
    const RIGHT_PASSING_PHASE: f32 = 0.75;
    const LEFT_RELEASE_SHOULDER_PHASE: f32 = 0.875;

    #[test]
    fn sparse_catalog_declares_airborne_and_three_attack_assets() {
        let catalog = AnimationPackCatalog::biped_root().unwrap();
        let root = &catalog.packs[&AnimationPackId::HUMANOID_UNARMED];
        assert!(root.motions.contains_key(&MotionId::AIRBORNE_CENTER));
        assert!(root.motions.contains_key(&MotionId::AIRBORNE_TRAVEL));
        assert!(root.motions.contains_key(&MotionId::SWING));
        assert!(root.motions.contains_key(&MotionId::THRUST));
        assert!(root.motions.contains_key(&MotionId::OFFHAND));
        assert!(
            !root
                .motions
                .keys()
                .any(|name| name.to_string().starts_with(ABSENT_JUMP_MOTION_PREFIX))
        );
    }

    #[test]
    fn attack_semantics_use_the_canonical_motion_names() {
        let catalog = AnimationPackCatalog::biped_root().unwrap();
        let root = &catalog.packs[&AnimationPackId::HUMANOID_UNARMED];
        assert_eq!(
            root.poses[&SemanticPose::AttackThrust].motion,
            MotionId::THRUST
        );
        assert_eq!(
            root.poses[&SemanticPose::AttackSwing].motion,
            MotionId::SWING
        );
        assert_eq!(
            root.poses[&SemanticPose::ContinueSwing].motion,
            MotionId::SWING
        );
        assert_eq!(
            root.poses[&SemanticPose::AttackOffhand].motion,
            MotionId::OFFHAND
        );
    }

    #[test]
    fn combat_cycle_ik_switches_exclusive_support_at_authored_contacts() {
        assert_eq!(
            combat_cycle_ik_weights(CyclePhase::from_cycles(LEFT_CONTACT_PHASE).unwrap()),
            Vec2::Y
        );
        assert_eq!(
            combat_cycle_ik_weights(CyclePhase::from_cycles(LEFT_SUPPORT_SHOULDER_PHASE).unwrap()),
            Vec2::Y
        );
        assert_eq!(
            combat_cycle_ik_weights(CyclePhase::from_cycles(LEFT_PASSING_PHASE).unwrap()),
            Vec2::X
        );
        assert_eq!(
            combat_cycle_ik_weights(CyclePhase::from_cycles(RIGHT_SUPPORT_SHOULDER_PHASE).unwrap()),
            Vec2::X
        );
        assert_eq!(
            combat_cycle_ik_weights(CyclePhase::from_cycles(RIGHT_CONTACT_PHASE).unwrap()),
            Vec2::X
        );
        assert_eq!(
            combat_cycle_ik_weights(CyclePhase::from_cycles(RIGHT_RELEASE_SHOULDER_PHASE).unwrap()),
            Vec2::X
        );
        assert_eq!(
            combat_cycle_ik_weights(CyclePhase::from_cycles(RIGHT_PASSING_PHASE).unwrap()),
            Vec2::Y
        );
        assert_eq!(
            combat_cycle_ik_weights(CyclePhase::from_cycles(LEFT_RELEASE_SHOULDER_PHASE).unwrap()),
            Vec2::Y
        );
    }

    #[test]
    fn planted_and_moving_combat_both_component_blend_the_pelvis() {
        let mut planted = SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        planted
            .begin_attack(
                AttackSpec::default(),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(100u64),
            )
            .unwrap();
        let planted_evaluation = AnimationEvaluation::from_skeleton(&planted).unwrap();
        assert!(!planted_evaluation.action.is_empty());
        assert!(!planted_evaluation.lower_body.is_empty());
        assert!(combat_pelvis_is_component_blended(
            &planted,
            &planted_evaluation
        ));

        let moving = planted
            .clone()
            .with_raised_locomotion(RaisedLocomotionIntent::moving(
                Vec2::X,
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(1.0)
                    .unwrap(),
            ));
        let moving_evaluation = AnimationEvaluation::from_skeleton(&moving).unwrap();
        assert!(!moving_evaluation.action.is_empty());
        assert!(combat_pelvis_is_component_blended(
            &moving,
            &moving_evaluation
        ));

        let mut quickstep = SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        quickstep
            .begin_dodge(
                DodgeSpec::quickstep(Vec2::X).unwrap(),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(100u64),
            )
            .unwrap();
        let quickstep_evaluation = AnimationEvaluation::from_skeleton(&quickstep).unwrap();
        assert!(!combat_pelvis_is_component_blended(
            &quickstep,
            &quickstep_evaluation
        ));
    }

    #[test]
    fn authored_quickstep_timeline_never_requests_leg_ik() {
        let mut state = SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        state
            .begin_dodge(
                DodgeSpec::quickstep(Vec2::X).unwrap(),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(100u64),
            )
            .unwrap();
        // The first half is the interpolation from the planted stance into
        // authored takeoff. Once frame 0 owns the lower body, every authored
        // quickstep pose remains completely free of leg IK.
        for tick in [100, 150, 199] {
            state.advance_action(
                adventuresim_tactical_core::animation::AnimationSampleTick::from(tick as u64),
            );
            let evaluation = AnimationEvaluation::from_skeleton(&state).unwrap();
            assert_eq!(
                semantic_foot_ik_weights(&evaluation).unwrap(),
                Vec2::ZERO,
                "{tick}"
            );
        }
    }
}

#[cfg(test)]
mod tests;
