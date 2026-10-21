//! Fixed-rate local-pose sampling and interruption-safe inertialization.
//!
//! This is deliberately downstream of semantic animation-pack resolution:
//! the semantic router produces an `AnimationPlayback` plan from the shared
//! `PresentedSkeleton`. This backend samples sparse locomotion anchors with
//! Bevy-side interpolation; other motions use clip curves baked onto a 30 Hz
//! grid. Both paths write per-character local-pose buffers before the shared
//! procedural passes.

use super::procedural::clock::{AnimationSampleTick, FixedAnimationStep};
use super::*;
use adventuresim_tactical_core::scene_coordinates::ScenePlanPoint;
use baking::{BakedClip, BoneTrackSource, bake_clip, get_or_bake};
#[cfg(test)]
use baking::{BakedFrameCount, BoneTrack};
use bevy::{
    animation::{AnimationTargetId, animated_field},
    asset::AssetId,
    camera::visibility::RenderLayers,
};
pub(in crate::animation) use calibration_inputs::{CalibrationInputs, CalibrationLookup};
use contact_calibration::{
    AuthoredLocomotionCalibration, measure_authored_contact_step_distance,
    measure_authored_foot_range,
};
use display_history::PosePresentationMode;
use fabelgeist_rig::{RigJointName, RigJointOrdinal, RigJointTable};
use hierarchy::{
    aim_pose_joint, hand_animation_joint, local_pose_global, sampled_global_transform,
};
use inertialization::{
    JointInertialOffset, hemisphere_slerp, local_pose_velocity, quaternion_exp, quaternion_log,
    shortest_rotation,
};
use interpolation::PoseInterpolation;
use locomotion_sampling::sample_locomotion_at_time;
pub(in crate::animation) use phase_calibration::AuthoredPhaseCurve;
use proportions::sample_character_plan;
use rig::{RigDefinition, RigJoint};
use spline::clamped_cubic_spline_vec3;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use stride_calibration::CalibrationAxis;
pub(super) use stride_calibration::{CharacterLocomotionStrides, calibrate_character_strides};

mod baking;
mod calibration_inputs;
mod contact_calibration;
mod display_history;
mod hierarchy;
mod inertialization;
mod interpolation;
mod locomotion_sampling;
mod phase_calibration;
mod physics_pose;
mod proportions;
mod rig;
mod sampling_clock;
mod spline;
mod stride_calibration;
mod visibility;

#[derive(Resource, Default, Debug, Clone, Copy, serde::Serialize)]
pub(crate) struct PoseBufferMetrics {
    pub(crate) baked_clip_bytes: usize,
    pub(crate) baked_clip_count: usize,
    pub(crate) sampled_pose_count: u64,
    pub(crate) culled_character_count: usize,
}

/// Rig-local semantic target before inertialization, secondary physics, IK,
/// and hierarchical render transforms modify the weapon-driving chain.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct AuthoredWeaponTarget {
    pub(crate) translation: Vec3,
    pub(crate) rotation: Quat,
}

#[derive(Resource, Default)]
pub(super) struct RigDefinitions(HashMap<SkeletonFamily, Arc<RigDefinition>>);

#[derive(Resource, Default)]
pub(super) struct BakedClipBank(HashMap<(SkeletonFamily, AssetId<AnimationClip>), Arc<BakedClip>>);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::animation) struct LocalPose {
    pub(in crate::animation) translation: Vec3,
    pub(in crate::animation) rotation: Quat,
    pub(in crate::animation) scale: Vec3,
}

#[derive(Debug, Clone, Copy)]
struct ContinuationTransition {
    contact: LocalPose,
    ready: LocalPose,
    outgoing: LocalPose,
    finish: LocalPose,
    start_coordinate: AttackPoseCoordinate,
    incoming_tangent: IncomingAttackTangent,
    ready_phase: ContinuationReadiness,
}

#[derive(Debug)]
struct SampledPlan {
    pose: RigJointTable<LocalPose>,
    /// Pelvis rotation sampled from combat locomotion before the attack/guard
    /// rotation is applied. This is the FK reference for compensating leg IK.
    locomotion_pelvis_rotation: Option<Quat>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PosePlanKey {
    clips: Vec<(AssetId<AnimationClip>, ClipLayer)>,
}

#[derive(Component)]
pub(crate) struct PoseBufferRig {
    definition: Arc<RigDefinition>,
    entities: RigJointTable<Option<Entity>>,
    previous: RigJointTable<LocalPose>,
    next: RigJointTable<LocalPose>,
    sample_accumulator: AnimationElapsed,
    interpolation_alpha: PoseInterpolation,
    last_evaluation_tick: Option<AnimationSampleTick>,
    decay_delta_seconds: AnimationElapsed,
    offsets: RigJointTable<JointInertialOffset>,
    target_linear_velocities: RigJointTable<Vec3>,
    target_angular_velocities: RigJointTable<Vec3>,
    terrain_plants: [Option<AuthoredContactPlant>; 2],
    plan: Option<PosePlanKey>,
    presentation: PosePresentationMode,
    visibility: PoseVisibility,
    /// Every inertial offset has decayed to zero: the decay is skipped until
    /// the next transition captures a new one.
    inertial_state: InertialState,
    /// A physics ragdoll writes this buffer; authored sampling, transitions,
    /// and LOD freezing stand down until it hands the pose back.
    ownership: PoseOwnership,
}

#[derive(Clone, Copy, Debug)]
struct AuthoredContactPlant {
    position_world: Vec3,
    rotation_world: Quat,
    /// The stance anchor at acquisition, expressed in owner space. Advancing
    /// authored clip phase must not consume the plant's reach allowance; only
    /// gameplay translation/turning of the owner moves this reference.
    reference_owner_position: Vec3,
    reference_owner_rotation: Quat,
}

#[derive(Debug, Clone, Copy)]
struct PoseConformity<'a> {
    owner: &'a GlobalTransform,
    weights: FootIkInfluence,
    terrain: Option<&'a SceneTerrain>,
    contact_plants: ContactPlantPolicy,
    locomotion_pelvis_rotation: Option<Quat>,
}

/// Solve the terrain-adjusted upcoming sample in pose-buffer space. The
/// renderer subsequently interpolates `previous -> next`, so terrain
/// conformity follows exactly the same continuous local-pose interpolation as
/// authored FK instead of being switched onto the already displayed frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContactPlantPolicy {
    Reset,
    Retain,
}

impl AuthoredWeaponTarget {
    fn from_pose(definition: &RigDefinition, pose: &RigJointTable<LocalPose>) -> Option<Self> {
        let weapon_joint = definition
            .joints
            .iter()
            .position(|joint| joint.role == Some(BoneRole::WeaponRight))
            .map(RigJointOrdinal::from)?;
        let mut cache = RigJointTable::from(vec![None; pose.len()]);
        let weapon = local_pose_global(definition, pose, weapon_joint, &mut cache);
        Some(Self {
            translation: weapon.translation,
            rotation: weapon.rotation,
        })
    }
}

impl ContinuationTransition {
    fn from_span(
        contact: LocalPose,
        ready: LocalPose,
        outgoing: LocalPose,
        finish: LocalPose,
        span: &ContinuationSpan,
    ) -> Self {
        Self {
            contact,
            ready,
            outgoing,
            finish,
            start_coordinate: span.start_coordinate,
            incoming_tangent: span.incoming_tangent,
            ready_phase: span.ready_phase,
        }
    }
}

impl LocalPose {
    fn from_transform(transform: Transform) -> Self {
        Self {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }

    fn interpolate(self, next: Self, alpha: UnitInterval) -> Self {
        let alpha = alpha.fraction().clamp(0.0, 1.0);
        Self {
            translation: self.translation.lerp(next.translation, alpha),
            rotation: hemisphere_slerp(self.rotation, next.rotation, alpha),
            scale: self.scale.lerp(next.scale, alpha),
        }
    }

    fn extrapolate(self, next: Self, coordinate: AttackPoseCoordinate) -> Self {
        let coordinate = coordinate.clamped_to_curve().coordinate();
        self.extrapolate_unbounded(next, coordinate)
    }

    fn extrapolate_unbounded(self, next: Self, coordinate: f32) -> Self {
        let relative = shortest_rotation(next.rotation * self.rotation.inverse());
        Self {
            translation: self.translation + (next.translation - self.translation) * coordinate,
            rotation: (quaternion_exp(quaternion_log(relative) * coordinate) * self.rotation)
                .normalize(),
            scale: self.scale + (next.scale - self.scale) * coordinate,
        }
    }

    fn continuation_transition(self, transition: ContinuationTransition, progress: UnitInterval) -> Self {
        let start = self.extrapolate(transition.contact, transition.start_coordinate);
        // The complete native spline consumes phase knots, normalized progress
        // and an attack-coordinate derivative; checked roles end at this kernel.
        let progress = progress.fraction().clamp(0.0, 1.0);
        let ready_phase = transition.ready_phase.phase();
        let knots = [0.0, ready_phase, 0.5, 1.0];
        let incoming_scale = transition.incoming_tangent.coordinate_delta() / ready_phase;
        let rotation_values = [
            Vec3::ZERO,
            quaternion_log(shortest_rotation(
                transition.ready.rotation * start.rotation.inverse(),
            )),
            quaternion_log(shortest_rotation(
                transition.outgoing.rotation * start.rotation.inverse(),
            )),
            quaternion_log(shortest_rotation(
                transition.finish.rotation * start.rotation.inverse(),
            )),
        ];
        Self {
            translation: clamped_cubic_spline_vec3(
                knots,
                [
                    start.translation,
                    transition.ready.translation,
                    transition.outgoing.translation,
                    transition.finish.translation,
                ],
                (transition.contact.translation - self.translation) * incoming_scale,
                Vec3::ZERO,
                progress,
            ),
            rotation: (quaternion_exp(clamped_cubic_spline_vec3(
                knots,
                rotation_values,
                quaternion_log(shortest_rotation(
                    transition.contact.rotation * self.rotation.inverse(),
                )) * incoming_scale,
                Vec3::ZERO,
                progress,
            )) * start.rotation)
                .normalize(),
            scale: clamped_cubic_spline_vec3(
                knots,
                [
                    start.scale,
                    transition.ready.scale,
                    transition.outgoing.scale,
                    transition.finish.scale,
                ],
                (transition.contact.scale - self.scale) * incoming_scale,
                Vec3::ZERO,
                progress,
            ),
        }
    }
}

impl PosePlanKey {
    fn from_playback(playback: &AnimationPlayback) -> Self {
        let mut clips = playback
            .plan
            .clips()
            .iter()
            .map(|weighted| (weighted.clip.handle.id(), weighted.clip.layer))
            .collect::<Vec<_>>();
        clips.extend(playback.plan.extrapolated().iter().flat_map(|span| {
            [
                (span.start.handle.id(), span.start.layer),
                (span.end.handle.id(), span.end.layer),
            ]
        }));
        clips.extend(playback.plan.continuations().iter().flat_map(|span| {
            [
                (span.start.handle.id(), span.start.layer),
                (span.contact.handle.id(), span.contact.layer),
                (span.end.handle.id(), span.end.layer),
                (span.outgoing.handle.id(), span.outgoing.layer),
                (span.finish.handle.id(), span.finish.layer),
            ]
        }));
        clips.sort_by_key(|(id, layer)| (*id, *layer as u8));
        clips.dedup();
        Self { clips }
    }
}

impl BakedClipBank {
    pub(super) fn invalidate_assets(&mut self, changed: &HashSet<AssetId<AnimationClip>>) {
        self.0.retain(|(_, id), _| !changed.contains(id));
    }
}

impl PoseBufferRig {
    pub(super) fn invalidate_clip_revisions(&mut self, changed: &HashSet<AssetId<AnimationClip>>) {
        if self
            .plan
            .as_ref()
            .is_some_and(|plan| plan.clips.iter().any(|(id, _)| changed.contains(id)))
        {
            self.plan = None;
        }
    }

    fn set_target_velocities(&mut self, velocities: &RigJointTable<(Vec3, Vec3)>) {
        for (joint, &(linear, angular)) in velocities.indexed() {
            self.target_linear_velocities[joint] = linear;
            self.target_angular_velocities[joint] = angular;
        }
    }

    fn displayed_velocity(&self, joint: RigJointOrdinal) -> (Vec3, Vec3) {
        (
            self.target_linear_velocities[joint] + self.offsets[joint].translation_velocity(),
            self.target_angular_velocities[joint] + self.offsets[joint].angular_velocity(),
        )
    }

    /// Start an inertialized blend from the pose this rig is displaying onto
    /// `target`, the first pose of a new plan. An inactive rig has nothing
    /// displayed and snaps straight to the target.
    fn capture_transition_offsets(
        &mut self,
        target: &RigJointTable<LocalPose>,
        target_velocities: &RigJointTable<(Vec3, Vec3)>,
    ) -> AnimationValueResult<()> {
        let capture_displayed = self.has_presented_pose();
        for (joint, &target_pose) in target.indexed() {
            let displayed = if capture_displayed {
                // Animation evaluation may already have written the new
                // plan into live transforms this frame. The pose buffer is
                // the authoritative previous authored output; procedural
                // lean, secondary motion, and IK retain their own state and
                // are reapplied after this pass.
                self.displayed_pose(joint)
            } else {
                target_pose
            };
            let displayed_velocity = if capture_displayed {
                self.displayed_velocity(joint)
            } else {
                (Vec3::ZERO, Vec3::ZERO)
            };
            self.offsets[joint].capture(
                displayed,
                displayed_velocity,
                target_pose,
                target_velocities[joint],
                pose_tuning().inertial_blend_seconds,
            )?;
        }
        self.inertial_state = InertialState::Blending;
        Ok(())
    }
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "Bevy injects each independently borrowed animation resource and query as a system parameter"
)]
pub(super) fn update_pose_buffers(
    mut commands: Commands,
    time: Res<Time>,
    procedural_clock: Res<ProceduralAnimationClock>,
    catalog: Res<AnimationPackCatalog>,
    clips: Res<Assets<AnimationClip>>,
    cameras: visibility::AnimationViews,
    terrain: Query<&SceneTerrain>,
    terrain_ik_enabled: Res<TerrainIkMode>,
    owners: Query<(
        Entity,
        &PresentedSkeleton,
        &AnimationPlayback,
        &GlobalTransform,
        Option<&mut PoseBufferRig>,
        Option<&RenderLayers>,
    )>,
    rig_scenes: Query<(&AnimationRigScene, &Transform)>,
    targets: Query<(
        Entity,
        &AnimationTargetId,
        &AuthoredBindTransform,
        Option<&AuthoredRigJoint>,
        Option<&ChildOf>,
    )>,
    transforms: Query<&Transform>,
    proportion_offsets: Query<&skeletal_proportions::SkeletalJointOffset>,
    mut definitions: ResMut<RigDefinitions>,
    mut bank: ResMut<BakedClipBank>,
    mut metrics: ResMut<PoseBufferMetrics>,
) {
    let _spike = crate::animation::diagnostics::SpikeGuard::new("update_pose_buffers");
    metrics.culled_character_count = 0;

    for (owner, skeleton, playback, owner_transform, rig, layers) in owners {
        let family = catalog
            .packs
            .get(&skeleton.animation_pack)
            .map(|pack| pack.skeleton_family.clone())
            .unwrap_or(SkeletonFamily::HUMANOID);
        let mut rig = if let Some(rig) = rig {
            rig
        } else {
            let Some(pose_rig) = definitions.bind(owner, family, &targets, &transforms) else {
                continue;
            };
            commands.entity(owner).insert(pose_rig);
            continue;
        };

        let delta_seconds = match rig.begin_observation(&procedural_clock, &time) {
            Ok(Some(elapsed)) => elapsed,
            Ok(None) => continue,
            Err(error) => {
                warn!(%error, "Rejected pose-buffer interval");
                continue;
            }
        };

        if rig.ownership == PoseOwnership::Physics {
            // The ragdoll readback owns `previous`/`next`; a culled or
            // resampled buffer here would drag driven bones off their
            // bodies.
            rig.visibility = PoseVisibility::Visible;
            continue;
        }

        let position = owner_transform.translation();
        // Initialize retained models before culling so first selection has a pose.
        let frozen =
            rig.has_presented_pose() && visibility::outside_views(position, layers, &cameras);
        if frozen {
            rig.visibility = PoseVisibility::Frozen;
            metrics.culled_character_count += 1;
            continue;
        }
        rig.resume_visible_sampling();

        let key = PosePlanKey::from_playback(playback);
        let transition = !rig.has_presented_pose() || rig.plan.as_ref() != Some(&key);
        rig.sample_accumulator = match rig.sample_accumulator.checked_add(delta_seconds) {
            Ok(elapsed) => elapsed,
            Err(error) => {
                warn!(%error, "Rejected pose-buffer accumulator");
                continue;
            }
        };
        let Some(mut sampled) = sample_character_plan(
            playback,
            &rig,
            &clips,
            &mut bank,
            &mut metrics,
            &proportion_offsets,
        ) else {
            continue;
        };
        let locomotion_ik_owns = procedural::authored_locomotion_ik_owns(skeleton);
        let terrain = (terrain_ik_enabled.is_enabled() && locomotion_ik_owns)
            .then(|| terrain.single().ok())
            .flatten();
        if terrain.is_some() || sampled.locomotion_pelvis_rotation.is_some() {
            let definition = Arc::clone(&rig.definition);
            // Buffered joint poses are relative to the authored scene, not to
            // the controller entity. The scene is translated from the
            // capsule's centre to the character's visual ground origin.
            // Omitting that transform makes every sampled ankle appear about
            // one capsule half-height too high and disables terrain contact.
            let presentation_transform = rig_scenes
                .iter()
                .find_map(|(scene, local)| {
                    (scene.0 == owner).then(|| presentation_world_transform(owner_transform, local))
                })
                .unwrap_or(*owner_transform);
            conform_upcoming_pose_to_terrain(
                &definition,
                &mut sampled.pose,
                &mut rig.terrain_plants,
                PoseConformity {
                    owner: &presentation_transform,
                    weights: playback.foot_ik_weights,
                    terrain,
                    contact_plants: ContactPlantPolicy::Reset,
                    locomotion_pelvis_rotation: sampled.locomotion_pelvis_rotation,
                },
            );
            // Translating authored cycles own their complete XZ trajectories.
            // Stationary guard turning uses the separate procedural pole-limit
            // plant path; never carry a world-space plant through this terrain
            // conformity pass.
        } else {
            rig.terrain_plants = [None; 2];
        }
        let target = sampled.pose;
        if let Some(weapon_target) = AuthoredWeaponTarget::from_pose(&rig.definition, &target) {
            commands.entity(owner).insert(weapon_target);
        }
        let direct_sampling = playback.sampling_cadence() == PoseSamplingCadence::Direct;
        let target_velocities = target
            .iter()
            .zip(&rig.next)
            .map(|(target, previous)| {
                if rig.has_presented_pose() && direct_sampling && !transition {
                    local_pose_velocity(*previous, *target, delta_seconds)
                } else {
                    // Poses on opposite sides of a plan change do not form a
                    // trajectory. Differentiating them invents an angular
                    // velocity that inertialization would then preserve.
                    (Vec3::ZERO, Vec3::ZERO)
                }
            })
            .collect::<RigJointTable<_>>();
        if transition {
            if let Err(error) = rig.capture_transition_offsets(&target, &target_velocities) {
                warn!(%error, "Rejected pose inertialization duration");
                continue;
            }
            rig.set_target_velocities(&target_velocities);
            rig.previous.clone_from(&target);
            rig.next = target;
            rig.sample_accumulator = AnimationElapsed::ZERO;
            rig.interpolation_alpha = PoseInterpolation::ONE;
            rig.plan = Some(key);
            rig.presentation = PosePresentationMode::Displaying;
        } else {
            if direct_sampling {
                // The semantic curve supplies the continuous target. Preserve
                // transition inertialization, but do not quantize its normal
                // updates through the authored-clip sampling accumulator.
                rig.previous.clone_from(&target);
                rig.next = target;
                rig.sample_accumulator = AnimationElapsed::ZERO;
                rig.interpolation_alpha = PoseInterpolation::ONE;
                rig.set_target_velocities(&target_velocities);
            } else {
                // Terrain IK already owns `next`; interpolate the two solved
                // samples without introducing another contact transition.
                if let Err(error) = rig.advance_buffered_sample(target, pose_sample_interval()) {
                    warn!(%error, "Rejected pose-buffer sample clock");
                    continue;
                }
            }
        }
    }
}

pub(super) fn apply_pose_buffers(
    mut rigs: Query<&mut PoseBufferRig>,
    mut transforms: Query<&mut Transform, Without<PoseBufferRig>>,
) {
    let _spike = crate::animation::diagnostics::SpikeGuard::new("apply_pose_buffers");
    for mut rig in &mut rigs {
        if !rig.has_presented_pose() || rig.visibility == PoseVisibility::Frozen {
            continue;
        }
        let alpha = rig.interpolation_alpha.amount();
        let delta_seconds = rig.decay_delta_seconds;
        let rig = &mut *rig;
        for joint in rig.entities.ordinals() {
            let Some(entity) = rig.entities[joint] else {
                continue;
            };
            let input = rig.previous[joint].interpolate(rig.next[joint], alpha);
            let pose = if rig.inertial_state == InertialState::Settled {
                input
            } else if !delta_seconds.is_zero() {
                match rig.offsets[joint].update(input, delta_seconds) {
                    Ok(pose) => pose,
                    Err(error) => {
                        warn!(%error, "Rejected pose inertialization clock");
                        continue;
                    }
                }
            } else {
                rig.offsets[joint].peek(input)
            };
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.translation = pose.translation;
                transform.rotation = pose.rotation;
                transform.scale = pose.scale;
            }
        }
        // Once every quintic has run its full length the offsets are exactly
        // zero; clear them so at-rest rigs skip the decay entirely.
        if rig.inertial_state != InertialState::Settled
            && rig.offsets.iter().all(JointInertialOffset::done)
        {
            rig.inertial_state = InertialState::Settled;
            rig.offsets.fill(JointInertialOffset::default());
        }
    }
}

pub(super) fn calibrate_authored_locomotion_strides(
    definitions: Res<RigDefinitions>,
    runtime: Res<AnimationRuntime>,
    clips: Res<Assets<AnimationClip>>,
    mut strides: ResMut<AuthoredLocomotionStrides>,
) {
    let Some(definition) = definitions.0.get(&SkeletonFamily::HUMANOID) else {
        return;
    };
    if stride_calibration::has_unmeasured_motion(&runtime, &clips, &strides) {
        stride_calibration::calibrate(&runtime, &clips, definition, &mut strides, &[]);
    }
}

fn pose_tuning() -> PoseBufferConfig {
    runtime_animation_config().pose_buffer
}

fn pose_sample_interval() -> adventuresim_tactical_core::animation::PositiveAnimationDuration {
    pose_tuning().sample_hz.interval()
}

fn presentation_world_transform(
    owner: &GlobalTransform,
    rig_scene_local: &Transform,
) -> GlobalTransform {
    owner.mul_transform(*rig_scene_local)
}

fn sample_plan(
    playback: &AnimationPlayback,
    definition: &RigDefinition,
    clips: &Assets<AnimationClip>,
    bank: &mut BakedClipBank,
    metrics: &mut PoseBufferMetrics,
) -> Option<SampledPlan> {
    if playback.plan.is_bind() {
        return Some(SampledPlan {
            pose: definition.joints.iter().map(|joint| joint.bind).collect(),
            locomotion_pelvis_rotation: None,
        });
    }
    let mut baked = Vec::with_capacity(playback.plan.clips().len());
    for weighted in playback.plan.clips() {
        let clip = get_or_bake(
            weighted.clip.handle.id(),
            &weighted.clip.handle,
            definition,
            clips,
            bank,
            metrics,
        )?;
        baked.push((weighted, clip, clips.get(&weighted.clip.handle)?));
    }
    let mut baked_spans = Vec::with_capacity(playback.plan.extrapolated().len());
    for span in playback.plan.extrapolated() {
        let start = get_or_bake(
            span.start.handle.id(),
            &span.start.handle,
            definition,
            clips,
            bank,
            metrics,
        )?;
        let end = get_or_bake(
            span.end.handle.id(),
            &span.end.handle,
            definition,
            clips,
            bank,
            metrics,
        )?;
        baked_spans.push((span, start, end));
    }
    let mut baked_continuations = Vec::with_capacity(playback.plan.continuations().len());
    for span in playback.plan.continuations() {
        let start = get_or_bake(
            span.start.handle.id(),
            &span.start.handle,
            definition,
            clips,
            bank,
            metrics,
        )?;
        let contact = get_or_bake(
            span.contact.handle.id(),
            &span.contact.handle,
            definition,
            clips,
            bank,
            metrics,
        )?;
        let end = get_or_bake(
            span.end.handle.id(),
            &span.end.handle,
            definition,
            clips,
            bank,
            metrics,
        )?;
        let outgoing = get_or_bake(
            span.outgoing.handle.id(),
            &span.outgoing.handle,
            definition,
            clips,
            bank,
            metrics,
        )?;
        let finish = get_or_bake(
            span.finish.handle.id(),
            &span.finish.handle,
            definition,
            clips,
            bank,
            metrics,
        )?;
        baked_continuations.push((span, start, contact, end, outgoing, finish));
    }
    let mut pose = RigJointTable::default();
    let mut locomotion_pelvis_rotation = None;
    for (joint_index, joint) in definition.joints.indexed() {
        let mut blended = joint.bind;
        let mut accumulated = BlendWeight::ZERO;
        let is_pelvis = joint.role == Some(BoneRole::Pelvis);
        let mut combat_upper = joint.bind;
        let mut combat_upper_weight = BlendWeight::ZERO;
        let mut combat_lower = joint.bind;
        let mut combat_lower_weight = BlendWeight::ZERO;
        for (weighted, clip, source) in &baked {
            let included = joint.body_region.includes(weighted.clip.layer, joint.role);
            if !included || weighted.weight.is_absent() {
                continue;
            }
            let sample = if weighted.locomotion_phase.is_some() {
                sample_locomotion_at_time(
                    source,
                    joint,
                    weighted.clip.duration_seconds,
                    weighted.time_seconds,
                )
                .ok()?
            } else {
                clip.sample(joint_index, weighted.time_seconds)
            };
            let sample = sanitize_pose(sample, joint.bind);
            if is_pelvis && weighted.clip.layer == ClipLayer::CombatUpper {
                accumulate_local_pose(
                    &mut combat_upper,
                    &mut combat_upper_weight,
                    sample,
                    weighted.weight,
                )
                .ok()?;
                continue;
            }
            if is_pelvis && weighted.clip.layer == ClipLayer::CombatLower {
                accumulate_local_pose(
                    &mut combat_lower,
                    &mut combat_lower_weight,
                    sample,
                    weighted.weight,
                )
                .ok()?;
                continue;
            }
            accumulate_local_pose(&mut blended, &mut accumulated, sample, weighted.weight).ok()?;
        }
        for (span, start_clip, end_clip) in &baked_spans {
            let included = joint.body_region.includes(span.start.layer, joint.role);
            if !included || span.weight.is_absent() {
                continue;
            }
            let start = sanitize_pose(
                start_clip.sample(joint_index, span.start_time_seconds),
                joint.bind,
            );
            let end = sanitize_pose(
                end_clip.sample(joint_index, span.end_time_seconds),
                joint.bind,
            );
            // CurveSpan is defined by its two semantic anchors, not the keys
            // authored between or after them. Applying their transform delta
            // directly gives one constant spatial path through coordinate 1,
            // so contact cannot introduce a hidden ease-out/ease-in stop.
            let sample = start.extrapolate(end, span.coordinate);
            let sample = sanitize_pose(sample, joint.bind);
            if is_pelvis && span.start.layer == ClipLayer::CombatUpper {
                accumulate_local_pose(
                    &mut combat_upper,
                    &mut combat_upper_weight,
                    sample,
                    span.weight,
                )
                .ok()?;
                continue;
            }
            if is_pelvis && span.start.layer == ClipLayer::CombatLower {
                accumulate_local_pose(
                    &mut combat_lower,
                    &mut combat_lower_weight,
                    sample,
                    span.weight,
                )
                .ok()?;
                continue;
            }
            accumulate_local_pose(&mut blended, &mut accumulated, sample, span.weight).ok()?;
        }
        for (span, start_clip, contact_clip, end_clip, outgoing_clip, finish_clip) in
            &baked_continuations
        {
            let included = joint.body_region.includes(span.start.layer, joint.role);
            if !included || span.weight.is_absent() {
                continue;
            }
            let start = sanitize_pose(
                start_clip.sample(joint_index, span.start_time_seconds),
                joint.bind,
            );
            let contact = sanitize_pose(
                contact_clip.sample(joint_index, span.contact_time_seconds),
                joint.bind,
            );
            let end = sanitize_pose(
                end_clip.sample(joint_index, span.end_time_seconds),
                joint.bind,
            );
            let outgoing = sanitize_pose(
                outgoing_clip.sample(joint_index, span.outgoing_time_seconds),
                joint.bind,
            );
            let finish = sanitize_pose(
                finish_clip.sample(joint_index, span.finish_time_seconds),
                joint.bind,
            );
            let sample = sanitize_pose(
                start.continuation_transition(
                    ContinuationTransition::from_span(contact, end, outgoing, finish, span),
                    span.progress,
                ),
                joint.bind,
            );
            if is_pelvis && span.start.layer == ClipLayer::CombatUpper {
                accumulate_local_pose(
                    &mut combat_upper,
                    &mut combat_upper_weight,
                    sample,
                    span.weight,
                )
                .ok()?;
                continue;
            }
            if is_pelvis && span.start.layer == ClipLayer::CombatLower {
                accumulate_local_pose(
                    &mut combat_lower,
                    &mut combat_lower_weight,
                    sample,
                    span.weight,
                )
                .ok()?;
                continue;
            }
            accumulate_local_pose(&mut blended, &mut accumulated, sample, span.weight).ok()?;
        }
        let mut result =
            if is_pelvis && !combat_upper_weight.is_absent() && !combat_lower_weight.is_absent() {
                locomotion_pelvis_rotation = Some(combat_lower.rotation);
                combine_combat_pelvis(combat_upper, combat_lower)
            } else if !accumulated.is_absent() {
                blended
            } else {
                joint.bind
            };
        if let Some(hand) = hand_animation_joint(definition, joint_index) {
            let mut hand_pose = joint.bind;
            let mut hand_weight = BlendWeight::ZERO;
            for (weighted, clip, _) in &baked {
                if weighted.clip.layer != hand
                    || clip.track_source(joint_index) != BoneTrackSource::Authored
                    || weighted.weight.is_absent()
                {
                    continue;
                }
                let sample =
                    sanitize_pose(clip.sample(joint_index, weighted.time_seconds), joint.bind);
                accumulate_local_pose(&mut hand_pose, &mut hand_weight, sample, weighted.weight)
                    .ok()?;
            }
            if !hand_weight.is_absent() {
                result = hand_pose;
            }
        }
        pose.push(result);
    }
    metrics.sampled_pose_count = metrics.sampled_pose_count.saturating_add(1);
    Some(SampledPlan {
        pose,
        locomotion_pelvis_rotation,
    })
}

fn accumulate_local_pose(
    accumulated_pose: &mut LocalPose,
    accumulated_weight: &mut BlendWeight,
    sample: LocalPose,
    weight: BlendWeight,
) -> AnimationValueResult<()> {
    let next_total = accumulated_weight.checked_add(weight)?;
    let next_pose = if accumulated_weight.is_absent() {
        sample
    } else {
        accumulated_pose.interpolate(
            sample,
            UnitInterval::from_fraction(weight.factor() / next_total.factor())?,
        )
    };
    *accumulated_pose = next_pose;
    *accumulated_weight = next_total;
    Ok(())
}

fn combine_combat_pelvis(combat: LocalPose, locomotion: LocalPose) -> LocalPose {
    LocalPose {
        translation: locomotion.translation,
        rotation: combat.rotation,
        scale: locomotion.scale,
    }
}

fn sample_source_clip(clip: &AnimationClip, joint: &RigJoint, time_seconds: ClipTime) -> LocalPose {
    let translation_field = animated_field!(Transform::translation);
    let rotation_field = animated_field!(Transform::rotation);
    let scale_field = animated_field!(Transform::scale);
    LocalPose {
        translation: clip
            .sample_clamped(translation_field, joint.target, time_seconds.seconds())
            .unwrap_or(joint.bind.translation),
        rotation: clip
            .sample_clamped(rotation_field, joint.target, time_seconds.seconds())
            .unwrap_or(joint.bind.rotation),
        scale: clip
            .sample_clamped(scale_field, joint.target, time_seconds.seconds())
            .unwrap_or(joint.bind.scale),
    }
}

fn conform_upcoming_pose_to_terrain(
    definition: &RigDefinition,
    pose: &mut RigJointTable<LocalPose>,
    plants: &mut [Option<AuthoredContactPlant>; 2],
    conformity: PoseConformity<'_>,
) {
    let PoseConformity {
        owner,
        weights,
        terrain,
        contact_plants,
        locomotion_pelvis_rotation,
    } = conformity;
    if contact_plants == ContactPlantPolicy::Reset {
        *plants = [None; 2];
    }
    let locomotion_reference = locomotion_pelvis_rotation.and_then(|rotation| {
        definition
            .joints
            .iter()
            .position(|joint| joint.role == Some(BoneRole::Pelvis))
            .map(RigJointOrdinal::from)
            .map(|pelvis| {
                let mut reference = pose.clone();
                reference[pelvis].rotation = rotation;
                reference
            })
    });
    let mut locomotion_reference_cache = locomotion_reference
        .as_ref()
        .map(|reference| RigJointTable::from(vec![None; reference.len()]));
    for (index, (left, weight, names)) in [
        (true, weights.left(), RigJointName::LEFT_LEG_CHAIN),
        (false, weights.right(), RigJointName::RIGHT_LEG_CHAIN),
    ]
    .into_iter()
    .enumerate()
    {
        let weight = weight.fraction().clamp(0.0, 1.0);
        if weight <= 0.0 && locomotion_pelvis_rotation.is_none() {
            plants[index] = None;
            continue;
        }
        let [Some(upper), Some(lower), Some(foot)] = names.map(|name| {
            definition
                .joints
                .iter()
                .position(|joint| {
                    joint
                        .name
                        .as_ref()
                        .is_some_and(|joint_name| joint_name.eq_ignore_ascii_case(&name))
                })
                .map(RigJointOrdinal::from)
        }) else {
            continue;
        };
        let mut cache = RigJointTable::from(vec![None; pose.len()]);
        let upper_global = local_pose_global(definition, pose, upper, &mut cache);
        let lower_global = local_pose_global(definition, pose, lower, &mut cache);
        let foot_global = local_pose_global(definition, pose, foot, &mut cache);
        let foot_world = owner.transform_point(foot_global.translation);
        let reference_foot = match (&locomotion_reference, locomotion_reference_cache.as_mut()) {
            (Some(reference), Some(cache)) => local_pose_global(definition, reference, foot, cache),
            _ => foot_global,
        };
        let reference_foot_world = owner.transform_point(reference_foot.translation);
        let reference_foot_rotation_world = owner.rotation() * reference_foot.rotation;
        let terrain_target_world = ScenePlanPoint::from_metres(reference_foot_world.xz())
            .and_then(|point| terrain.and_then(|terrain| terrain.height_at(point)))
            .map(|height| {
                reference_foot_world.with_y(
                    reference_foot_world
                        .y
                        .max(height.metres() + measured_ankle_sole_offset_metres().metres()),
                )
            })
            .unwrap_or(reference_foot_world);
        let supported =
            terrain.is_some() && contact_plants == ContactPlantPolicy::Retain && weight > 0.05;
        if !supported {
            plants[index] = None;
        } else if plants[index].is_none() {
            plants[index] = Some(AuthoredContactPlant {
                position_world: terrain_target_world,
                rotation_world: reference_foot_rotation_world,
                reference_owner_position: owner
                    .affine()
                    .inverse()
                    .transform_point3(terrain_target_world),
                reference_owner_rotation: owner.rotation().inverse()
                    * reference_foot_rotation_world,
            });
        }
        if let Some(mut plant) = plants[index] {
            let reference_world = owner.transform_point(plant.reference_owner_position);
            let reference_rotation_world = owner.rotation() * plant.reference_owner_rotation;
            let displacement = reference_world.xz() - plant.position_world.xz();
            let distance = displacement.length();
            if distance > pose_tuning().authored_contact_plant_limit_metres.metres() {
                let excess = distance - pose_tuning().authored_contact_plant_limit_metres.metres();
                plant.position_world +=
                    Vec3::new(displacement.x, 0.0, displacement.y).normalize_or_zero() * excess;
                plant.rotation_world = hemisphere_slerp(
                    plant.rotation_world,
                    reference_rotation_world,
                    (excess / distance).clamp(0.0, 1.0),
                );
            }
            if let Some(plant_height) = ScenePlanPoint::from_metres(plant.position_world.xz())
                .and_then(|point| terrain.and_then(|terrain| terrain.height_at(point)))
            {
                plant.position_world.y = plant
                    .position_world
                    .y
                    .max(plant_height.metres() + measured_ankle_sole_offset_metres().metres());
            }
            plants[index] = Some(plant);
        }
        let (terrain_world, preserved_foot_rotation) = if let Some(plant) = plants[index] {
            (
                // Contact ownership is discrete. Smoothness comes from the
                // pose buffer interpolating the previous displayed pose into
                // this conformed upcoming target; blending the target itself
                // by the contact scalar lets the planted ankle follow the
                // authored clip and the rotating owner.
                plant.position_world,
                owner.rotation().inverse() * plant.rotation_world,
            )
        } else {
            (terrain_target_world, reference_foot.rotation)
        };
        if terrain_world.distance(foot_world) <= 0.0001
            && preserved_foot_rotation.angle_between(foot_global.rotation) <= 0.0001
        {
            // An identity target is not an identity two-bone solve: pole
            // selection can still perturb an authored chain. Treat an already
            // clear upcoming sample as a strict no-op.
            continue;
        }
        // Authored locomotion is calibrated against flat ground. A fixed
        // ankle-to-sole target is only a lower bound here: lowering a pitched
        // or non-flat authored combat foot changes its intended sole clearance
        // and can bury the visible mesh even on level terrain. Preserve the
        // upcoming authored height unless terrain actually rises into it.
        let target_world = if plants[index].is_some() {
            terrain_world
        } else {
            reference_foot_world.lerp(terrain_world, weight)
        };
        let target = owner.affine().inverse().transform_point3(target_world);
        let hip = upper_global.translation;
        let knee = lower_global.translation;
        let ankle = foot_global.translation;
        let upper_length = hip.distance(knee);
        let lower_length = knee.distance(ankle);
        let Some(solution) = solve_pose_two_bone(
            hip,
            knee,
            target,
            upper_length,
            lower_length,
            if left { Vec3::NEG_Z } else { Vec3::Z },
        ) else {
            continue;
        };
        aim_pose_joint(definition, pose, upper, knee - hip, solution.0 - hip);
        let mut cache = RigJointTable::from(vec![None; pose.len()]);
        let lower_after = local_pose_global(definition, pose, lower, &mut cache);
        let foot_after = local_pose_global(definition, pose, foot, &mut cache);
        aim_pose_joint(
            definition,
            pose,
            lower,
            foot_after.translation - lower_after.translation,
            solution.1 - solution.0,
        );
        let mut cache = RigJointTable::from(vec![None; pose.len()]);
        let parent_rotation = definition.joints[foot]
            .parent
            .map(|parent| local_pose_global(definition, pose, parent, &mut cache).rotation)
            .unwrap_or(Quat::IDENTITY);
        let local = parent_rotation.inverse() * preserved_foot_rotation;
        if local.is_finite() {
            pose[foot].rotation = local.normalize();
        }
    }
}

fn solve_pose_two_bone(
    hip: Vec3,
    current_knee: Vec3,
    target: Vec3,
    upper_length: f32,
    lower_length: f32,
    fallback_pole: Vec3,
) -> Option<(Vec3, Vec3)> {
    let offset = target - hip;
    let direction = offset.try_normalize()?;
    let distance = offset.length().clamp(
        (upper_length - lower_length).abs() + 0.0001,
        upper_length + lower_length - 0.0001,
    );
    let end = hip + direction * distance;
    let along = (upper_length * upper_length - lower_length * lower_length + distance * distance)
        / (2.0 * distance);
    let height = (upper_length * upper_length - along * along)
        .max(0.0)
        .sqrt();
    let bend = (current_knee - hip)
        .reject_from_normalized(direction)
        .try_normalize()
        .or_else(|| {
            fallback_pole
                .reject_from_normalized(direction)
                .try_normalize()
        })?;
    let knee = hip + direction * along + bend * height;
    (knee.is_finite() && end.is_finite()).then_some((knee, end))
}

fn sanitize_pose(pose: LocalPose, fallback: LocalPose) -> LocalPose {
    if pose.translation.is_finite()
        && pose.rotation.is_finite()
        && pose.rotation.length_squared() > 1e-8
        && pose.scale.is_finite()
    {
        LocalPose {
            rotation: pose.rotation.normalize(),
            ..pose
        }
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose(translation: Vec3, rotation: Quat) -> LocalPose {
        LocalPose {
            translation,
            rotation,
            scale: Vec3::ONE,
        }
    }

    #[test]
    fn combat_pelvis_uses_attack_rotation_and_locomotion_translation() {
        let combat = LocalPose {
            translation: Vec3::splat(9.0),
            rotation: Quat::from_rotation_y(0.7),
            scale: Vec3::splat(2.0),
        };
        let locomotion = LocalPose {
            translation: Vec3::new(0.1, 0.2, 0.3),
            rotation: Quat::from_rotation_x(0.4),
            scale: Vec3::splat(1.1),
        };

        let combined = combine_combat_pelvis(combat, locomotion);

        assert_eq!(combined.translation, locomotion.translation);
        assert_eq!(combined.rotation, combat.rotation);
        assert_eq!(combined.scale, locomotion.scale);
    }

    #[test]
    fn pose_extrapolation_extends_translation_and_shortest_rotation() {
        let start = pose(Vec3::ZERO, Quat::IDENTITY);
        let end = pose(Vec3::X, Quat::from_rotation_y(0.5));
        let drawback = start.extrapolate(end, AttackPoseCoordinate::from_coordinate(-0.5).unwrap());
        let follow_through =
            start.extrapolate(end, AttackPoseCoordinate::from_coordinate(1.5).unwrap());
        assert!((drawback.translation.x + 0.5).abs() < 1.0e-5);
        assert!((follow_through.translation.x - 1.5).abs() < 1.0e-5);
        assert!(
            drawback
                .rotation
                .angle_between(Quat::from_rotation_y(-0.25))
                < 1.0e-4
        );
        assert!(
            follow_through
                .rotation
                .angle_between(Quat::from_rotation_y(0.75))
                < 1.0e-4
        );
    }

    #[test]
    fn anchor_curve_span_conserves_transform_velocity_through_contact() {
        let start = LocalPose {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        let end = LocalPose {
            translation: Vec3::X,
            rotation: Quat::from_rotation_y(0.2),
            scale: Vec3::ONE,
        };
        let step = 0.001;
        let before = start.extrapolate(
            end,
            AttackPoseCoordinate::from_coordinate(1.0 - step).unwrap(),
        );
        let contact = start.extrapolate(end, AttackPoseCoordinate::from_coordinate(1.0).unwrap());
        let after = start.extrapolate(
            end,
            AttackPoseCoordinate::from_coordinate(1.0 + step).unwrap(),
        );
        let incoming_velocity = (contact.translation.x - before.translation.x) / step;
        let outgoing_velocity = (after.translation.x - contact.translation.x) / step;
        assert!((incoming_velocity - outgoing_velocity).abs() < 0.01);
        let incoming_angular_velocity = before.rotation.angle_between(contact.rotation) / step;
        let outgoing_angular_velocity = contact.rotation.angle_between(after.rotation) / step;
        assert!((incoming_angular_velocity - outgoing_angular_velocity).abs() < 0.01);
    }

    #[test]
    fn continuation_spline_interpolates_anchors_and_preserves_derivatives() {
        let guard = LocalPose {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        let contact = LocalPose {
            translation: Vec3::X,
            rotation: Quat::from_rotation_y(0.4),
            scale: Vec3::ONE,
        };
        let preparation = LocalPose {
            translation: Vec3::new(1.5, 0.2, 0.0),
            rotation: Quat::from_rotation_y(0.7),
            scale: Vec3::ONE,
        };
        let follow = LocalPose {
            translation: Vec3::new(1.7, 0.5, 0.1),
            rotation: Quat::from_rotation_y(1.0),
            scale: Vec3::ONE,
        };
        let finish = LocalPose {
            translation: Vec3::new(0.2, 0.1, -0.1),
            rotation: Quat::from_rotation_y(0.1),
            scale: Vec3::ONE,
        };
        let coordinate = 1.2;
        let incoming_tangent = 0.35;
        let ready_phase = 7.0 / 24.0;
        let transition = ContinuationTransition {
            contact,
            ready: preparation,
            outgoing: follow,
            finish,
            start_coordinate: AttackPoseCoordinate::from_coordinate(coordinate).unwrap(),
            incoming_tangent: IncomingAttackTangent::from_coordinate_delta(incoming_tangent).unwrap(),
            ready_phase: ContinuationReadiness::from_phase(ready_phase).unwrap(),
        };
        let step = 0.0005;
        let before = guard
            .extrapolate_unbounded(contact, coordinate - incoming_tangent * step / ready_phase);
        let start = guard.continuation_transition(transition, UnitInterval::ZERO);
        let after = guard.continuation_transition(transition, UnitInterval::from_fraction(step).unwrap());
        let incoming_translation_velocity = (start.translation - before.translation) / step;
        let outgoing_translation_velocity = (after.translation - start.translation) / step;
        assert!(incoming_translation_velocity.distance(outgoing_translation_velocity) < 0.02);
        let incoming_angular_velocity = before.rotation.angle_between(start.rotation) / step;
        let outgoing_angular_velocity = start.rotation.angle_between(after.rotation) / step;
        assert!((incoming_angular_velocity - outgoing_angular_velocity).abs() < 0.02);

        let ready = guard.continuation_transition(transition, UnitInterval::from_fraction(ready_phase).unwrap());
        let outgoing = guard.continuation_transition(transition, UnitInterval::HALF);
        let end = guard.continuation_transition(transition, UnitInterval::ONE);
        assert!(ready.translation.distance(preparation.translation) < 0.0001);
        assert!(ready.rotation.angle_between(preparation.rotation) < 0.0001);
        assert!(outgoing.translation.distance(follow.translation) < 0.0001);
        assert!(outgoing.rotation.angle_between(follow.rotation) < 0.0001);
        assert!(end.translation.distance(finish.translation) < 0.0001);
        assert!(end.rotation.angle_between(finish.rotation) < 0.0001);

        let step = 0.002;
        for seam in [ready_phase, 0.5] {
            let before_three = guard.continuation_transition(transition, UnitInterval::from_fraction(seam - 3.0 * step).unwrap());
            let before_two = guard.continuation_transition(transition, UnitInterval::from_fraction(seam - 2.0 * step).unwrap());
            let before_one = guard.continuation_transition(transition, UnitInterval::from_fraction(seam - step).unwrap());
            let at = guard.continuation_transition(transition, UnitInterval::from_fraction(seam).unwrap());
            let after_one = guard.continuation_transition(transition, UnitInterval::from_fraction(seam + step).unwrap());
            let after_two = guard.continuation_transition(transition, UnitInterval::from_fraction(seam + 2.0 * step).unwrap());
            let after_three = guard.continuation_transition(transition, UnitInterval::from_fraction(seam + 3.0 * step).unwrap());
            let velocity_before = (3.0 * at.translation - 4.0 * before_one.translation
                + before_two.translation)
                / (2.0 * step);
            let velocity_after = (-3.0 * at.translation + 4.0 * after_one.translation
                - after_two.translation)
                / (2.0 * step);
            assert!(velocity_before.distance(velocity_after) < 0.01);
            let acceleration_before = (2.0 * at.translation - 5.0 * before_one.translation
                + 4.0 * before_two.translation
                - before_three.translation)
                / (step * step);
            let acceleration_after = (2.0 * at.translation - 5.0 * after_one.translation
                + 4.0 * after_two.translation
                - after_three.translation)
                / (step * step);
            assert!(acceleration_before.distance(acceleration_after) < 2.0);

            let rotation_vector = |pose: LocalPose| {
                quaternion_log(shortest_rotation(pose.rotation * start.rotation.inverse()))
            };
            let angular_velocity_before = (3.0 * rotation_vector(at)
                - 4.0 * rotation_vector(before_one)
                + rotation_vector(before_two))
                / (2.0 * step);
            let angular_velocity_after = (-3.0 * rotation_vector(at)
                + 4.0 * rotation_vector(after_one)
                - rotation_vector(after_two))
                / (2.0 * step);
            assert!(angular_velocity_before.distance(angular_velocity_after) < 0.01);
            let angular_acceleration_before = (2.0 * rotation_vector(at)
                - 5.0 * rotation_vector(before_one)
                + 4.0 * rotation_vector(before_two)
                - rotation_vector(before_three))
                / (step * step);
            let angular_acceleration_after = (2.0 * rotation_vector(at)
                - 5.0 * rotation_vector(after_one)
                + 4.0 * rotation_vector(after_two)
                - rotation_vector(after_three))
                / (step * step);
            assert!(angular_acceleration_before.distance(angular_acceleration_after) < 2.0);
        }

        let before_end = guard.continuation_transition(transition, UnitInterval::from_fraction(1.0 - step).unwrap());
        let before_end_two = guard.continuation_transition(transition, UnitInterval::from_fraction(1.0 - 2.0 * step).unwrap());
        let end_velocity = (3.0 * end.translation - 4.0 * before_end.translation
            + before_end_two.translation)
            / (2.0 * step);
        assert!(end_velocity.length() < 0.01);
        let end_rotation_vector = |pose: LocalPose| {
            quaternion_log(shortest_rotation(pose.rotation * start.rotation.inverse()))
        };
        let end_angular_velocity = (3.0 * end_rotation_vector(end)
            - 4.0 * end_rotation_vector(before_end)
            + end_rotation_vector(before_end_two))
            / (2.0 * step);
        assert!(end_angular_velocity.length() < 0.01);
    }

    #[test]
    fn non_finite_samples_fall_back_to_the_bind_pose() {
        let bind = pose(Vec3::Y, Quat::IDENTITY);
        let invalid = pose(Vec3::splat(f32::NAN), Quat::IDENTITY);
        assert_eq!(sanitize_pose(invalid, bind), bind);
    }

    #[test]
    fn terrain_solve_modifies_next_pose_before_local_pose_interpolation() {
        let joint = |name: &str, parent: Option<usize>, translation: Vec3| {
            let identity = AuthoredRigJoint::new(name.into());
            RigJoint {
                target: AnimationTargetId::from_name(&Name::new(name.to_owned())),
                bind: pose(translation, Quat::IDENTITY),
                parent: parent.map(RigJointOrdinal::from),
                name: Some(identity.name().clone()),
                role: identity.role(),
                body_region: AnimationBodyRegion::Lower,
            }
        };
        let definition = RigDefinition {
            family: "test".into(),
            joints: vec![
                joint("root", None, Vec3::ZERO),
                joint("l_upleg", Some(0), Vec3::Y * 2.0),
                joint("l_lowleg", Some(1), Vec3::NEG_Y),
                joint("l_foot", Some(2), Vec3::NEG_Y),
            ]
            .into(),
            name_ordinals: Default::default(),
        };
        let previous = definition
            .joints
            .iter()
            .map(|joint| joint.bind)
            .collect::<RigJointTable<_>>();
        let mut next = previous.clone();
        let terrain = SceneTerrain::new(2, 2, 1.0, |_| 0.5).unwrap();
        conform_upcoming_pose_to_terrain(
            &definition,
            &mut next,
            &mut [None; 2],
            PoseConformity {
                owner: &GlobalTransform::IDENTITY,
                weights: FootIkInfluence::LEFT,
                terrain: Some(&terrain),
                contact_plants: ContactPlantPolicy::Reset,
                locomotion_pelvis_rotation: None,
            },
        );

        assert_ne!(
            next[RigJointOrdinal::from(1usize)].rotation,
            previous[RigJointOrdinal::from(1usize)].rotation
        );
        let displayed = previous
            .iter()
            .zip(&next)
            .map(|(previous, next)| {
                previous.interpolate(*next, UnitInterval::from_fraction(0.5).unwrap())
            })
            .collect::<RigJointTable<_>>();
        let next_foot = local_pose_global(
            &definition,
            &next,
            RigJointOrdinal::from(3usize),
            &mut RigJointTable::from(vec![None; definition.joints.len()]),
        )
        .translation;
        let displayed_foot = local_pose_global(
            &definition,
            &displayed,
            RigJointOrdinal::from(3usize),
            &mut RigJointTable::from(vec![None; definition.joints.len()]),
        )
        .translation;
        assert!(displayed_foot.y > 0.0);
        assert!(displayed_foot.y < next_foot.y);
        assert!((next_foot.y - (0.5 + measured_ankle_sole_offset_metres().metres())).abs() < 0.001);
    }

    #[test]
    fn pelvis_rotation_compensation_keeps_an_unweighted_foot_at_locomotion_fk() {
        let joint = |name: &str, parent: Option<usize>, translation: Vec3| {
            let identity = AuthoredRigJoint::new(name.into());
            RigJoint {
                target: AnimationTargetId::from_name(&Name::new(name.to_owned())),
                bind: pose(translation, Quat::IDENTITY),
                parent: parent.map(RigJointOrdinal::from),
                name: Some(identity.name().clone()),
                role: identity.role(),
                body_region: AnimationBodyRegion::Lower,
            }
        };
        let definition = RigDefinition {
            family: "test".into(),
            joints: vec![
                joint("root", None, Vec3::ZERO),
                joint("l_upleg", Some(0), Vec3::new(-0.2, 1.8, 0.0)),
                joint("l_lowleg", Some(1), Vec3::new(0.0, -0.9, 0.15)),
                joint("l_foot", Some(2), Vec3::new(0.0, -0.85, 0.15)),
            ]
            .into(),
            name_ordinals: Default::default(),
        };
        let locomotion = definition
            .joints
            .iter()
            .map(|joint| joint.bind)
            .collect::<RigJointTable<_>>();
        let expected = local_pose_global(
            &definition,
            &locomotion,
            RigJointOrdinal::from(3usize),
            &mut RigJointTable::from(vec![None; locomotion.len()]),
        )
        .translation;
        let mut combined = locomotion;
        combined[RigJointOrdinal::from(0usize)].rotation = Quat::from_rotation_y(0.6);

        conform_upcoming_pose_to_terrain(
            &definition,
            &mut combined,
            &mut [None; 2],
            PoseConformity {
                owner: &GlobalTransform::IDENTITY,
                weights: FootIkInfluence::NONE,
                terrain: None,
                contact_plants: ContactPlantPolicy::Reset,
                locomotion_pelvis_rotation: Some(Quat::IDENTITY),
            },
        );

        let compensated = local_pose_global(
            &definition,
            &combined,
            RigJointOrdinal::from(3usize),
            &mut RigJointTable::from(vec![None; combined.len()]),
        )
        .translation;
        assert!(compensated.distance(expected) < 0.0002);
    }

    #[test]
    fn presentation_space_removes_the_controller_centre_height() {
        let owner = GlobalTransform::from_translation(Vec3::Y * 0.95);
        let rig_scene = Transform::from_translation(Vec3::NEG_Y * 0.95);

        let presentation = presentation_world_transform(&owner, &rig_scene);

        assert!(presentation.translation().abs().max_element() < 0.0001);
    }

    #[test]
    fn terrain_solve_does_not_lower_an_authored_flat_ground_foot() {
        let joint = |name: &str, parent: Option<usize>, translation: Vec3| {
            let identity = AuthoredRigJoint::new(name.into());
            RigJoint {
                target: AnimationTargetId::from_name(&Name::new(name.to_owned())),
                bind: pose(translation, Quat::IDENTITY),
                parent: parent.map(RigJointOrdinal::from),
                name: Some(identity.name().clone()),
                role: identity.role(),
                body_region: AnimationBodyRegion::Lower,
            }
        };
        let definition = RigDefinition {
            family: "test".into(),
            joints: vec![
                joint("root", None, Vec3::ZERO),
                joint("l_upleg", Some(0), Vec3::Y * 2.1),
                joint("l_lowleg", Some(1), Vec3::NEG_Y),
                joint("l_foot", Some(2), Vec3::NEG_Y),
            ]
            .into(),
            name_ordinals: Default::default(),
        };
        let mut next = definition
            .joints
            .iter()
            .map(|joint| joint.bind)
            .collect::<RigJointTable<_>>();
        let before = local_pose_global(
            &definition,
            &next,
            RigJointOrdinal::from(3usize),
            &mut RigJointTable::from(vec![None; next.len()]),
        )
        .translation;
        let terrain = SceneTerrain::new(2, 2, 1.0, |_| 0.0).unwrap();

        conform_upcoming_pose_to_terrain(
            &definition,
            &mut next,
            &mut [None; 2],
            PoseConformity {
                owner: &GlobalTransform::IDENTITY,
                weights: FootIkInfluence::LEFT,
                terrain: Some(&terrain),
                contact_plants: ContactPlantPolicy::Reset,
                locomotion_pelvis_rotation: None,
            },
        );

        let after = local_pose_global(
            &definition,
            &next,
            RigJointOrdinal::from(3usize),
            &mut RigJointTable::from(vec![None; next.len()]),
        )
        .translation;
        assert!(before.y > measured_ankle_sole_offset_metres().metres());
        assert!(after.distance(before) < 0.0001);
    }

    #[test]
    fn translating_combat_terrain_conformity_preserves_every_authored_xz_sample() {
        let joint = |name: &str, parent: Option<usize>, translation: Vec3| {
            let identity = AuthoredRigJoint::new(name.into());
            RigJoint {
                target: AnimationTargetId::from_name(&Name::new(name.to_owned())),
                bind: pose(translation, Quat::IDENTITY),
                parent: parent.map(RigJointOrdinal::from),
                name: Some(identity.name().clone()),
                role: identity.role(),
                body_region: AnimationBodyRegion::Lower,
            }
        };
        let definition = RigDefinition {
            family: "test".into(),
            joints: vec![
                joint("root", None, Vec3::ZERO),
                joint("l_upleg", Some(0), Vec3::new(-0.2, 2.085, 0.0)),
                joint("l_lowleg", Some(1), Vec3::NEG_Y),
                joint("l_foot", Some(2), Vec3::new(0.0, -1.003, 0.0)),
                joint("r_upleg", Some(0), Vec3::new(0.2, 2.085, 0.0)),
                joint("r_lowleg", Some(4), Vec3::NEG_Y),
                joint("r_foot", Some(5), Vec3::new(0.0, -1.003, 0.0)),
            ]
            .into(),
            name_ordinals: Default::default(),
        };
        let terrain = SceneTerrain::new(4, 4, 1.0, |_| 0.0).unwrap();
        let paths = [
            Vec2::new(-0.12, 0.0),
            Vec2::new(0.12, 0.0),
            Vec2::new(0.0, -0.12),
            Vec2::new(0.0, 0.12),
            Vec2::new(-0.09, -0.09),
            Vec2::new(0.09, -0.09),
            Vec2::new(-0.09, 0.09),
            Vec2::new(0.09, 0.09),
        ];

        let feet = [3usize, 6].map(RigJointOrdinal::from);
        for (sample_index, displacement) in paths.into_iter().enumerate() {
            let mut authored = definition
                .joints
                .iter()
                .map(|joint| joint.bind)
                .collect::<RigJointTable<_>>();
            authored[RigJointOrdinal::from(3usize)].translation +=
                Vec3::new(displacement.x, 0.0, displacement.y);
            authored[RigJointOrdinal::from(6usize)].translation -=
                Vec3::new(displacement.x, 0.0, displacement.y);
            let before = feet.map(|foot| {
                local_pose_global(
                    &definition,
                    &authored,
                    foot,
                    &mut RigJointTable::from(vec![None; authored.len()]),
                )
                .translation
            });
            let mut conformed = authored;
            conform_upcoming_pose_to_terrain(
                &definition,
                &mut conformed,
                &mut [None; 2],
                PoseConformity {
                    owner: &GlobalTransform::IDENTITY,
                    weights: if sample_index % 2 == 0 {
                        FootIkInfluence::LEFT
                    } else {
                        FootIkInfluence::RIGHT
                    },
                    terrain: Some(&terrain),
                    contact_plants: ContactPlantPolicy::Reset,
                    locomotion_pelvis_rotation: None,
                },
            );
            let after = feet.map(|foot| {
                local_pose_global(
                    &definition,
                    &conformed,
                    foot,
                    &mut RigJointTable::from(vec![None; conformed.len()]),
                )
                .translation
            });
            for foot in 0..2 {
                let xz_delta = after[foot].xz().distance(before[foot].xz());
                assert!(
                    xz_delta < 0.0005,
                    "sample {sample_index} foot {foot} changed XZ by {xz_delta:.6}m"
                );
            }
        }
    }

    #[test]
    fn combat_contact_retains_world_pose_until_the_plant_limit() {
        let joint = |name: &str, parent: Option<usize>, translation: Vec3| {
            let identity = AuthoredRigJoint::new(name.into());
            RigJoint {
                target: AnimationTargetId::from_name(&Name::new(name.to_owned())),
                bind: pose(translation, Quat::IDENTITY),
                parent: parent.map(RigJointOrdinal::from),
                name: Some(identity.name().clone()),
                role: identity.role(),
                body_region: AnimationBodyRegion::Lower,
            }
        };
        let definition = RigDefinition {
            family: "test".into(),
            joints: vec![
                joint("root", None, Vec3::ZERO),
                joint("l_upleg", Some(0), Vec3::new(0.2, 2.1, 0.0)),
                joint("l_lowleg", Some(1), Vec3::NEG_Y),
                joint("l_foot", Some(2), Vec3::NEG_Y),
            ]
            .into(),
            name_ordinals: Default::default(),
        };
        let authored = definition
            .joints
            .iter()
            .map(|joint| joint.bind)
            .collect::<RigJointTable<_>>();
        let terrain = SceneTerrain::new(2, 2, 1.0, |_| 0.0).unwrap();
        let mut plants = [None; 2];
        let mut first = authored.clone();
        conform_upcoming_pose_to_terrain(
            &definition,
            &mut first,
            &mut plants,
            PoseConformity {
                owner: &GlobalTransform::IDENTITY,
                weights: FootIkInfluence::LEFT,
                terrain: Some(&terrain),
                contact_plants: ContactPlantPolicy::Retain,
                locomotion_pelvis_rotation: None,
            },
        );
        let acquired = plants[0].expect("supported foot should acquire a plant");

        let modest_owner =
            GlobalTransform::from(Transform::from_rotation(Quat::from_rotation_y(0.35)));
        let mut modest = authored.clone();
        conform_upcoming_pose_to_terrain(
            &definition,
            &mut modest,
            &mut plants,
            PoseConformity {
                owner: &modest_owner,
                weights: FootIkInfluence::new(
                    UnitInterval::from_fraction(0.25).unwrap(),
                    UnitInterval::ZERO,
                ),
                terrain: Some(&terrain),
                contact_plants: ContactPlantPolicy::Retain,
                locomotion_pelvis_rotation: None,
            },
        );
        let retained = plants[0].expect("plant should remain owned");
        assert!(retained.position_world.distance(acquired.position_world) < 0.0001);
        assert!(
            retained
                .rotation_world
                .angle_between(acquired.rotation_world)
                < 0.0001
        );
        let modest_foot = local_pose_global(
            &definition,
            &modest,
            RigJointOrdinal::from(3usize),
            &mut RigJointTable::from(vec![None; definition.joints.len()]),
        );
        let retained_distance = modest_owner
            .transform_point(modest_foot.translation)
            .distance(acquired.position_world);
        assert!(
            retained_distance < 0.002,
            "retained distance {retained_distance}"
        );
        assert!(
            (modest_owner.rotation() * modest_foot.rotation).angle_between(acquired.rotation_world)
                < 0.001
        );

        let far_owner = GlobalTransform::from(Transform::from_rotation(Quat::from_rotation_y(1.5)));
        let mut far = authored.clone();
        conform_upcoming_pose_to_terrain(
            &definition,
            &mut far,
            &mut plants,
            PoseConformity {
                owner: &far_owner,
                weights: FootIkInfluence::LEFT,
                terrain: Some(&terrain),
                contact_plants: ContactPlantPolicy::Retain,
                locomotion_pelvis_rotation: None,
            },
        );
        let slid = plants[0].expect("plant should slide rather than release");
        let authored_foot = local_pose_global(
            &definition,
            &authored,
            RigJointOrdinal::from(3usize),
            &mut RigJointTable::from(vec![None; definition.joints.len()]),
        );
        let far_authored_world = far_owner.transform_point(authored_foot.translation);
        assert!(slid.position_world.distance(acquired.position_world) > 0.001);
        assert!(
            slid.position_world.xz().distance(far_authored_world.xz())
                <= pose_tuning().authored_contact_plant_limit_metres.metres() + 0.0001
        );
    }
}
