//! Client presentation ragdoll: physics as a third pose source.
//!
//! While the replicated body state is ragdolled, the server's coarse dynamic
//! body owns the character root. This module hangs an articulated body off
//! that root: the pelvis is kinematic and rides the controller, every other
//! body is dynamic, and PD muscles brace the joints toward the pose the
//! character fell in. Bodies exist only while ragdolling. They spawn at the
//! live pose (inheriting the animation's limb velocities and the
//! controller's momentum), write the pose buffer every frame, and despawn on
//! exit, where the ordinary inertialized transition carries the skeleton out
//! of whatever tangle the ragdoll ended in.

use adventuresim_tactical_core::prelude::*;
use bevy::{math::Affine3A, prelude::*};

mod definition;
mod drive;
mod entry;
mod muscles;

pub(super) use drive::{
    apply_ragdoll_poses, drive_ragdoll_pose_buffers, resolve_ragdoll_terrain_contacts,
};
pub(super) use muscles::{apply_muscles, damp_joints};

use self::{
    definition::{PELVIS_BODY, RagdollDefinition},
    entry::{
        MuscleTarget, joint_base_affine, joint_world_affines, live_world_affine, seed_bodies,
        spawn_ragdoll, topological_joint_order,
    },
};
use super::{
    AuthoredBindTransform, PresentedSkeleton,
    pose_buffer::PoseBufferRig,
    procedural::{BoneRole, HumanoidRig},
    skeletal_proportions::SkeletalJointOffset,
};

/// The client-only collision island every ragdoll body lives on.
pub(super) const RAGDOLL_LAYER: u32 = 1 << 7;

/// On a rig owner while its skeleton is a physics ragdoll.
#[derive(Component)]
pub(crate) struct Ragdolling {
    definition: RagdollDefinition,
    /// Everything to despawn on exit: bodies (colliders are their
    /// children) and joints.
    parts: Vec<Entity>,
    /// Body entities aligned with the definition's bodies.
    bodies: Vec<Option<Entity>>,
    /// Pose-buffer joint index to definition body index.
    joint_bodies: Vec<Option<usize>>,
    /// Pose-buffer joints ordered parent before child.
    joint_order: Vec<usize>,
    muscles: Vec<MuscleTarget>,
    /// Seconds since entry; drives the stun, rise, and limp envelope.
    age_seconds: f32,
    /// The owner's uniform world scale at entry: bind-space lengths are
    /// multiplied by it.
    scale: f32,
    /// The pelvis body in the owner's frame at entry; the pelvis rides the
    /// owner there every frame.
    pelvis_owner_local: Affine3A,
}

/// On every body and joint entity, so a ragdoll whose owner disappears is
/// swept up.
#[derive(Component)]
pub(super) struct RagdollPart {
    owner: Entity,
}

/// Capsule geometry for the terrain contact solver, world units.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct RagdollBodyPart {
    radius: f32,
    half_length: f32,
    center_local: Vec3,
    axis_local: Vec3,
    kinematic: bool,
}

/// The owner-space bind transform of a rig node with its skeletal
/// proportion offset applied, composed from the authored locals.
fn proportioned_bind_global(
    entity: Entity,
    owner: Entity,
    bind_nodes: &Query<(
        &AuthoredBindTransform,
        Option<&SkeletalJointOffset>,
        Option<&ChildOf>,
    )>,
) -> Option<Transform> {
    let mut current = entity;
    let mut locals = Vec::new();
    for _ in 0..64 {
        let Ok((bind, offset, parent)) = bind_nodes.get(current) else {
            break;
        };
        if bind.owner != owner {
            break;
        }
        let mut local = bind.local;
        local.translation += offset.map(|offset| offset.0).unwrap_or_default();
        locals.push(local);
        let Some(parent) = parent else {
            break;
        };
        current = parent.parent();
    }
    (!locals.is_empty()).then(|| {
        locals
            .into_iter()
            .rev()
            .fold(Transform::IDENTITY, |global, local| {
                global.mul_transform(local)
            })
    })
}

#[expect(
    clippy::type_complexity,
    reason = "the Bevy query pairs each presented rig with its pose buffer and ragdoll ownership"
)]
pub(super) fn sync_ragdolls(
    mut commands: Commands,
    mut owners: Query<
        (
            Entity,
            &PresentedSkeleton,
            &HumanoidRig,
            &mut PoseBufferRig,
            Option<&Ragdolling>,
        ),
        With<Player>,
    >,
    chain: Query<(&Transform, Option<&ChildOf>)>,
    bind_nodes: Query<(
        &AuthoredBindTransform,
        Option<&SkeletalJointOffset>,
        Option<&ChildOf>,
    )>,
    parts: Query<(Entity, &RagdollPart)>,
) {
    let config = runtime_animation_config().ragdoll;
    for (owner, skeleton, humanoid, mut rig, ragdoll) in &mut owners {
        if skeleton.body() != BodyState::Ragdolled {
            if let Some(ragdoll) = ragdoll {
                exit_ragdoll(&mut commands, owner, &mut rig, ragdoll);
            }
            continue;
        }
        match ragdoll {
            Some(ragdoll) => {
                let target = live_world_affine(owner, &chain) * ragdoll.pelvis_owner_local;
                let (_, rotation, translation) = target.to_scale_rotation_translation();
                if let Some(pelvis) = ragdoll.bodies[PELVIS_BODY] {
                    commands
                        .entity(pelvis)
                        .insert((Position(translation), Rotation(rotation)));
                }
            }
            None => enter_ragdoll(
                &mut commands,
                owner,
                skeleton,
                humanoid,
                &mut rig,
                &chain,
                &bind_nodes,
                &config,
            ),
        }
    }
    for (part, ragdoll_part) in &parts {
        if owners
            .get(ragdoll_part.owner)
            .is_ok_and(|(_, _, _, _, ragdoll)| ragdoll.is_some())
        {
            continue;
        }
        commands.entity(part).despawn();
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "ragdoll entry samples every independently borrowed rig source once"
)]
fn enter_ragdoll(
    commands: &mut Commands,
    owner: Entity,
    skeleton: &PresentedSkeleton,
    humanoid: &HumanoidRig,
    rig: &mut PoseBufferRig,
    chain: &Query<(&Transform, Option<&ChildOf>)>,
    bind_nodes: &Query<(
        &AuthoredBindTransform,
        Option<&SkeletalJointOffset>,
        Option<&ChildOf>,
    )>,
    config: &RagdollConfig,
) {
    let bind = |role: BoneRole| {
        humanoid
            .get(&role)
            .and_then(|entity| proportioned_bind_global(*entity, owner, bind_nodes))
    };
    let Some(definition) = RagdollDefinition::from_bind_pose(bind, &config.capsules) else {
        warn_once!("ragdoll: rig {owner} lacks a driving bone; no presentation ragdoll");
        return;
    };
    let joint_bodies = (0..rig.joint_count())
        .map(|joint| {
            rig.joint_name(joint)
                .and_then(BoneRole::from_name)
                .and_then(|role| definition.body_of(role))
        })
        .collect::<Vec<_>>();
    let joint_order = topological_joint_order(rig);
    let owner_world = live_world_affine(owner, chain);
    let scale = owner_world.to_scale_rotation_translation().0.x;
    let (world_previous, world_next) = joint_world_affines(rig, &joint_order, &|joint| {
        joint_base_affine(rig, joint, chain)
    });
    let seeds = seed_bodies(
        &definition,
        rig,
        &joint_bodies,
        &world_previous,
        &world_next,
        skeleton.world_velocity,
        config,
    );
    let Some(pelvis) = seeds[PELVIS_BODY] else {
        warn_once!("ragdoll: rig {owner} has no pelvis joint in its pose buffer");
        return;
    };
    let spawned = spawn_ragdoll(commands, owner, &definition, &seeds, scale, config);
    let pelvis_owner_local = owner_world.inverse()
        * Affine3A::from_rotation_translation(pelvis.rotation, pelvis.position);
    rig.set_physics_owned(true);
    commands.entity(owner).insert(Ragdolling {
        definition,
        parts: spawned.parts,
        bodies: spawned.bodies,
        joint_bodies,
        joint_order,
        muscles: spawned.muscles,
        age_seconds: 0.0,
        scale,
        pelvis_owner_local,
    });
}

/// Despawn the physics and hand the pose buffer back to authored sampling;
/// its next update inertializes out of the final ragdoll pose.
fn exit_ragdoll(
    commands: &mut Commands,
    owner: Entity,
    rig: &mut PoseBufferRig,
    ragdoll: &Ragdolling,
) {
    for part in &ragdoll.parts {
        commands.entity(*part).despawn();
    }
    commands.entity(owner).remove::<Ragdolling>();
    rig.set_physics_owned(false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ragdoll_collision_layer_interacts_with_itself_only() {
        let layers = CollisionLayers::new(RAGDOLL_LAYER, RAGDOLL_LAYER);
        assert!(layers.interacts_with(layers));
        assert!(!layers.interacts_with(CollisionLayers::new(1 << 3, 1 << 3)));
    }

    #[test]
    fn proportioned_bind_global_composes_authored_locals_with_offsets() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let root = world
            .spawn(AuthoredBindTransform {
                owner,
                local: Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
            })
            .id();
        let bone = world
            .spawn((
                AuthoredBindTransform {
                    owner,
                    local: Transform::from_xyz(1.0, 0.0, 0.0),
                },
                SkeletalJointOffset(Vec3::X * 0.5),
                ChildOf(root),
            ))
            .id();
        let mut state = bevy::ecs::system::SystemState::<
            Query<(
                &AuthoredBindTransform,
                Option<&SkeletalJointOffset>,
                Option<&ChildOf>,
            )>,
        >::new(&mut world);
        let query = state.get(&world);
        let global = proportioned_bind_global(bone, owner, &query).unwrap();
        assert!(
            global.translation.distance(Vec3::new(0.0, 0.0, -1.5)) < 1e-5,
            "{global:?}"
        );
    }
}
