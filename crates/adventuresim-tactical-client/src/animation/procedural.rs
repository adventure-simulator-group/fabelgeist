use super::{
    AnimationElapsed, AnimationPlayback, AnimationValueError, AnimationValueResult,
    AuthoredBindTransform, BlendWeight, PresentedSkeleton,
};
use adventuresim_tactical_core::animation::{AnimationPackId, LandingImpactSpeed};
use adventuresim_tactical_core::animation::{
    AnimationQuantity, NonnegativeLength, SettleProgress, UnitInterval,
};
use adventuresim_tactical_core::prelude::*;
use bevy::{math::Affine3A, prelude::*};
pub(in crate::animation) use body_response::apply_locomotion_body_response;
#[cfg(test)]
use body_response::body_response_target;
use body_response::presentation_tick_delta;
pub(crate) use body_values::BodyAngle;
use body_values::{
    BodyAngularMagnitude, BodyAngularVelocity, BodyLocalAcceleration, BodyLocalVelocity,
    BodyResponseAngles, BodyResponseStep, BrakingLeanScale,
};
pub(crate) use clock::ProceduralAnimationClock;
use clock::{AnimationSampleTick, FixedAnimationStep};
pub(super) use dive::apply_procedural_dive_lower_body;
use height::clear_landing_foot_plants;
#[cfg(test)]
use height::{
    authored_height_compensation, landing_compression_for_action, landing_compression_for_impact,
    locomotion_height_wave,
};
pub(crate) use ik::{
    ArmIkState, HandIkTarget, HandSide, HeldWeaponConstraint, HumanoidIkTargets, LegIkDiagnostics,
    LegIkState, RaisedFootworkState, locomotion_support_weights, measured_ankle_sole_offset_metres,
    sole_contact_tolerance_metres,
};
use ik::{
    FootTrackSide, LimbSegments, TwoBoneChain, apply_two_bone_solution, canonical_knee_pole,
    constrain_rendered_leg_pole, smoothstep, snapshot_chain, solve_landing_two_bone,
};
#[cfg(test)]
use ik::{
    TwoBoneSolution, advance_foot_target_at_speed, advance_pelvis_shift, authored_knee_pole_world,
    constrain_foot_to_track, constrain_target_to_reach, foot_track_inner_metres,
    landing_maximum_reach, maximum_pelvis_correction_step_metres, maximum_reach,
    minimum_inter_foot_separation_metres, plant_is_continuous, raised_footwork_posture_is_valid,
    retained_plant_requires_release, secondary_grip_world, slope_aligned_world_rotation,
    sole_is_at_contact, solve_two_bone, terrain_conformed_guard_target,
    terrain_ik_posture_is_valid, terrain_leg_has_support,
};
pub(super) use ik::{
    apply_arm_and_weapon_constraints, apply_terrain_leg_ik, enforce_anatomical_knee_yaw,
    refresh_raised_support_after_propagation,
};
pub(crate) use rig::*;
use sample_state::{BodyObservation, HeightHistory};
pub(crate) use sample_state::{LocomotionBodyResponseState, LocomotionHeightState};
use std::collections::BTreeMap;

mod body_response;
mod body_values;
pub(in crate::animation) mod clock;
mod dive;
mod height;
mod height_values;
mod ik;
mod rig;
mod sample_state;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct ProceduralLookState {
    base_rotation: Quat,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct ProceduralJumpAnticipationState {
    base: Transform,
    applied: Transform,
    amount: f32,
    pelvis_amount: f32,
    evaluation_tick: AnimationSampleTick,
}

#[derive(Resource, Debug, Default)]
pub(super) struct FixedTickPoseCache {
    tick: Option<AnimationSampleTick>,
    bones: BTreeMap<Entity, Transform>,
}

#[derive(Clone, Copy)]
struct MirrorBone {
    entity: Entity,
    local: Transform,
    global: GlobalTransform,
    parent: Option<Entity>,
    parent_global: GlobalTransform,
}

#[derive(Clone, Copy, Debug)]
struct BoneSnapshot {
    entity: Entity,
    global: GlobalTransform,
    parent_rotation: Quat,
}

pub(crate) fn authored_locomotion_ik_owns(skeleton: &SkeletonState) -> bool {
    ik::authored_locomotion_owns(skeleton)
}

/// Tools can render gameplay, side, and front views from one logical sample.
/// Preserve the first complete post-procedural local pose for that fixed tick
/// and restore it before propagation on every repeated render. Live gameplay
/// leaves the fixed clock unset and never enters this path.
pub(super) fn stabilize_repeated_fixed_tick_pose(
    clock: Res<ProceduralAnimationClock>,
    mut cache: ResMut<FixedTickPoseCache>,
    mut bones: Query<(Entity, &mut Transform), With<MhrBone>>,
) {
    let Some(FixedAnimationStep { tick, .. }) = clock.fixed_step() else {
        cache.tick = None;
        cache.bones.clear();
        return;
    };
    if cache.tick != Some(tick) {
        cache.tick = Some(tick);
        cache.bones.clear();
        cache.bones.extend(
            bones
                .iter_mut()
                .map(|(entity, transform)| (entity, *transform)),
        );
        return;
    }
    for (entity, mut transform) in &mut bones {
        if let Some(cached) = cache.bones.get(&entity) {
            *transform = *cached;
        }
    }
}

/// Remove last frame's procedural look before authored and other procedural
/// layers evaluate. This makes the look pass an explicit base-plus-offset
/// operation instead of inferring whether an incoming rotation already
/// contains its own previous output.
pub(super) fn restore_procedural_look_base(
    mut bones: Query<(&mut Transform, &ProceduralLookState)>,
) {
    for (mut transform, state) in &mut bones {
        transform.rotation = state.base_rotation;
    }
}

/// Procedural facing is an additive post-FK layer. Sparse authored clips do not
/// necessarily rewrite every torso bone, so retain the pre-look local rotation
/// and reuse it when the same logical pose reaches this pass again.
#[expect(
    clippy::type_complexity,
    reason = "the Bevy ParamSet must expose separate read-only and mutable transform queries"
)]
pub(super) fn apply_head_and_torso_look(
    mut commands: Commands,
    owners: Query<(Entity, &CharacterLook, &PresentedSkeleton)>,
    mut transforms: ParamSet<(
        Query<&Transform>,
        Query<(
            Entity,
            &HumanoidBone,
            &mut Transform,
            Option<&mut ProceduralLookState>,
        )>,
    )>,
) {
    let owner_look = owners
        .iter()
        .filter_map(|(owner, look, skeleton)| {
            // This pass runs before transform propagation. GlobalTransform is
            // therefore one frame behind the current replicated root and
            // would reapply that root yaw across the neck chain.
            let owner_rotation = transforms.p0().get(owner).ok()?.rotation;
            Some((
                owner,
                (
                    guarded_camera_look(
                        look,
                        owner_rotation,
                        skeleton.weapon_guard(),
                        skeleton.posture_transition(),
                    ),
                    skeleton.action_direction().x.clamp(-1.0, 1.0) * 0.35,
                ),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    for (entity, bone, mut transform, state) in &mut transforms.p1() {
        let Some(&(aim, directional_yaw)) = owner_look.get(&bone.owner) else {
            continue;
        };
        let weight = match bone.role {
            BoneRole::StomachOne => 0.08,
            BoneRole::StomachTwo => 0.12,
            BoneRole::StomachThree => 0.16,
            BoneRole::Chest => 0.18,
            BoneRole::NeckOne => 0.2,
            BoneRole::Head => 0.26,
            _ => continue,
        };
        let aim_offset = match (aim, bone.role) {
            (Some(offset), BoneRole::NeckOne | BoneRole::Head) => offset,
            _ => Vec2::ZERO,
        };
        // Owner yaw is already on the character transform. Only the bounded
        // residual camera direction and local action offset are distributed
        // here. Camera pitch belongs exclusively to held aim/block; ordinary
        // camera look must not nod the presented head and neck.
        let offset = Quat::from_euler(
            EulerRot::YXZ,
            directional_yaw * weight + aim_offset.x,
            aim_offset.y,
            0.0,
        );
        let (rotation, next) = additive_look_rotation(transform.rotation, offset);
        transform.rotation = rotation;
        if let Some(mut state) = state {
            *state = next;
        } else {
            commands.entity(entity).insert(next);
        }
    }
}

/// Space begins an upright jump with a small procedural anticipation rather
/// than launching on the press edge. FK remains authored; this layer lowers
/// the pelvis and shares a modest forward fold across the spine.
pub(super) fn apply_jump_anticipation(
    mut commands: Commands,
    owners: Query<&PresentedSkeleton>,
    mut bones: Query<(
        Entity,
        &HumanoidBone,
        &mut Transform,
        Option<&mut ProceduralJumpAnticipationState>,
    )>,
) {
    for (entity, bone, mut transform, state) in &mut bones {
        let Ok(skeleton) = owners.get(bone.owner) else {
            continue;
        };
        let charging =
            skeleton.jump_anticipation() == JumpAnticipation::Charging && !skeleton.is_quickstep();
        let previous = state.as_deref().copied();
        let base = previous.map_or(*transform, |previous| {
            if previous.evaluation_tick == skeleton.locomotion_sample_tick
                || transform
                    .translation
                    .abs_diff_eq(previous.applied.translation, 0.000_01)
                    && transform.rotation.angle_between(previous.applied.rotation) <= 0.000_01
            {
                previous.base
            } else {
                *transform
            }
        });
        let previous_amount = previous.map_or(0.0, |previous| previous.amount);
        let previous_pelvis_amount = previous.map_or(0.0, |previous| previous.pelvis_amount);
        let amount = if skeleton.is_quickstep() {
            0.0
        } else if previous
            .is_some_and(|previous| previous.evaluation_tick == skeleton.locomotion_sample_tick)
        {
            previous_amount
        } else {
            let target = charging as u8 as f32;
            previous_amount + (target - previous_amount).clamp(-0.125, 0.125)
        };
        let pelvis_amount = if skeleton.is_quickstep() {
            0.0
        } else if previous
            .is_some_and(|previous| previous.evaluation_tick == skeleton.locomotion_sample_tick)
        {
            previous_pelvis_amount
        } else {
            let target =
                jump_charge_pelvis_target(skeleton.jump_anticipation(), skeleton.weapon_guard());
            previous_pelvis_amount + (target - previous_pelvis_amount).clamp(-0.125, 0.125)
        };
        let mut applied = base;
        if amount > 0.0 {
            match bone.role {
                BoneRole::Pelvis => applied.translation.y -= 0.12 * pelvis_amount,
                BoneRole::StomachOne => {
                    applied.rotation =
                        (applied.rotation * Quat::from_rotation_x(0.075 * amount)).normalize()
                }
                BoneRole::StomachTwo => {
                    applied.rotation =
                        (applied.rotation * Quat::from_rotation_x(0.06 * amount)).normalize()
                }
                BoneRole::StomachThree => {
                    applied.rotation =
                        (applied.rotation * Quat::from_rotation_x(0.05 * amount)).normalize()
                }
                BoneRole::Chest => {
                    applied.rotation =
                        (applied.rotation * Quat::from_rotation_x(0.04 * amount)).normalize()
                }
                _ => continue,
            }
        } else if !matches!(
            bone.role,
            BoneRole::Pelvis
                | BoneRole::StomachOne
                | BoneRole::StomachTwo
                | BoneRole::StomachThree
                | BoneRole::Chest
        ) {
            continue;
        }
        *transform = applied;
        let next = ProceduralJumpAnticipationState {
            base,
            applied,
            amount,
            pelvis_amount,
            evaluation_tick: skeleton.locomotion_sample_tick,
        };
        if let Some(mut state) = state {
            *state = next;
        } else {
            commands.entity(entity).insert(next);
        }
    }
}

/// Applies whole-body reflection for same-pack semantic pose fallbacks.
/// Ordinary gait parity is already baked into distinct endpoint clips before
/// FK blending and must never pass through this post-blend operation.
pub(super) fn apply_pose_mirroring(
    playbacks: Query<&AnimationPlayback>,
    rigs: Query<(Entity, &HumanoidRig)>,
    parents: Query<&ChildOf>,
    mut transforms: ParamSet<(Query<&Transform>, TransformHelper, Query<&mut Transform>)>,
) {
    for (owner, topology) in &rigs {
        let Ok(playback) = playbacks.get(owner) else {
            continue;
        };
        let whole_body_weight = playback.whole_body_mirror;
        if whole_body_weight == UnitInterval::ZERO {
            continue;
        }
        let Some(rig_scene) = topology.rig_scene() else {
            continue;
        };
        let mirror_entities = topology
            .mirror_centers()
            .iter()
            .copied()
            .chain(
                topology
                    .mirror_pairs()
                    .iter()
                    .flat_map(|pair| [pair.left, pair.right]),
            )
            .collect::<Vec<_>>();
        let local_snapshots = {
            let locals = transforms.p0();
            mirror_entities
                .iter()
                .filter_map(|&entity| Some((entity, *locals.get(entity).ok()?)))
                .collect::<Vec<_>>()
        };
        let (rig, rig_global) = {
            let helper = transforms.p1();
            let Ok(rig_global) = helper.compute_global_transform(rig_scene) else {
                continue;
            };
            let mut rig = BTreeMap::new();
            for (entity, local) in local_snapshots {
                let Ok(global) = helper.compute_global_transform(entity) else {
                    continue;
                };
                let parent = parents.get(entity).ok().map(ChildOf::parent);
                let parent_global = parent
                    .and_then(|parent| helper.compute_global_transform(parent).ok())
                    .unwrap_or(GlobalTransform::IDENTITY);
                rig.insert(
                    entity,
                    MirrorBone {
                        entity,
                        local,
                        global,
                        parent,
                        parent_global,
                    },
                );
            }
            (rig, rig_global)
        };
        let mut desired_globals = BTreeMap::<Entity, Affine3A>::new();
        let mut mirror_weights = BTreeMap::<Entity, UnitInterval>::new();
        for &entity in topology.mirror_centers() {
            let Some(bone) = rig.get(&entity) else {
                continue;
            };
            desired_globals.insert(bone.entity, mirrored_global_affine(bone.global, rig_global));
            mirror_weights.insert(bone.entity, whole_body_weight);
        }
        for pair in topology.mirror_pairs() {
            let (Some(left), Some(right)) = (rig.get(&pair.left), rig.get(&pair.right)) else {
                continue;
            };
            desired_globals.insert(
                left.entity,
                mirrored_global_affine(right.global, rig_global),
            );
            desired_globals.insert(
                right.entity,
                mirrored_global_affine(left.global, rig_global),
            );
            mirror_weights.insert(left.entity, whole_body_weight);
            mirror_weights.insert(right.entity, whole_body_weight);
        }
        let mut bones = transforms.p2();
        for bone in rig.values() {
            let Some(&desired_global) = desired_globals.get(&bone.entity) else {
                continue;
            };
            let Some(&weight) = mirror_weights.get(&bone.entity) else {
                continue;
            };
            if weight == UnitInterval::ZERO {
                continue;
            }
            let desired_parent = bone
                .parent
                .and_then(|parent| desired_globals.get(&parent).copied())
                .unwrap_or_else(|| bone.parent_global.affine());
            let (scale, rotation, translation) =
                (desired_parent.inverse() * desired_global).to_scale_rotation_translation();
            if !translation.is_finite() || !rotation.is_finite() || !scale.is_finite() {
                continue;
            }
            if let Ok(mut transform) = bones.get_mut(bone.entity) {
                transform.translation = bone.local.translation.lerp(translation, weight.fraction());
                transform.rotation = bone.local.rotation.slerp(rotation, weight.fraction());
                transform.scale = bone.local.scale.lerp(scale, weight.fraction());
            }
        }
    }
}

/// Applies the admitted scalar height treatment after authored pose evaluation.
pub(super) fn apply_locomotion_height(
    mut commands: Commands,
    mut owners: Query<(
        Entity,
        &PresentedSkeleton,
        Option<&mut LocomotionHeightState>,
    )>,
    mut bones: Query<(&HumanoidBone, &mut Transform)>,
) {
    let mut heights = BTreeMap::new();
    for (owner, skeleton, state) in &mut owners {
        let current = state.as_deref().copied().unwrap_or_default();
        let next = match current.advance_height(skeleton) {
            Ok(next) => next,
            Err(error) => {
                warn!(%error, ?owner, "Rejected locomotion height sample");
                continue;
            }
        };
        if !skeleton.is_quickstep() {
            heights.insert(owner, next);
        }
        if let Some(mut state) = state {
            *state = next;
        } else {
            commands.entity(owner).insert(next);
        }
    }
    for (bone, mut transform) in &mut bones {
        let Some(height) = heights.get(&bone.owner) else {
            continue;
        };
        if height.displayed_wave.metres() == 0.0 {
            continue;
        }
        if bone.role == BoneRole::Root {
            let translated_y = transform.translation.y + height.displayed_wave.metres();
            if !translated_y.is_finite() {
                warn!(error = %AnimationValueError::Overflow { quantity: AnimationQuantity::HeightOffset }, owner = ?bone.owner, "Rejected root height displacement");
                continue;
            }
            transform.translation.y = translated_y;
        }
    }
}

/// Turns authored walk/run legs into the movement direction while preserving
/// the weapon layer's world-facing torso. The pelvis owns the yaw and the
/// first spine bone cancels it, so descendants on either side of the layer
/// boundary remain independent.
pub(super) fn orient_guarded_run_lower_body(
    owners: Query<&PresentedSkeleton>,
    rigs: Query<(Entity, &HumanoidRig)>,
    mut transforms: Query<&mut Transform>,
) {
    for (owner, rig) in &rigs {
        let Ok(skeleton) = owners.get(owner) else {
            continue;
        };
        if skeleton.is_quickstep()
            || !skeleton.is_guarded_sprinting()
            || !skeleton.raised_locomotion().is_moving()
        {
            continue;
        }
        let direction = skeleton.raised_locomotion().local_direction();
        let Some((pelvis_yaw, spine_counter_yaw)) = guarded_run_split_yaw(direction) else {
            continue;
        };
        let (Some(&pelvis), Some(&spine)) =
            (rig.get(&BoneRole::Pelvis), rig.get(&BoneRole::StomachOne))
        else {
            continue;
        };
        let Ok([mut pelvis, mut spine]) = transforms.get_many_mut([pelvis, spine]) else {
            continue;
        };
        pelvis.rotation = (pelvis.rotation * pelvis_yaw).normalize();
        spine.rotation = (spine_counter_yaw * spine.rotation).normalize();
    }
}

/// Preserves both world-space feet during the short landing-only pelvis
/// compression by flexing the actual hip/knee chains. This is independent of
/// the opt-in terrain solver and never translates or stretches thigh roots.
pub(super) fn apply_landing_leg_compression(
    mut owners: Query<
        (
            &PresentedSkeleton,
            &mut LocomotionHeightState,
            &GlobalTransform,
        ),
        Without<HumanoidBone>,
    >,
    rigs: Query<(Entity, &HumanoidRig)>,
    bones: Query<(Entity, &GlobalTransform), With<HumanoidBone>>,
    parents: Query<&ChildOf>,
    mut transforms: ParamSet<(TransformHelper, Query<&mut Transform>)>,
) {
    let _spike = crate::animation::diagnostics::SpikeGuard::new("apply_landing_leg_compression");
    let mut previous_world_positions = BTreeMap::<Entity, Vec3>::new();
    for (entity, global) in &bones {
        previous_world_positions.insert(entity, global.translation());
    }
    for (owner, rig) in &rigs {
        let Ok((skeleton, mut height, owner_transform)) = owners.get_mut(owner) else {
            continue;
        };
        if !height.supports_foot_plants(skeleton) {
            clear_landing_foot_plants(&mut height);
            continue;
        }
        let owner_position = owner_transform.translation();
        if height.landing_plant_resync_tick == Some(skeleton.locomotion_sample_tick) {
            continue;
        }
        height.landing_plant_resync_tick = None;
        let discontinuous = landing_plant_is_discontinuous(
            height.landing_plant_tick,
            skeleton.locomotion_sample_tick,
            height.landing_plant_owner_position,
            owner_position,
        );
        if discontinuous {
            clear_landing_foot_plants(&mut height);
            height.landing_plant_resync_tick = Some(skeleton.locomotion_sample_tick);
            continue;
        }
        height.landing_plant_tick = Some(skeleton.locomotion_sample_tick);
        height.landing_plant_owner_position = Some(owner_position);
        // Native root-local displacement enters the skeletal vector kernel here.
        let landing_compression = height.landing_compression;
        height.apply_landing_pelvis_compression(rig, &parents, &mut transforms);
        for (upper_role, lower_role, foot_role, foot_side) in [
            (
                BoneRole::ThighLeft,
                BoneRole::ShinLeft,
                BoneRole::FootLeft,
                LeadFoot::Left,
            ),
            (
                BoneRole::ThighRight,
                BoneRole::ShinRight,
                BoneRole::FootRight,
                LeadFoot::Right,
            ),
        ] {
            let (Some(&upper), Some(&lower), Some(&foot)) = (
                rig.get(&upper_role),
                rig.get(&lower_role),
                rig.get(&foot_role),
            ) else {
                continue;
            };
            let Some((upper_snapshot, lower_snapshot, foot_snapshot)) =
                snapshot_chain(upper, lower, foot, &parents, &transforms.p0())
            else {
                continue;
            };
            let LimbSegments {
                upper: upper_length,
                lower: lower_length,
            } = match LimbSegments::measure(
                upper_snapshot.global.translation(),
                lower_snapshot.global.translation(),
                foot_snapshot.global.translation(),
            ) {
                Ok(segments) => segments,
                Err(error) => {
                    warn!(%error, "landing segment measurement rejected");
                    continue;
                }
            };
            let retained_target = match foot_side {
                LeadFoot::Left => &mut height.landing_left_foot_target,
                LeadFoot::Right => &mut height.landing_right_foot_target,
            };
            let target =
                retain_landing_foot_target(retained_target, previous_world_positions[&foot]);
            let side = match foot_side {
                LeadFoot::Left => FootTrackSide::NegativeX,
                LeadFoot::Right => FootTrackSide::PositiveX,
            };
            let pole = owner_transform.rotation() * canonical_knee_pole(side);
            let pole = constrain_rendered_leg_pole(
                rig,
                foot_side,
                upper_snapshot.global.translation(),
                foot_snapshot.global.translation(),
                target,
                pole,
                &parents,
                &transforms.p0(),
            );
            let solution = match solve_landing_two_bone(
                TwoBoneChain::new(
                    upper_snapshot.global.translation(),
                    lower_snapshot.global.translation(),
                    foot_snapshot.global.translation(),
                    upper_length,
                    lower_length,
                    pole,
                ),
                target,
                landing_compression,
            ) {
                Ok(solution) => solution,
                Err(error) => {
                    warn!(%error, "landing geometry rejected");
                    continue;
                }
            };
            if let Some(solution) = solution {
                apply_two_bone_solution(upper, lower, foot, solution, &parents, &mut transforms);
            }
        }
    }
}

fn procedural_tuning() -> ProceduralAnimationConfig {
    runtime_animation_config().procedural
}

fn additive_look_rotation(current: Quat, offset: Quat) -> (Quat, ProceduralLookState) {
    let applied_rotation = (current * offset).normalize();
    (
        applied_rotation,
        ProceduralLookState {
            base_rotation: current,
        },
    )
}

fn guarded_camera_look(
    look: &CharacterLook,
    owner_rotation: Quat,
    weapon_guard: WeaponGuardState,
    posture_transition: Option<PostureTransitionState>,
) -> Option<Vec2> {
    (weapon_guard == WeaponGuardState::Raised && posture_transition.is_none())
        .then(|| constrained_camera_look(look, owner_rotation))
}

fn constrained_camera_look(look: &CharacterLook, owner_rotation: Quat) -> Vec2 {
    let tuning = procedural_tuning();
    let joint_limit = tuning.guarded_look_joint_limit_degrees.radians();
    let camera_forward = Quat::from_euler(EulerRot::YXZ, look.yaw, look.pitch, 0.0) * Vec3::NEG_Z;
    let local = owner_rotation.inverse() * camera_forward;
    // Directly behind the body there is no anatomically preferable left/right
    // twist. Using signed Z puts atan2's branch cut at exactly that ambiguous
    // direction and can flip the neck chain between both joint limits. Fold
    // the rear hemisphere onto neutral instead: side look remains signed and
    // continuous, while a camera directly behind leaves the neck untwisted.
    let yaw = local.x.atan2(local.z.abs());
    let pitch = local.y.atan2(local.xz().length());
    Vec2::new(
        tuning
            .guarded_look_joint_count
            .distribute_radians(yaw)
            .clamp(-joint_limit, joint_limit),
        tuning
            .guarded_look_joint_count
            .distribute_radians(pitch)
            .clamp(-joint_limit, joint_limit),
    )
}

fn jump_charge_pelvis_target(anticipation: JumpAnticipation, guard: WeaponGuardState) -> f32 {
    (anticipation == JumpAnticipation::Charging && guard == WeaponGuardState::Lowered) as u8 as f32
}

/// Split local travel yaw between the pelvis and its inverse leg compensation.
fn guarded_run_split_yaw(direction: Vec2) -> Option<(Quat, Quat)> {
    let direction = direction.try_normalize()?;
    // Local movement uses -Y for character-forward, corresponding to -Z in
    // the authored rig. Positive X is character-right.
    let angle = -direction.x.atan2(-direction.y);
    let pelvis = Quat::from_rotation_y(angle);
    Some((pelvis, pelvis.inverse()))
}

fn retain_landing_foot_target(
    retained: &mut Option<Vec3>,
    pre_landing_foot_position: Vec3,
) -> Vec3 {
    *retained.get_or_insert(pre_landing_foot_position)
}

fn landing_plant_is_discontinuous(
    previous_tick: Option<AnimationSampleTick>,
    current_tick: AnimationSampleTick,
    previous_owner_position: Option<Vec3>,
    current_owner_position: Vec3,
) -> bool {
    presentation_tick_delta(previous_tick, current_tick).is_none()
        || previous_owner_position
            .is_some_and(|previous| previous.distance_squared(current_owner_position) > 4.0)
}

fn mirrored_global_affine(source: GlobalTransform, owner: GlobalTransform) -> Affine3A {
    let owner_affine = owner.affine();
    let relative = owner_affine.inverse() * source.affine();
    let (scale, rotation, translation) = relative.to_scale_rotation_translation();
    let mirrored = mirrored_across_anatomical_center(Transform {
        translation,
        rotation,
        scale,
    });
    owner_affine * mirrored.compute_affine()
}

fn mirrored_across_anatomical_center(mut transform: Transform) -> Transform {
    transform.translation.x = -transform.translation.x;
    let rotation = transform.rotation;
    transform.rotation = Quat::from_xyzw(rotation.x, -rotation.y, -rotation.z, rotation.w);
    transform
}

#[cfg(test)]
mod contract_tests {
    use super::*;

    #[test]
    fn walk_preserves_authored_root_and_pelvis_translation() {
        let mut app = App::new();
        app.add_systems(Update, apply_locomotion_height);
        let state = SkeletonState::default()
            .with_local_velocity(
                Vec3::NEG_Z
                    * walk_locomotion_profile()
                        .reference_speed
                        .metres_per_second(),
            )
            .with_gait_phase(CyclePhase::from_cycles(0.25).unwrap())
            .with_locomotion_sample_tick(
                adventuresim_tactical_core::animation::AnimationSampleTick::from(1u64),
            );
        let owner = app
            .world_mut()
            .spawn(PresentedSkeleton::new(state, None))
            .id();
        let authored_root = Transform::from_xyz(0.03, 0.81, -0.04);
        let authored_pelvis = Transform::from_xyz(-0.02, 0.92, 0.05);
        let root = app
            .world_mut()
            .spawn((
                HumanoidBone {
                    owner,
                    role: BoneRole::Root,
                },
                authored_root,
            ))
            .id();
        let pelvis = app
            .world_mut()
            .spawn((
                HumanoidBone {
                    owner,
                    role: BoneRole::Pelvis,
                },
                authored_pelvis,
            ))
            .id();

        app.update();

        assert_eq!(*app.world().get::<Transform>(root).unwrap(), authored_root);
        assert_eq!(
            *app.world().get::<Transform>(pelvis).unwrap(),
            authored_pelvis
        );
    }

    #[test]
    fn run_flight_is_additive_without_replacing_authored_pelvis_translation() {
        let mut app = App::new();
        app.add_systems(Update, apply_locomotion_height);
        let state = SkeletonState::default()
            .with_local_velocity(
                Vec3::NEG_Z * run_locomotion_profile().reference_speed.metres_per_second(),
            )
            .with_gait_phase(CyclePhase::from_cycles(0.25).unwrap())
            .with_locomotion_sample_tick(
                adventuresim_tactical_core::animation::AnimationSampleTick::from(1u64),
            );
        let owner = app
            .world_mut()
            .spawn(PresentedSkeleton::new(state, None))
            .id();
        let authored_root = Transform::from_xyz(0.03, 0.81, -0.04);
        let authored_pelvis = Transform::from_xyz(-0.02, 0.92, 0.05);
        let root = app
            .world_mut()
            .spawn((
                HumanoidBone {
                    owner,
                    role: BoneRole::Root,
                },
                authored_root,
            ))
            .id();
        let pelvis = app
            .world_mut()
            .spawn((
                HumanoidBone {
                    owner,
                    role: BoneRole::Pelvis,
                },
                authored_pelvis,
            ))
            .id();

        app.update();

        let expected_flight = run_locomotion_profile().flight_apex_metres.metres()
            - procedural_tuning()
                .authored_ordinary_passing_rise_metres
                .metres();
        let displayed_root = app.world().get::<Transform>(root).unwrap();
        assert!(
            (displayed_root.translation.y - (authored_root.translation.y + expected_flight)).abs()
                <= 0.0001
        );
        assert_eq!(
            displayed_root.translation.xz(),
            authored_root.translation.xz()
        );
        assert_eq!(
            *app.world().get::<Transform>(pelvis).unwrap(),
            authored_pelvis
        );
    }

    #[test]
    fn measured_sole_offset_matches_the_authored_rig() {
        assert_eq!(measured_ankle_sole_offset_metres().metres(), 0.085);
    }

    #[test]
    fn presentation_tick_delta_handles_wrap_and_rejects_large_gaps() {
        assert_eq!(
            presentation_tick_delta(
                Some(AnimationSampleTick::from(u64::MAX)),
                AnimationSampleTick::ZERO
            ),
            Some(AnimationSampleSpan::ONE)
        );
        assert_eq!(
            presentation_tick_delta(
                Some(AnimationSampleTick::from(1u64)),
                AnimationSampleTick::from(100u64)
            ),
            None
        );
    }

    #[test]
    fn guarded_run_pelvis_yaw_is_cancelled_at_the_upper_body_boundary() {
        for direction in [Vec2::NEG_Y, Vec2::X, Vec2::Y, Vec2::NEG_X] {
            let (pelvis, spine) = guarded_run_split_yaw(direction).unwrap();
            assert!((pelvis * spine).abs_diff_eq(Quat::IDENTITY, 0.0001));
            let expected = Vec3::new(direction.x, 0.0, direction.y);
            assert!((pelvis * Vec3::NEG_Z).abs_diff_eq(expected, 0.0001));
        }
    }

    #[test]
    fn raised_guard_never_adds_jump_charge_pelvis_lowering() {
        assert_eq!(
            jump_charge_pelvis_target(JumpAnticipation::Charging, WeaponGuardState::Raised),
            0.0
        );
        assert_eq!(
            jump_charge_pelvis_target(JumpAnticipation::Charging, WeaponGuardState::Lowered),
            1.0
        );
    }
}

#[cfg(test)]
mod ik_tests {
    use super::*;
    use crate::animation::procedural::ik::ReachLimit;
    use adventuresim_building_generator::spatial_geometry::SignedLength;
    use adventuresim_core::linear_speed::NonnegativeSpeed;
    use adventuresim_tactical_core::animation::NonnegativeLength;
    use adventuresim_tactical_core::city_layout::grounding::SupportElevation;

    #[test]
    fn canonical_mhr_role_names_are_recognized() {
        for name in [
            "body_world",
            "root",
            "c_spine0",
            "c_spine1",
            "c_spine2",
            "c_spine3",
            "c_neck",
            "c_head",
            "c_camera",
            "l_upleg",
            "r_lowleg",
            "l_lowarm",
            "r_weapon",
            "l_ball",
        ]
        .map(fabelgeist_rig::RigJointName::from)
        {
            assert!(BoneRole::from_name(&name).is_some(), "missing {name}");
        }
        assert_eq!(BoneRole::from_name(&"l_upleg_twist2_proc".into()), None);
        assert_eq!(BoneRole::from_name(&"Cylinder".into()), None);
    }

    #[test]
    fn raised_camera_look_is_shared_across_three_heavily_limited_joints() {
        let look = CharacterLook {
            yaw: std::f32::consts::FRAC_PI_2,
            pitch: 1.2,
        };
        let per_joint = constrained_camera_look(&look, Quat::IDENTITY);
        assert!(per_joint.x.abs() <= std::f32::consts::PI / 8.0 + 0.000_01);
        assert!(per_joint.y.abs() <= std::f32::consts::PI / 8.0 + 0.000_01);
        assert!(per_joint.x.abs() > 0.3);
        assert!(per_joint.y.abs() > 0.3);
    }

    #[test]
    fn camera_pitch_reaches_head_and_neck_only_while_guard_is_raised() {
        let look = CharacterLook {
            yaw: 0.0,
            pitch: 0.6,
        };
        assert_eq!(
            guarded_camera_look(&look, Quat::IDENTITY, WeaponGuardState::Lowered, None),
            None
        );
        assert!(
            guarded_camera_look(&look, Quat::IDENTITY, WeaponGuardState::Raised, None)
                .unwrap()
                .y
                .abs()
                > 0.1
        );
        let mut transitioning = SkeletonState::default();
        transitioning.begin_posture_transition(
            PostureTransitionKind::UprightToProne,
            adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
            adventuresim_tactical_core::animation::AnimationSampleSpan::from(10u64),
        );
        assert_eq!(
            guarded_camera_look(
                &look,
                Quat::IDENTITY,
                WeaponGuardState::Raised,
                transitioning.posture_transition(),
            ),
            None,
            "authored posture transitions exclusively own the spine and head"
        );
    }

    #[test]
    fn current_root_facing_removes_camera_yaw_from_the_neck_chain() {
        let look = CharacterLook {
            yaw: 0.7,
            pitch: 0.0,
        };
        let camera_forward = Quat::from_rotation_y(look.yaw) * Vec3::NEG_Z;
        let current_root = Quat::from_rotation_y(camera_forward.x.atan2(camera_forward.z));

        assert!(constrained_camera_look(&look, Quat::IDENTITY).x.abs() > 0.1);
        assert!(constrained_camera_look(&look, current_root).x.abs() <= 0.000_01);
    }

    #[test]
    fn rear_camera_crossing_cannot_flip_between_neck_joint_limits() {
        let left_of_rear = CharacterLook {
            yaw: -0.001,
            pitch: 0.0,
        };
        let right_of_rear = CharacterLook {
            yaw: 0.001,
            pitch: 0.0,
        };
        let left = constrained_camera_look(&left_of_rear, Quat::IDENTITY).x;
        let right = constrained_camera_look(&right_of_rear, Quat::IDENTITY).x;
        assert!(left.abs() < 0.001);
        assert!(right.abs() < 0.001);
        assert!((left - right).abs() < 0.001);
    }

    #[test]
    fn run_has_unconstrained_flight_but_walk_retains_support() {
        for phase in [0.25, 0.75] {
            let (run_left, run_right) = gait_support_weights(
                run_locomotion_profile(),
                CyclePhase::from_cycles(phase).unwrap(),
            )
            .unwrap();
            assert_eq!(
                (run_left, run_right),
                (UnitInterval::ZERO, UnitInterval::ZERO)
            );
            let (walk_left, walk_right) = gait_support_weights(
                walk_locomotion_profile(),
                CyclePhase::from_cycles(phase).unwrap(),
            )
            .unwrap();
            assert!(walk_left.fraction() + walk_right.fraction() > 0.0);
        }

        let phase_step = gait_cycle_phase_delta(
            run_locomotion_profile(),
            NonnegativeSpeed::from_metres_per_second(5.5).unwrap(),
            AnimationElapsed::from_seconds(1.0 / 64.0).unwrap(),
        )
        .unwrap()
        .cycles();
        let mut longest = 0_u32;
        let mut current = 0_u32;
        for frame in 0..=64 {
            let phase = frame as f32 * phase_step;
            let (left, right) = gait_support_weights(
                run_locomotion_profile(),
                CyclePhase::from_cycles(phase).unwrap(),
            )
            .unwrap();
            if left.fraction() <= 0.001 && right.fraction() <= 0.001 {
                current += 1;
                longest = longest.max(current);
            } else {
                current = 0;
            }
        }
        let observed_seconds = longest.saturating_sub(1) as f32 / 64.0;
        assert!((0.085..=0.12).contains(&observed_seconds));
    }

    #[test]
    fn phase_owned_height_has_two_contact_minima_and_two_equal_peaks() {
        for phase in [0.0, 0.5] {
            let state = SkeletonState::default()
                .with_local_velocity(Vec3::NEG_Z * 5.5)
                .with_gait_phase(CyclePhase::from_cycles(phase).unwrap());
            assert!(locomotion_height_wave(&state).unwrap().metres().abs() < 0.0001);
        }
        for phase in [0.25, 0.75] {
            let state = SkeletonState::default()
                .with_local_velocity(Vec3::NEG_Z * 5.5)
                .with_gait_phase(CyclePhase::from_cycles(phase).unwrap());
            assert!(
                (locomotion_height_wave(&state).unwrap().metres()
                    - run_locomotion_profile().flight_apex_metres.metres())
                .abs()
                    < 0.0001
            );
        }
    }

    #[test]
    fn additive_locomotion_height_covers_run_only() {
        let moving = |speed, posture, guard| {
            SkeletonState::default()
                .with_local_velocity(Vec3::NEG_Z * speed)
                .with_body_state(match posture {
                    Posture::Airborne => BodyState::Airborne,
                    _ => BodyState::Grounded(GroundedPosture::Upright),
                })
                .with_weapon_guard(guard)
                .with_raised_locomotion(if guard == WeaponGuardState::Raised {
                    RaisedLocomotionIntent::moving(
                        Vec2::NEG_Y,
                        adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                            speed,
                        )
                        .unwrap(),
                    )
                } else {
                    RaisedLocomotionIntent::default()
                })
        };
        assert_eq!(
            locomotion_height_wave(
                &moving(2.0, Posture::Upright, WeaponGuardState::Lowered,)
                    .with_gait_phase(CyclePhase::from_cycles(0.25).unwrap())
            )
            .unwrap()
            .metres(),
            0.0
        );
        assert!(
            (locomotion_height_wave(
                &moving(5.5, Posture::Upright, WeaponGuardState::Lowered,)
                    .with_gait_phase(CyclePhase::from_cycles(0.25).unwrap())
            )
            .unwrap()
            .metres()
                - run_locomotion_profile().flight_apex_metres.metres())
            .abs()
                < 0.0001
        );
        let raised = moving(2.0, Posture::Upright, WeaponGuardState::Raised)
            .with_gait_phase(CyclePhase::from_cycles(0.25).unwrap());
        assert_eq!(locomotion_height_wave(&raised).unwrap().metres(), 0.0);
        assert_eq!(
            locomotion_height_wave(
                &raised.with_guarded_locomotion_mode(GuardedLocomotionMode::Sprint)
            )
            .unwrap()
            .metres(),
            0.0
        );
        assert_eq!(
            authored_height_compensation(
                &moving(2.0, Posture::Upright, WeaponGuardState::Lowered,)
            )
            .unwrap()
            .metres(),
            0.0
        );
        assert!(
            (authored_height_compensation(&moving(
                5.5,
                Posture::Upright,
                WeaponGuardState::Lowered,
            ))
            .unwrap()
            .metres()
                - procedural_tuning()
                    .authored_ordinary_passing_rise_metres
                    .metres())
            .abs()
                < 0.0001
        );
        assert_eq!(
            authored_height_compensation(&moving(2.0, Posture::Upright, WeaponGuardState::Raised,))
                .unwrap()
                .metres(),
            0.0
        );
        let mut specialized = moving(2.0, Posture::Upright, WeaponGuardState::Lowered);
        specialized.animation_pack = "humanoid_sword_and_shield".into();
        assert_eq!(
            authored_height_compensation(&specialized).unwrap().metres(),
            0.0
        );
    }

    #[test]
    fn body_response_and_landing_compression_are_bounded() {
        let steady_run = body_response_target(
            BodyLocalVelocity::from_metres_per_second(Vec3::Z * 5.5).unwrap(),
            BodyLocalAcceleration::from_metres_per_second_squared(Vec3::ZERO).unwrap(),
            BrakingLeanScale::from_fraction(1.0).unwrap(),
        )
        .unwrap()
        .radians();
        let forward = body_response_target(
            BodyLocalVelocity::from_metres_per_second(Vec3::Z * 5.5).unwrap(),
            BodyLocalAcceleration::from_metres_per_second_squared(Vec3::Z * 12.0).unwrap(),
            BrakingLeanScale::from_fraction(1.0).unwrap(),
        )
        .unwrap()
        .radians();
        let braking = body_response_target(
            BodyLocalVelocity::from_metres_per_second(Vec3::Z * 5.5).unwrap(),
            BodyLocalAcceleration::from_metres_per_second_squared(Vec3::NEG_Z * 12.0).unwrap(),
            BrakingLeanScale::from_fraction(1.0).unwrap(),
        )
        .unwrap()
        .radians();
        let walking_braking = body_response_target(
            BodyLocalVelocity::from_metres_per_second(
                Vec3::Z
                    * walk_locomotion_profile()
                        .reference_speed
                        .metres_per_second(),
            )
            .unwrap(),
            BodyLocalAcceleration::from_metres_per_second_squared(Vec3::NEG_Z * 12.0).unwrap(),
            BrakingLeanScale::from_fraction(
                walk_locomotion_profile()
                    .reference_speed
                    .metres_per_second()
                    / run_locomotion_profile().reference_speed.metres_per_second(),
            )
            .unwrap(),
        )
        .unwrap()
        .radians();
        let stopped_braking = body_response_target(
            BodyLocalVelocity::from_metres_per_second(Vec3::ZERO).unwrap(),
            BodyLocalAcceleration::from_metres_per_second_squared(Vec3::NEG_Z * 12.0).unwrap(),
            BrakingLeanScale::from_fraction(0.0).unwrap(),
        )
        .unwrap()
        .radians();
        let stationary_lateral = body_response_target(
            BodyLocalVelocity::from_metres_per_second(Vec3::ZERO).unwrap(),
            BodyLocalAcceleration::from_metres_per_second_squared(Vec3::X * 12.0).unwrap(),
            BrakingLeanScale::from_fraction(1.0).unwrap(),
        )
        .unwrap()
        .radians();
        let moving_lateral = body_response_target(
            BodyLocalVelocity::from_metres_per_second(Vec3::X * 5.5).unwrap(),
            BodyLocalAcceleration::from_metres_per_second_squared(Vec3::X * 12.0).unwrap(),
            BrakingLeanScale::from_fraction(1.0).unwrap(),
        )
        .unwrap()
        .radians();
        assert!((15.9..=16.1).contains(&steady_run.x.to_degrees()));
        assert!((17.7..=17.9).contains(&forward.x.to_degrees()));
        assert!(forward.x > steady_run.x);
        assert!((1.7..=1.9).contains(&(forward.x - steady_run.x).to_degrees()));
        assert!((-3.0..=-1.0).contains(&braking.x.to_degrees()));
        assert!((-1.0..=0.0).contains(&walking_braking.x.to_degrees()));
        assert!(stopped_braking.x == 0.0);
        assert!((1.9..=2.0).contains(&stationary_lateral.y.abs().to_degrees()));
        assert!((17.3..=17.5).contains(&moving_lateral.y.abs().to_degrees()));
        assert!(
            body_response_target(
                BodyLocalVelocity::from_metres_per_second(Vec3::ZERO).unwrap(),
                BodyLocalAcceleration::from_metres_per_second_squared(Vec3::new(40.0, 0.0, 40.0))
                    .unwrap(),
                BrakingLeanScale::from_fraction(1.0).unwrap()
            )
            .unwrap()
            .radians()
            .length()
            .to_degrees()
                <= 30.0
        );
        assert_eq!(
            landing_compression_for_impact(
                walk_locomotion_profile(),
                LandingImpactSpeed::from_metres_per_second(0.5).unwrap()
            )
            .unwrap()
            .metres(),
            0.0
        );
        assert!(
            (0.04..=0.08).contains(
                &landing_compression_for_impact(
                    walk_locomotion_profile(),
                    LandingImpactSpeed::from_metres_per_second(4.5).unwrap(),
                )
                .unwrap()
                .metres()
            )
        );
        assert!(
            landing_compression_for_action(
                walk_locomotion_profile(),
                LandingImpactSpeed::from_metres_per_second(4.5).unwrap(),
                Some(SkeletonAction::Dodge),
            )
            .unwrap()
            .metres()
                > 0.0
        );
        assert!(
            landing_compression_for_action(
                walk_locomotion_profile(),
                LandingImpactSpeed::from_metres_per_second(4.5).unwrap(),
                None
            )
            .unwrap()
            .metres()
                > 0.0
        );
        assert_eq!(
            presentation_tick_delta(
                Some(AnimationSampleTick::from(10u64)),
                AnimationSampleTick::from(10u64)
            ),
            Some(AnimationSampleSpan::ZERO)
        );
        assert_eq!(
            presentation_tick_delta(
                Some(AnimationSampleTick::from(10u64)),
                AnimationSampleTick::from(14u64)
            ),
            Some(AnimationSampleSpan::from(4u64))
        );
        assert_eq!(
            presentation_tick_delta(
                Some(AnimationSampleTick::from(14u64)),
                AnimationSampleTick::from(2u64)
            ),
            None
        );
        assert_eq!(
            presentation_tick_delta(
                Some(AnimationSampleTick::from(2u64)),
                AnimationSampleTick::from(40u64)
            ),
            None
        );
    }

    #[test]
    fn look_evaluation_is_a_strict_base_plus_offset() {
        let base = Quat::from_rotation_z(0.17);
        let offset = Quat::from_rotation_x(0.2 * 0.16);
        let (applied, state) = additive_look_rotation(base, offset);
        assert!((base * offset).angle_between(applied) <= 0.000_001);
        assert!(state.base_rotation.angle_between(base) <= 0.000_001);
    }

    #[test]
    fn landing_leg_solve_preserves_the_foot_with_real_knee_flexion() {
        let root = Vec3::new(0.0, 1.75, 0.0);
        let knee = Vec3::new(0.0, 0.85, 0.08);
        let compressed_foot = Vec3::new(0.0, -0.05, 0.0);
        let target = compressed_foot + Vec3::Y * 0.05;
        let upper = root.distance(knee);
        let lower = knee.distance(compressed_foot);
        let solution = solve_two_bone(
            TwoBoneChain::new(
                root,
                knee,
                compressed_foot,
                NonnegativeLength::from_metres(upper).unwrap(),
                NonnegativeLength::from_metres(lower).unwrap(),
                Vec3::Z,
            ),
            target,
        )
        .expect("compressed leg should reach its pre-compression foot");
        let flexion = 180.0
            - (root - solution.knee)
                .angle_between(solution.end - solution.knee)
                .to_degrees();
        assert!(solution.end.distance(target) <= 0.0001);
        assert!(flexion >= 10.0);

        let mut retained = None;
        let first = retain_landing_foot_target(&mut retained, target);
        let recovered_frame =
            retain_landing_foot_target(&mut retained, Vec3::new(0.0, -0.02, 0.03));
        assert_eq!(first, target);
        assert_eq!(recovered_frame, first);
        assert!(!landing_plant_is_discontinuous(
            Some(AnimationSampleTick::from(10u64)),
            AnimationSampleTick::from(10u64),
            Some(Vec3::ZERO),
            Vec3::ZERO,
        ));
        assert!(landing_plant_is_discontinuous(
            Some(AnimationSampleTick::from(10u64)),
            AnimationSampleTick::from(9u64),
            Some(Vec3::ZERO),
            Vec3::ZERO,
        ));
        assert!(landing_plant_is_discontinuous(
            Some(AnimationSampleTick::from(10u64)),
            AnimationSampleTick::from(11u64),
            Some(Vec3::ZERO),
            Vec3::X * 3.0,
        ));

        let mut height = LocomotionHeightState {
            landing_left_foot_target: retained,
            landing_right_foot_target: Some(first),
            landing_plant_owner_position: Some(Vec3::ZERO),
            landing_plant_tick: Some(AnimationSampleTick::from(10u64)),
            landing_plant_resync_tick: Some(AnimationSampleTick::from(10u64)),
            ..default()
        };
        clear_landing_foot_plants(&mut height);
        assert!(height.landing_left_foot_target.is_none());
        assert!(height.landing_right_foot_target.is_none());
        assert!(height.landing_plant_owner_position.is_none());
        assert!(height.landing_plant_tick.is_none());
        assert!(height.landing_plant_resync_tick.is_none());
    }

    #[test]
    fn landing_knee_reserve_releases_smoothly_for_late_recovery_reach() {
        let upper = 0.9;
        let lower = 0.9;
        let authored_reach = 1.79;
        let reserved_reach = maximum_reach(
            NonnegativeLength::from_metres(upper).unwrap(),
            NonnegativeLength::from_metres(lower).unwrap(),
        )
        .native_metres()
        .unwrap();

        let peak_reach = landing_maximum_reach(
            NonnegativeLength::from_metres(upper).unwrap(),
            NonnegativeLength::from_metres(lower).unwrap(),
            NonnegativeLength::from_metres(authored_reach).unwrap(),
            HeightDistance::from_metres(0.04).unwrap(),
        )
        .native_metres()
        .unwrap();
        assert!((peak_reach - reserved_reach) == 0.0);
        let peak_flexion = 180.0
            - ((upper * upper + lower * lower - peak_reach * peak_reach) / (2.0 * upper * lower))
                .clamp(-1.0, 1.0)
                .acos()
                .to_degrees();
        assert!(peak_flexion >= 10.0);

        let release_samples = [0.04, 0.033, 0.026, 0.019, 0.012];
        let reaches = release_samples.map(|compression| {
            landing_maximum_reach(
                NonnegativeLength::from_metres(upper).unwrap(),
                NonnegativeLength::from_metres(lower).unwrap(),
                NonnegativeLength::from_metres(authored_reach).unwrap(),
                HeightDistance::from_metres(compression).unwrap(),
            )
            .native_metres()
            .unwrap()
        });
        for pair in reaches.windows(2) {
            assert!(pair[1] >= pair[0]);
            assert!(pair[1] - pair[0] < 0.01);
        }
        assert!((reaches[4] - authored_reach).abs() <= 0.0001);

        let target = Vec3::ZERO;
        let current_knee = Vec3::new(0.0, 0.89, 0.1);
        for compression in [0.012, 0.008, 0.004, 0.001] {
            let root = Vec3::Y * (authored_reach - compression);
            let current_end = Vec3::NEG_Y * compression;
            let solution = solve_landing_two_bone(
                TwoBoneChain::new(
                    root,
                    current_knee,
                    current_end,
                    NonnegativeLength::from_metres(upper).unwrap(),
                    NonnegativeLength::from_metres(lower).unwrap(),
                    Vec3::Z,
                ),
                target,
                HeightDistance::from_metres(compression).unwrap(),
            )
            .unwrap()
            .expect("late landing recovery should remain solvable");
            assert!(solution.end.distance(target) <= 0.0001);
        }
    }

    #[test]
    fn airborne_release_is_bounded_to_fifty_three_millimetres_per_tick() {
        let previous = Vec3::ZERO;
        let desired = Vec3::Z;
        let next = advance_foot_target_at_speed(
            Some(previous),
            desired,
            AnimationElapsed::from_seconds(1.0 / 64.0).unwrap(),
            NonnegativeSpeed::from_metres_per_second(3.4).unwrap(),
        );
        assert!(next.distance(previous) <= 0.053_126);
        assert!(next.distance(desired) < previous.distance(desired));
        assert_eq!(
            advance_foot_target_at_speed(
                None,
                desired,
                AnimationElapsed::from_seconds(1.0 / 64.0).unwrap(),
                NonnegativeSpeed::from_metres_per_second(3.4).unwrap()
            ),
            desired
        );
        assert_eq!(
            advance_foot_target_at_speed(
                Some(previous),
                Vec3::NAN,
                AnimationElapsed::from_seconds(1.0 / 64.0).unwrap(),
                NonnegativeSpeed::from_metres_per_second(3.4).unwrap()
            ),
            previous
        );
    }

    #[test]
    fn rendered_sole_contact_uses_the_shared_hierarchy_tolerance() {
        let terrain_height = SupportElevation::from_metres(2.0).unwrap();
        let exact_ankle = terrain_height.metres() + measured_ankle_sole_offset_metres().metres();
        assert!(sole_is_at_contact(exact_ankle, terrain_height));
        assert!(sole_is_at_contact(
            exact_ankle + sole_contact_tolerance_metres().metres() - 0.00001,
            terrain_height
        ));
        assert!(!sole_is_at_contact(
            exact_ankle + sole_contact_tolerance_metres().metres() + 0.0001,
            terrain_height
        ));
    }

    #[test]
    fn exhausted_support_releases_before_the_plant_can_skate() {
        let plant = Vec3::new(0.1, 0.085, -0.4);
        assert!(!retained_plant_requires_release(
            plant,
            plant + Vec3::X * 0.014
        ));
        assert!(retained_plant_requires_release(
            plant,
            plant + Vec3::Z * 0.016
        ));
    }

    #[test]
    fn first_support_solve_seeds_the_authored_knee_in_the_canonical_hemisphere() {
        let hip = Vec3::ZERO;
        let target = Vec3::NEG_Y * 1.8;
        let authored_knee = Vec3::new(0.0, -0.9, 0.2);
        let seeded = authored_knee_pole_world(hip, authored_knee, target, Vec3::Z)
            .expect("authored bend shares the canonical hemisphere");
        assert!(seeded.dot(Vec3::Z) > 0.99);
        assert!(authored_knee_pole_world(hip, authored_knee, target, Vec3::NEG_Z).is_none());
    }

    #[test]
    fn attacks_keep_live_locomotion_support_while_unsupported_actions_preserve_authored_fk() {
        let mut attack = SkeletonState::default()
            .with_local_velocity(Vec3::NEG_Z * 5.5)
            .with_gait_phase(CyclePhase::from_cycles(0.0).unwrap());
        let locomotion =
            locomotion_support_weights(&attack, attack.animation_speed().unwrap()).unwrap();
        attack
            .begin_attack(
                AttackSpec::default(),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(1u64),
            )
            .unwrap();
        assert_eq!(
            locomotion_support_weights(&attack, attack.animation_speed().unwrap()).unwrap(),
            locomotion
        );

        let mut dodge = SkeletonState::default();
        dodge
            .begin_dodge(
                DodgeSpec::default(),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(1u64),
            )
            .unwrap();
        assert_eq!(
            locomotion_support_weights(&dodge, dodge.animation_speed().unwrap()).unwrap(),
            (UnitInterval::ZERO, UnitInterval::ZERO)
        );
        assert!(!terrain_leg_has_support(
            UnitInterval::from_fraction(0.0).unwrap()
        ));
        assert!(!terrain_leg_has_support(
            UnitInterval::from_fraction(0.01).unwrap()
        ));
        assert!(terrain_leg_has_support(
            UnitInterval::from_fraction(0.1).unwrap()
        ));
    }

    #[test]
    fn terrain_height_is_transient_during_an_active_guard_step() {
        let flat = Vec3::new(0.2, -0.85, 0.3);
        let elevated = terrain_conformed_guard_target(
            flat,
            Some(SupportElevation::from_metres(1.25).unwrap()),
        );
        assert_eq!(elevated.y, 1.335);
        assert_eq!(terrain_conformed_guard_target(flat, None), flat);
        assert_eq!(flat.y, -0.85);
    }

    #[test]
    fn inactive_postures_require_raised_footwork_reset() {
        for body in [BodyState::Airborne, BodyState::Prone] {
            let skeleton = SkeletonState::default()
                .with_body_state(body)
                .with_weapon_guard(WeaponGuardState::Raised);
            assert!(!raised_footwork_posture_is_valid(&skeleton));
        }
    }

    #[test]
    fn cascadeur_foot_bind_axis_keeps_flat_ground_unchanged() {
        // Actual left-foot global bind rotation from assets_src/base.glb.
        let bind = Quat::from_xyzw(0.8856122, 0.00000032, 0.00000032, 0.46442544).normalize();
        let sole_up = sole_up_axis_from_bind(bind).normalize();
        assert!(sole_up.y < -0.5 && sole_up.z < -0.8);
        assert!((bind * sole_up).abs_diff_eq(Vec3::Y, 0.0001));
        assert!((bind * Vec3::Y).dot(Vec3::Y) < 0.0);
    }

    #[test]
    fn slope_alignment_is_idempotent_across_repeated_evaluation() {
        let bind = Quat::from_xyzw(0.8856122, 0.00000032, 0.00000032, 0.46442544).normalize();
        let sole_up = sole_up_axis_from_bind(bind).normalize();
        let current = Quat::from_rotation_y(0.7) * bind;
        let steep_normal = bevy::math::Dir3::new(Vec3::new(0.8, 0.3, -0.4)).unwrap();

        let once = slope_aligned_world_rotation(current, sole_up, steep_normal).unwrap();
        let twice = slope_aligned_world_rotation(once, sole_up, steep_normal).unwrap();

        assert!(once.angle_between(twice).to_degrees() < 0.0001);
        assert!((once * sole_up).angle_between(Vec3::Y).to_degrees() <= 28.0001);
    }

    #[test]
    fn terrain_ik_accepts_grounded_upright_but_not_airborne() {
        let upright = SkeletonState::default();
        assert!(terrain_ik_posture_is_valid(&upright));

        let airborne = SkeletonState::default().with_body_state(BodyState::Airborne);
        assert!(!terrain_ik_posture_is_valid(&airborne));
    }

    #[test]
    fn remembered_pole_follows_owner_yaw() {
        let original_yaw = Quat::from_rotation_y(0.3);
        let owner_local = Vec3::new(0.2, -0.1, -0.97).normalize();
        let saved = pole_to_owner(original_yaw, pole_to_world(original_yaw, owner_local));
        assert!(saved.abs_diff_eq(owner_local, 0.0001));
        let new_yaw = Quat::from_rotation_y(1.4);
        let expected = new_yaw * owner_local;
        assert!(pole_to_world(new_yaw, saved).abs_diff_eq(expected, 0.0001));
    }

    #[test]
    fn pose_mirror_is_an_involution() {
        let original = Transform::from_xyz(0.3, -0.8, 0.2).with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.2,
            -0.3,
            0.4,
        ));
        let twice = mirrored_across_anatomical_center(mirrored_across_anatomical_center(original));
        assert!(twice.translation.abs_diff_eq(original.translation, 0.0001));
        assert!(twice.rotation.abs_diff_eq(original.rotation, 0.0001));
    }

    #[test]
    fn foot_tracks_prevent_crossing_and_keep_minimum_separation() {
        let rotation = Quat::from_rotation_y(0.6);
        let origin = Vec3::new(2.0, 0.0, -3.0);
        let left = constrain_foot_to_track(origin, origin, rotation, FootTrackSide::NegativeX);
        let right = constrain_foot_to_track(origin, origin, rotation, FootTrackSide::PositiveX);
        let left_local = rotation.inverse() * (left - origin);
        let right_local = rotation.inverse() * (right - origin);
        assert!(left_local.x < 0.0 && right_local.x > 0.0);
        assert!(
            right_local.x - left_local.x
                >= minimum_inter_foot_separation_metres().metres() - 0.0001
        );
    }

    #[test]
    fn plant_releases_only_on_a_real_discontinuity_and_reach_slides_continuously() {
        let foot = Vec3::new(-0.2, -1.8, 0.0);
        assert!(plant_is_continuous(foot, foot));
        assert!(!plant_is_continuous(Vec3::NAN, foot));
        assert!(!plant_is_continuous(foot + Vec3::X * 2.1, foot));

        let root = Vec3::Y;
        let target = Vec3::new(-0.2, 0.0, 3.0);
        let constrained = constrain_target_to_reach(
            target,
            root,
            ReachLimit::from(NonnegativeLength::from_metres(1.5).unwrap()),
        )
        .unwrap();
        assert!(constrained.distance(root) <= 1.5001);
        assert_eq!(constrained.y, target.y);
    }

    #[test]
    fn pelvis_smoothing_depends_on_elapsed_time_not_evaluation_count() {
        fn simulate(step_seconds: f32, evaluations: usize) -> f32 {
            (0..evaluations).fold(0.0, |current, _| {
                advance_pelvis_shift(
                    SignedLength::from_metres(current).unwrap(),
                    SignedLength::from_metres(-1.0).unwrap(),
                    AnimationElapsed::from_seconds(step_seconds).unwrap(),
                )
                .unwrap()
                .metres()
            })
        }

        let at_64_hz = simulate(1.0 / 64.0, 16);
        let at_128_hz = simulate(1.0 / 128.0, 32);
        assert!((at_64_hz - at_128_hz).abs() < 0.0001);
        assert!((at_64_hz + 0.3).abs() < 0.0001);

        let after_hitch = advance_pelvis_shift(
            SignedLength::from_metres(0.0).unwrap(),
            SignedLength::from_metres(-1.0).unwrap(),
            AnimationElapsed::from_seconds(1.0).unwrap(),
        )
        .unwrap()
        .metres();
        assert!((after_hitch + maximum_pelvis_correction_step_metres().metres()).abs() < 0.0001);
    }

    #[test]
    fn standing_knee_yaw_constraint_preserves_downed_and_transition_poses() {
        assert!(ik::anatomical_knee_yaw_posture_is_valid(
            &SkeletonState::default()
        ));
        assert!(!ik::anatomical_knee_yaw_posture_is_valid(
            &SkeletonState::default().with_body_state(BodyState::Prone)
        ));
        let mut transition = SkeletonState::default();
        assert!(transition.begin_posture_transition(
            PostureTransitionKind::UprightToProne,
            adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
            adventuresim_tactical_core::animation::AnimationSampleSpan::from(10u64),
        ));
        assert!(!ik::anatomical_knee_yaw_posture_is_valid(&transition));
        let mut quickstep = SkeletonState::default();
        quickstep
            .begin_dodge(
                DodgeSpec::quickstep(Vec2::X).unwrap(),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(0u64),
                adventuresim_tactical_core::animation::AnimationSampleTick::from(100u64),
            )
            .unwrap();
        quickstep
            .advance_action(
                adventuresim_tactical_core::animation::AnimationSampleTick::from(150u64),
            )
            .unwrap();
        assert!(!ik::anatomical_knee_yaw_posture_is_valid(&quickstep));
    }

    #[test]
    fn anatomical_side_does_not_collapse_during_mirror_blend() {
        let left = Transform::from_xyz(0.18, -0.1, 0.25);
        let right = Transform::from_xyz(-0.18, -0.1, -0.25);
        let mirrored_right = mirrored_across_anatomical_center(right);
        let midpoint_x = left.translation.lerp(mirrored_right.translation, 0.5).x;
        assert!((midpoint_x - 0.18).abs() < 0.0001);
    }

    #[test]
    fn global_mirror_uses_character_space_under_owner_rotation() {
        let owner = GlobalTransform::from(
            Transform::from_xyz(4.0, 2.0, -3.0).with_rotation(Quat::from_rotation_y(1.1)),
        );
        let relative = Transform::from_xyz(0.4, -0.7, 0.2).with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.2,
            -0.3,
            0.4,
        ));
        let source = owner.mul_transform(relative);
        let mirrored = owner.affine().inverse() * mirrored_global_affine(source, owner);
        let (scale, rotation, translation) = mirrored.to_scale_rotation_translation();
        let expected = mirrored_across_anatomical_center(relative);

        assert!(translation.abs_diff_eq(expected.translation, 0.0001));
        assert!(rotation.abs_diff_eq(expected.rotation, 0.0001));
        assert!(scale.abs_diff_eq(expected.scale, 0.0001));
    }
}
