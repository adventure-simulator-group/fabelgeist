//! Physics as a pose source: read the bodies back into the pose buffer,
//! re-assert that pose after the procedural passes, and keep the bodies out
//! of the authoritative terrain heightfield.

use adventuresim_tactical_core::prelude::*;
use bevy::{math::Affine3A, prelude::*};

use super::super::pose_buffer::{LocalPose, PoseBufferRig};
use super::{
    RagdollBodyPart, Ragdolling,
    entry::{joint_base_affine, joint_world_affines},
};

/// Read the ragdoll bodies back into the pose buffer: driven joints follow
/// their body (converted to joint-local through the live parent chain),
/// undriven ancestors of driven joints are held exactly, and everything else
/// (fingers, toes, the face) keeps its last pose.
pub(in crate::animation) fn drive_ragdoll_pose_buffers(
    chain: Query<(&Transform, Option<&ChildOf>)>,
    mut rigs: Query<(&mut PoseBufferRig, &Ragdolling)>,
) {
    for (mut rig, ragdoll) in &mut rigs {
        let count = rig.joint_count();
        let mut driven = vec![false; count];
        for (joint, body) in ragdoll.joint_bodies.iter().enumerate() {
            driven[joint] = body
                .and_then(|body| ragdoll.bodies[body])
                .is_some_and(|entity| chain.contains(entity));
        }
        // Undriven joints with a driven descendant must hold exactly `next`:
        // an interpolated or offset value there shears every physics bone
        // off its body.
        let mut ancestor_of_driven = vec![false; count];
        for &joint in ragdoll.joint_order.iter().rev() {
            if (driven[joint] || ancestor_of_driven[joint])
                && let Some(parent) = rig.joint_parent(joint)
            {
                ancestor_of_driven[parent] = true;
            }
        }
        let mut world = vec![Affine3A::IDENTITY; count];
        for &joint in &ragdoll.joint_order {
            let parent_affine = match rig.joint_parent(joint) {
                Some(parent) => world[parent],
                None => joint_base_affine(&rig, joint, &chain),
            };
            if driven[joint] {
                let body = ragdoll.joint_bodies[joint].expect("driven joints map to a body");
                let entity = ragdoll.bodies[body].expect("driven joints have a body entity");
                let body_affine = chain
                    .get(entity)
                    .map(|(transform, _)| transform.compute_affine())
                    .expect("driven joints have a body transform");
                let bind = ragdoll.definition.bodies[body].bind;
                // `bone = body * bind_rotation`, at the scale the real
                // ancestor chain carries, or the model root's scale would
                // re-apply on top of world-unit locals.
                let bone_world = body_affine
                    * Affine3A::from_scale_rotation_translation(
                        bind.scale * ragdoll.scale,
                        bind.rotation,
                        Vec3::ZERO,
                    );
                let (_, rotation, translation) =
                    (parent_affine.inverse() * bone_world).to_scale_rotation_translation();
                let scale = rig.upcoming_pose(joint).scale;
                rig.pin_joint(
                    joint,
                    LocalPose {
                        translation,
                        rotation,
                        scale,
                    },
                );
                world[joint] = bone_world;
            } else {
                if ancestor_of_driven[joint] {
                    rig.hold_joint(joint);
                }
                world[joint] = parent_affine * rig.upcoming_pose(joint).affine();
            }
        }
    }
}

/// Re-assert the physics pose after every procedural pass: while the body
/// is a ragdoll nothing else may displace a driven bone from its body.
pub(in crate::animation) fn apply_ragdoll_poses(
    rigs: Query<&PoseBufferRig, With<Ragdolling>>,
    mut transforms: Query<&mut Transform, Without<PoseBufferRig>>,
) {
    for rig in &rigs {
        for joint in 0..rig.joint_count() {
            let Some(entity) = rig.joint_entity(joint) else {
                continue;
            };
            let pose = rig.displayed_pose(joint);
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.translation = pose.translation;
                transform.rotation = pose.rotation;
                transform.scale = pose.scale;
            }
        }
    }
}

/// How far a capsule sits below the terrain, sampling both end spheres and
/// the centre: a single centre test lets a tilted limb tunnel into a slope
/// whenever its low endpoint is over higher terrain.
fn capsule_terrain_penetration(
    part: &RagdollBodyPart,
    position: Vec3,
    rotation: Quat,
    mut height_at: impl FnMut(Vec2) -> Option<f32>,
) -> f32 {
    let center = position + rotation * part.center_local;
    let axis = rotation * part.axis_local * part.half_length;
    [center - axis, center, center + axis]
        .into_iter()
        .filter_map(|sample| height_at(sample.xz()).map(|height| height + part.radius - sample.y))
        .fold(0.0_f32, f32::max)
}

/// Resolve body contacts against the authoritative terrain heightfield. The
/// client has no terrain collider; keeping terrain response out of Avian's
/// contact graph also keeps this client-only island apart from the disabled
/// replicated collision world.
pub(in crate::animation) fn resolve_ragdoll_terrain_contacts(
    terrains: Query<&SceneTerrain>,
    mut bodies: Query<(
        &RagdollBodyPart,
        &mut Position,
        &Rotation,
        &mut LinearVelocity,
    )>,
) {
    let retention = runtime_animation_config()
        .ragdoll
        .terrain_horizontal_velocity_retention;
    for (part, mut position, rotation, mut velocity) in &mut bodies {
        if part.kinematic {
            continue;
        }
        let penetration = capsule_terrain_penetration(part, position.0, rotation.0, |point| {
            terrains
                .iter()
                .filter_map(|terrain| terrain.height_at(point))
                .reduce(f32::max)
        });
        if penetration <= 0.0 {
            continue;
        }
        position.0.y += penetration;
        if velocity.y < 0.0 {
            // Settle rather than rebound through the surface between
            // presentation frames.
            velocity.y = 0.0;
        }
        velocity.x *= retention;
        velocity.z *= retention;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilted_capsule_samples_its_low_endpoint_on_a_slope() {
        let part = RagdollBodyPart {
            radius: 0.1,
            half_length: 0.5,
            center_local: Vec3::ZERO,
            axis_local: Vec3::Y,
            kinematic: false,
        };
        let rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let penetration =
            capsule_terrain_penetration(&part, Vec3::new(0.0, 0.2, 0.0), rotation, |point| {
                Some(point.x.max(0.0))
            });
        assert!((penetration - 0.4).abs() < 1.0e-5, "{penetration}");
    }

    #[test]
    fn capsule_offset_from_its_body_origin_is_honoured() {
        let part = RagdollBodyPart {
            radius: 0.1,
            half_length: 0.2,
            center_local: Vec3::NEG_Y * 0.2,
            axis_local: Vec3::NEG_Y,
            kinematic: false,
        };
        let penetration =
            capsule_terrain_penetration(&part, Vec3::new(0.0, 0.45, 0.0), Quat::IDENTITY, |_| {
                Some(0.0)
            });
        // Lowest sample is the far cap centre at y = 0.05, radius 0.1.
        assert!((penetration - 0.05).abs() < 1.0e-5, "{penetration}");
    }
}
