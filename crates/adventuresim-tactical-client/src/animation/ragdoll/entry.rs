//! Ragdoll entry: seed every body from the live pose buffer, then spawn the
//! bodies, colliders, and joints.
//!
//! The seed is pure: each body's world pose comes from the buffered joint
//! transforms composed under the live rig base, and its velocity from the
//! finite difference of the two buffered samples plus the whole-body
//! momentum, so a mid-swing limb carries its swing into physics.

use adventuresim_tactical_core::prelude::*;
use bevy::{math::Affine3A, prelude::*};

use super::super::pose_buffer::PoseBufferRig;
use super::{
    RAGDOLL_LAYER, RagdollBodyPart, RagdollPart,
    definition::{RagdollDefinition, RagdollJointKind},
};

/// One body's entry state: world pose plus the velocities the animation was
/// carrying.
#[derive(Debug, Clone, Copy)]
pub(super) struct BodySeed {
    pub(super) position: Vec3,
    pub(super) rotation: Quat,
    pub(super) linear: Vec3,
    pub(super) angular: Vec3,
}

/// One motored joint: pull `child` toward `relative`, its entry orientation
/// expressed in `parent`'s frame. Indices into the definition's bodies.
#[derive(Debug, Clone, Copy)]
pub(super) struct MuscleTarget {
    pub(super) parent: usize,
    pub(super) child: usize,
    pub(super) relative: Quat,
}

/// World affine of `entity` from the live `Transform` chain. The cached
/// `GlobalTransform` is last frame's propagation, one frame behind a root
/// that presentation moves in `Update`; deriving bone locals through it
/// shifts the whole skeleton by a frame of root motion.
pub(super) fn live_world_affine(
    entity: Entity,
    chain: &Query<(&Transform, Option<&ChildOf>)>,
) -> Affine3A {
    let mut affine = Affine3A::IDENTITY;
    let mut current = Some(entity);
    while let Some(entity) = current {
        let Ok((transform, parent)) = chain.get(entity) else {
            break;
        };
        affine = transform.compute_affine() * affine;
        current = parent.map(ChildOf::parent);
    }
    affine
}

/// Pose-buffer joints ordered parent before child.
pub(super) fn topological_joint_order(rig: &PoseBufferRig) -> Vec<usize> {
    let count = rig.joint_count();
    let mut order = Vec::with_capacity(count);
    let mut placed = vec![false; count];
    while order.len() < count {
        let before = order.len();
        for joint in 0..count {
            if !placed[joint] && rig.joint_parent(joint).is_none_or(|parent| placed[parent]) {
                placed[joint] = true;
                order.push(joint);
            }
        }
        if order.len() == before {
            // A parent cycle cannot happen in a scene hierarchy; refuse to
            // spin rather than trust it.
            break;
        }
    }
    order
}

/// World affine the parent-less joints compose under: the live world of
/// the scene node they hang from.
pub(super) fn joint_base_affine(
    rig: &PoseBufferRig,
    joint: usize,
    chain: &Query<(&Transform, Option<&ChildOf>)>,
) -> Affine3A {
    rig.joint_entity(joint)
        .and_then(|entity| chain.get(entity).ok())
        .and_then(|(_, parent)| parent.map(ChildOf::parent))
        .map(|parent| live_world_affine(parent, chain))
        .unwrap_or(Affine3A::IDENTITY)
}

/// World affines of every joint for the previous and upcoming buffered
/// samples, composed in `order`.
pub(super) fn joint_world_affines(
    rig: &PoseBufferRig,
    order: &[usize],
    base: &dyn Fn(usize) -> Affine3A,
) -> (Vec<Affine3A>, Vec<Affine3A>) {
    let count = rig.joint_count();
    let mut previous = vec![Affine3A::IDENTITY; count];
    let mut next = vec![Affine3A::IDENTITY; count];
    for &joint in order {
        let (parent_previous, parent_next) = match rig.joint_parent(joint) {
            Some(parent) => (previous[parent], next[parent]),
            None => {
                let base = base(joint);
                (base, base)
            }
        };
        previous[joint] = parent_previous * rig.previous_pose(joint).affine();
        next[joint] = parent_next * rig.upcoming_pose(joint).affine();
    }
    (previous, next)
}

/// Angular velocity (rad/s) carrying `current` onto `next` over
/// `delta_seconds`, along the short arc.
fn angular_velocity_between(next: Quat, current: Quat, delta_seconds: f32) -> Vec3 {
    let mut delta = next * current.inverse();
    if delta.w < 0.0 {
        delta = -delta;
    }
    delta.to_scaled_axis() / delta_seconds.max(1e-5)
}

/// Seed each body from the buffered joint that drives it. `momentum` is the
/// whole-body velocity added on top of the per-bone animation velocities.
pub(super) fn seed_bodies(
    definition: &RagdollDefinition,
    rig: &PoseBufferRig,
    joint_bodies: &[Option<usize>],
    world_previous: &[Affine3A],
    world_next: &[Affine3A],
    momentum: Vec3,
    config: &RagdollConfig,
) -> Vec<Option<BodySeed>> {
    let sample_seconds = rig.sample_interval_seconds();
    let mut seeds = vec![None; definition.bodies.len()];
    for (joint, body) in joint_bodies.iter().enumerate() {
        let Some(body) = *body else {
            continue;
        };
        let bind_rotation = definition.bodies[body].bind.rotation;
        let (_, rotation_next, position_next) = world_next[joint].to_scale_rotation_translation();
        let (_, rotation_previous, position_previous) =
            world_previous[joint].to_scale_rotation_translation();
        // `body = bone * bind_rotation⁻¹`: the body rests at identity where
        // the bone rests at its bind rotation.
        let rotation = (rotation_next * bind_rotation.inverse()).normalize();
        let angular = angular_velocity_between(rotation_next, rotation_previous, sample_seconds)
            .clamp_length_max(config.maximum_seed_spin_radians_per_second);
        let linear = momentum
            + ((position_next - position_previous) / sample_seconds)
                .clamp_length_max(config.maximum_seed_speed_metres_per_second);
        let kinematic = definition.bodies[body].kinematic;
        seeds[body] = Some(BodySeed {
            position: position_next,
            rotation,
            linear: if kinematic { Vec3::ZERO } else { linear },
            angular: if kinematic { Vec3::ZERO } else { angular },
        });
    }
    seeds
}

pub(super) struct SpawnedRagdoll {
    pub(super) parts: Vec<Entity>,
    pub(super) bodies: Vec<Option<Entity>>,
    pub(super) muscles: Vec<MuscleTarget>,
}

/// Spawn bodies at their seeds, colliders under them, and the joints between
/// them. Muscle targets capture each joint's relative orientation at entry.
pub(super) fn spawn_ragdoll(
    commands: &mut Commands,
    owner: Entity,
    definition: &RagdollDefinition,
    seeds: &[Option<BodySeed>],
    scale: f32,
    config: &RagdollConfig,
) -> SpawnedRagdoll {
    let mut parts = Vec::new();
    let bodies = spawn_bodies(
        commands, owner, definition, seeds, scale, config, &mut parts,
    );
    let muscles = spawn_joints(
        commands, owner, definition, seeds, scale, &bodies, &mut parts,
    );
    SpawnedRagdoll {
        parts,
        bodies,
        muscles,
    }
}

/// Spawn one rigid body per seeded definition body, each with a capsule
/// collider child. Returns the body entity for each definition index.
fn spawn_bodies(
    commands: &mut Commands,
    owner: Entity,
    definition: &RagdollDefinition,
    seeds: &[Option<BodySeed>],
    scale: f32,
    config: &RagdollConfig,
    parts: &mut Vec<Entity>,
) -> Vec<Option<Entity>> {
    let mut bodies = vec![None; definition.bodies.len()];
    for (index, body) in definition.bodies.iter().enumerate() {
        let Some(seed) = seeds[index] else {
            continue;
        };
        let rigid_body = if body.kinematic {
            RigidBody::Kinematic
        } else {
            RigidBody::Dynamic
        };
        let axis_local = body.collider_local.rotation * Vec3::Y;
        let entity = commands
            .spawn((
                Name::new(format!("Presentation ragdoll {:?}", body.role)),
                rigid_body,
                Position::new(seed.position),
                Rotation(seed.rotation),
                Transform::from_translation(seed.position).with_rotation(seed.rotation),
                LinearVelocity(seed.linear),
                AngularVelocity(seed.angular),
                // Render-rate easing between physics steps; the pose readback
                // in `PostUpdate` then sees smooth body transforms.
                TransformInterpolation,
                // A resting body cycling sleep/wake reads as floor shaking,
                // and these bodies despawn on exit anyway.
                SleepingDisabled,
                SpeculativeMargin(config.speculative_contact_margin_metres),
                RagdollPart { owner },
                RagdollBodyPart {
                    radius: body.shape.radius * scale,
                    half_length: body.shape.length * scale * 0.5,
                    center_local: body.collider_local.translation * scale,
                    axis_local,
                    kinematic: body.kinematic,
                },
            ))
            .remove::<RigidBodyDisabled>()
            .id();
        commands.spawn((
            ChildOf(entity),
            Transform::from_translation(body.collider_local.translation * scale)
                .with_rotation(body.collider_local.rotation),
            Collider::capsule(body.shape.radius * scale, body.shape.length * scale),
            // A client-only collision island: non-neighbouring parts collide
            // with one another and nothing else.
            CollisionLayers::new(RAGDOLL_LAYER, RAGDOLL_LAYER),
        ));
        parts.push(entity);
        bodies[index] = Some(entity);
    }
    bodies
}

/// Spawn the joint between each pair of spawned bodies and record the
/// muscle target that holds it at its entry orientation.
fn spawn_joints(
    commands: &mut Commands,
    owner: Entity,
    definition: &RagdollDefinition,
    seeds: &[Option<BodySeed>],
    scale: f32,
    bodies: &[Option<Entity>],
    parts: &mut Vec<Entity>,
) -> Vec<MuscleTarget> {
    let mut muscles = Vec::with_capacity(definition.joints.len());
    for joint in &definition.joints {
        let (Some(parent), Some(child)) = (bodies[joint.parent], bodies[joint.child]) else {
            continue;
        };
        let (Some(parent_seed), Some(child_seed)) = (seeds[joint.parent], seeds[joint.child])
        else {
            continue;
        };
        let anchor1 = (joint.anchor - definition.bodies[joint.parent].bind.translation) * scale;
        let anchor2 = (joint.anchor - definition.bodies[joint.child].bind.translation) * scale;
        let entity = match joint.kind {
            RagdollJointKind::Spherical {
                twist_axis,
                swing,
                twist,
            } => commands
                .spawn((
                    SphericalJoint::new(parent, child)
                        .with_local_anchor1(anchor1)
                        .with_local_anchor2(anchor2)
                        .with_twist_axis(twist_axis)
                        .with_swing_limits(-swing, swing)
                        .with_twist_limits(-twist, twist),
                    JointCollisionDisabled,
                    RagdollPart { owner },
                ))
                .id(),
            RagdollJointKind::Hinge { axis, limits } => commands
                .spawn((
                    RevoluteJoint::new(parent, child)
                        .with_local_anchor1(anchor1)
                        .with_local_anchor2(anchor2)
                        .with_hinge_axis(axis)
                        .with_angle_limits(limits.0, limits.1),
                    JointCollisionDisabled,
                    RagdollPart { owner },
                ))
                .id(),
        };
        parts.push(entity);
        muscles.push(MuscleTarget {
            parent: joint.parent,
            child: joint.child,
            relative: parent_seed.rotation.inverse() * child_seed.rotation,
        });
    }
    muscles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angular_velocity_takes_the_short_arc() {
        let current = Quat::from_rotation_y(0.3);
        let next = Quat::from_rotation_y(0.5);
        let velocity = angular_velocity_between(next, current, 0.1);
        assert!((velocity.y - 2.0).abs() < 1e-4, "{velocity}");
        let flipped = angular_velocity_between(-next, current, 0.1);
        assert!(flipped.distance(velocity) < 1e-4);
    }
}
